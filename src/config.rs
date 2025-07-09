use crate::cli::Env;
use anyhow::Result as AnyResult;
pub use redis::{aio::Connection as AsyncConnection, Client, aio::ConnectionManager as AsyncConnManager};
use std::env;
use log::{info, warn};

pub const ENV_REDIS_URL: &str = "REDIS_URL"; // full connection string including timeout, credentials, and schema/namespace
pub const ENV_CACHE_ENABLED: &str = "CACHE_ENABLED"; // enable/disable caching
pub const ENV_CACHE_TTL_SECONDS: &str = "CACHE_TTL_SECONDS"; // cache expiration time

pub fn init_redis_vars() {
    let _env = Env::from_env();
    env::set_var(ENV_REDIS_URL, _env.to_redis_uri());
}

pub fn is_cache_enabled() -> bool {
    env::var(ENV_CACHE_ENABLED)
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase() == "true"
}

pub fn get_cache_ttl() -> u64 {
    env::var(ENV_CACHE_TTL_SECONDS)
        .unwrap_or_else(|_| "3600".to_string())
        .parse()
        .unwrap_or(3600)
}

pub async fn create_redis_pool(redis_uri: &str) -> AnyResult<AsyncConnection> {
    let client = Client::open(redis_uri)?;
    let async_conn = client.get_async_connection().await?;
    Ok(async_conn)
}

pub async fn get_redis_pool() -> AnyResult<AsyncConnection> {
    if let Ok(env_redis_uri) = env::var(ENV_REDIS_URL) {
        let redis_uri = env_redis_uri;
        return create_redis_pool(&redis_uri).await;
    }

    Err(anyhow::anyhow!("Environment variable \"REDIS_URL\" is not set!"))
}

pub async fn create_redis_conn_manager(redis_uri: &str) -> AnyResult<AsyncConnManager> {
    let client = Client::open(redis_uri)?;
    let conn = AsyncConnManager::new(client).await?;
    
    Ok(conn)
}

pub async fn get_redis_conn_manager() -> AnyResult<AsyncConnManager> {
    if let Ok(env_redis_uri) = env::var(ENV_REDIS_URL) {
        let redis_uri = env_redis_uri;
        return create_redis_conn_manager(&redis_uri).await;
    }

    Err(anyhow::anyhow!("Environment variable \"REDIS_URL\" is not set!"))
}

pub async fn get_redis_conn_manager_optional() -> Option<AsyncConnManager> {
    if !is_cache_enabled() {
        info!("Redis caching is disabled");
        return None;
    }

    match get_redis_conn_manager().await {
        Ok(conn) => {
            info!("Redis connection manager created successfully");
            Some(conn)
        }
        Err(e) => {
            warn!("Failed to create Redis connection manager: {}. Continuing without cache.", e);
            None
        }
    }
}