//! Basic Caching Example
//! 
//! This example demonstrates how to use shared-redis for basic caching operations
//! including cache hits, misses, and automatic key generation.

use shared_redis::cache::CacheManager;
use serde::{Deserialize, Serialize};
use log::{info, warn};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserProfile {
    id: u32,
    name: String,
    email: String,
    preferences: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UserRequest {
    user_id: u32,
    include_preferences: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    info!("Starting shared-redis basic caching example");
    
    // Set up Redis connection (you can use environment variables)
    std::env::set_var("REDIS_URL", "redis://localhost:6379");
    std::env::set_var("CACHE_ENABLED", "true");
    std::env::set_var("CACHE_TTL_SECONDS", "3600");
    
    let mut cache_manager = CacheManager::new().await;
    
    // Check if cache is available
    if !cache_manager.is_available() {
        warn!("Redis cache not available, continuing without caching");
    } else {
        info!("Redis cache is available");
    }
    
    // Example 1: Basic caching with automatic key generation
    let request = UserRequest {
        user_id: 123,
        include_preferences: true,
    };
    
    info!("Fetching user profile for user ID: {}", request.user_id);
    
    // Try to get cached response
    if let Ok(Some(cached)) = cache_manager.get_cached_response::<UserProfile, UserRequest>(
        "user_profile",
        &request
    ).await {
        info!("Cache HIT - returning cached user profile");
        info!("Cached data: {:?}", cached.data);
        info!("Cached at: {}", cached.cached_at);
    } else {
        info!("Cache MISS - generating user profile");
        
        // Simulate expensive operation
        let profile = generate_user_profile(&request).await;
        
        // Cache the response
        match cache_manager.cache_response(
            "user_profile",
            &request,
            profile.clone()
        ).await {
            Ok(cached) => {
                info!("Successfully cached user profile");
                info!("Cache key: {}", cached.cache_key);
            }
            Err(e) => {
                warn!("Failed to cache user profile: {}", e);
            }
        }
        
        info!("Generated profile: {:?}", profile);
    }
    
    // Example 2: Manual cache operations
    let cache_key = "manual:user:123";
    let user_data = UserProfile {
        id: 123,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        preferences: vec!["dark_mode".to_string(), "notifications".to_string()],
    };
    
    info!("Setting manual cache entry");
    let cached_response = shared_redis::cache::CachedResponse::new(user_data.clone(), cache_key.to_string());
    
    if let Err(e) = cache_manager.set(cache_key, &cached_response).await {
        warn!("Failed to set cache: {}", e);
    } else {
        info!("Successfully set manual cache entry");
    }
    
    // Retrieve manual cache entry
    if let Ok(Some(cached)) = cache_manager.get::<UserProfile>(cache_key).await {
        info!("Retrieved manual cache entry: {:?}", cached.data);
    }
    
    // Example 3: Cache statistics
    if let Ok(stats) = cache_manager.get_cache_info().await {
        info!("Cache statistics: {:?}", stats);
    }
    
    info!("Basic caching example completed");
    Ok(())
}

async fn generate_user_profile(request: &UserRequest) -> UserProfile {
    // Simulate expensive database query or API call
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    UserProfile {
        id: request.user_id,
        name: format!("User {}", request.user_id),
        email: format!("user{}@example.com", request.user_id),
        preferences: if request.include_preferences {
            vec!["theme".to_string(), "language".to_string()]
        } else {
            vec![]
        },
    }
}
