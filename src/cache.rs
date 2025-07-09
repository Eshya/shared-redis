use crate::config::{get_redis_conn_manager_optional, get_cache_ttl, AsyncConnManager};
use anyhow::Result as AnyResult;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use log::{info, error, debug};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse<T> {
    pub data: T,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub cache_key: String,
}

impl<T> CachedResponse<T> {
    pub fn new(data: T, cache_key: String) -> Self {
        Self {
            data,
            cached_at: chrono::Utc::now(),
            cache_key,
        }
    }
}

pub struct CacheManager {
    conn: Option<AsyncConnManager>,
}

impl CacheManager {
    pub async fn new() -> Self {
        let conn = get_redis_conn_manager_optional().await;
        Self { conn }
    }

    pub fn is_available(&self) -> bool {
        self.conn.is_some()
    }

    /// Generate a cache key from request data using SHA256 hash
    pub fn generate_cache_key<T: Serialize>(prefix: &str, request_data: &T) -> AnyResult<String> {
        let serialized = serde_json::to_string(request_data)?;
        let mut hasher = Sha256::new();
        hasher.update(serialized.as_bytes());
        let hash = hex::encode(hasher.finalize());
        Ok(format!("{}:{}", prefix, hash))
    }

    /// Get cached response by key
    pub async fn get<T>(&mut self, key: &str) -> AnyResult<Option<CachedResponse<T>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(ref mut conn) = self.conn {
            match conn.get::<&str, String>(key).await {
                Ok(cached_data) => {
                    debug!("Cache HIT for key: {}", key);
                    match serde_json::from_str::<CachedResponse<T>>(&cached_data) {
                        Ok(response) => Ok(Some(response)),
                        Err(e) => {
                            error!("Failed to deserialize cached data for key {}: {}", key, e);
                            // Clean up corrupted cache entry
                            let _: Result<(), redis::RedisError> = conn.del(key).await;
                            Ok(None)
                        }
                    }
                }
                Err(e) => {
                    if e.to_string().contains("nil") || e.to_string().contains("not found") {
                        debug!("Cache MISS for key: {}", key);
                        Ok(None)
                    } else {
                        error!("Redis error while getting key {}: {}", key, e);
                        Ok(None)
                    }
                }
            }
        } else {
            debug!("Redis not available, returning cache miss for key: {}", key);
            Ok(None)
        }
    }

    /// Set cached response with TTL
    pub async fn set<T>(&mut self, key: &str, data: &CachedResponse<T>) -> AnyResult<bool>
    where
        T: Serialize,
    {
        if let Some(ref mut conn) = self.conn {
            let serialized = serde_json::to_string(data)?;
            let ttl = get_cache_ttl() as usize;
            
            match conn.set_ex::<&str, String, ()>(key, serialized, ttl).await {
                Ok(_) => {
                    debug!("Cache SET for key: {} with TTL: {}s", key, ttl);
                    Ok(true)
                }
                Err(e) => {
                    error!("Failed to set cache for key {}: {}", key, e);
                    Ok(false)
                }
            }
        } else {
            debug!("Redis not available, skipping cache set for key: {}", key);
            Ok(false)
        }
    }

    /// Cache a response
    pub async fn cache_response<T, R>(
        &mut self,
        cache_prefix: &str,
        request_data: &R,
        response_data: T,
    ) -> AnyResult<CachedResponse<T>>
    where
        T: Serialize + Clone,
        R: Serialize,
    {
        let cache_key = Self::generate_cache_key(cache_prefix, request_data)?;
        let cached_response = CachedResponse::new(response_data.clone(), cache_key.clone());
        
        if self.set(&cache_key, &cached_response).await? {
            info!("Successfully cached response for key: {}", cache_key);
        }
        
        Ok(cached_response)
    }

    /// Get cached response
    pub async fn get_cached_response<T, R>(
        &mut self,
        cache_prefix: &str,
        request_data: &R,
    ) -> AnyResult<Option<CachedResponse<T>>>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize,
    {
        let cache_key = Self::generate_cache_key(cache_prefix, request_data)?;
        self.get(&cache_key).await
    }

    /// Delete cache entry by key
    pub async fn delete(&mut self, key: &str) -> AnyResult<bool> {
        if let Some(ref mut conn) = self.conn {
            match conn.del::<&str, u32>(key).await {
                Ok(deleted_count) => {
                    debug!("Deleted {} cache entries for key: {}", deleted_count, key);
                    Ok(deleted_count > 0)
                }
                Err(e) => {
                    error!("Failed to delete cache for key {}: {}", key, e);
                    Ok(false)
                }
            }
        } else {
            debug!("Redis not available, skipping cache delete for key: {}", key);
            Ok(false)
        }
    }

    /// Clear cache entries matching a pattern
    pub async fn clear_pattern(&mut self, pattern: &str) -> AnyResult<u32> {
        if let Some(ref mut conn) = self.conn {
            let keys: Vec<String> = conn.keys(pattern).await.unwrap_or_default();
            let mut deleted_count = 0;
            
            for key in keys {
                if let Ok(count) = conn.del::<String, u32>(key.clone()).await {
                    deleted_count += count;
                }
            }
            
            info!("Cleared {} cache entries matching pattern: {}", deleted_count, pattern);
            Ok(deleted_count)
        } else {
            debug!("Redis not available, skipping pattern clear for: {}", pattern);
            Ok(0)
        }
    }

    /// Get cache statistics
    pub async fn get_cache_info(&mut self) -> AnyResult<HashMap<String, String>> {
        if let Some(ref mut conn) = self.conn {
            let info: String = redis::cmd("INFO")
                .arg("memory")
                .query_async(conn)
                .await
                .unwrap_or_default();
            
            let mut stats = HashMap::new();
            for line in info.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    stats.insert(key.to_string(), value.to_string());
                }
            }
            
            Ok(stats)
        } else {
            let mut stats = HashMap::new();
            stats.insert("status".to_string(), "Redis not available".to_string());
            Ok(stats)
        }
    }
} 