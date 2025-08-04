# ⚡ Quick Start Guide

Get up and running with shared-redis in 5 minutes!

## 🚀 Quick Setup

### 1. Add Dependency

```toml
# Cargo.toml
[dependencies]
shared-redis = { git = "https://github.com/Bliink-dev/shared-redis", branch = "main" }
```

### 2. Environment Variables

```env
# .env
REDIS_URL=redis://localhost:6379
CACHE_ENABLED=true
CACHE_TTL_SECONDS=3600
```

### 3. Basic Caching

```rust
use shared_redis::cache::CacheManager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct SearchRequest {
    query: String,
    location: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SearchResponse {
    results: Vec<String>,
    total: usize,
}

async fn search_with_cache(request: SearchRequest) -> SearchResponse {
    let mut cache_manager = CacheManager::new().await;
    
    // Try to get cached response
    if let Ok(Some(cached)) = cache_manager.get_cached_response::<SearchResponse, SearchRequest>(
        "search_results",
        &request
    ).await {
        println!("Cache HIT - returning cached results");
        return cached.data;
    }
    
    // Cache miss - perform search
    println!("Cache MISS - performing search");
    let response = perform_search(&request).await;
    
    // Cache the response
    let _ = cache_manager.cache_response(
        "search_results",
        &request,
        response.clone()
    ).await;
    
    response
}
```

## 🔄 Common Patterns

### Basic Caching

```rust
use shared_redis::cache::CacheManager;

async fn basic_caching() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache_manager = CacheManager::new().await;
    
    // Check if cache is available
    if !cache_manager.is_available() {
        println!("Cache not available, continuing without caching");
        return Ok(());
    }
    
    // Generate cache key
    let request_data = json!({
        "query": "hotel search",
        "location": "Jakarta"
    });
    
    let cache_key = CacheManager::generate_cache_key("hotel_search", &request_data)?;
    
    // Get cached data
    if let Ok(Some(cached)) = cache_manager.get::<serde_json::Value>(&cache_key).await {
        println!("Found cached data from: {}", cached.cached_at);
        return Ok(());
    }
    
    // Set cached data
    let response_data = json!({
        "hotels": ["Hotel A", "Hotel B"],
        "total": 2
    });
    
    let cached_response = CachedResponse::new(response_data, cache_key.clone());
    cache_manager.set(&cache_key, &cached_response).await?;
    
    Ok(())
}
```

### Pub/Sub Messaging

```rust
use shared_redis::operations::{broadcasting_data, subscribe_data};
use tokio::stream::StreamExt;

// Publisher
async fn publish_event(event_data: String) -> Result<(), Box<dyn std::error::Error>> {
    broadcasting_data("user_events".to_string(), event_data).await?;
    Ok(())
}

// Subscriber
async fn subscribe_to_events() -> Result<(), Box<dyn std::error::Error>> {
    let mut pubsub = subscribe_data("user_events".to_string()).await?;
    
    while let Some(msg) = pubsub.on_message().next().await {
        let payload: String = msg.get_payload()?;
        println!("Received event: {}", payload);
    }
    
    Ok(())
}
```

### Data Operations

```rust
use shared_redis::operations::{set_data, get_data, set_if_not_exist};
use shared_redis::config::get_redis_conn_manager;

async fn data_operations() -> Result<(), Box<dyn std::error::Error>> {
    let conn = get_redis_conn_manager().await?;
    
    // Set data
    set_data("user:123".to_string(), "John Doe".to_string(), conn.clone()).await?;
    
    // Get data
    let user: Option<String> = get_data("user:123".to_string(), conn.clone()).await?;
    
    // Set if not exists (idempotent)
    let created = set_if_not_exist(
        "session:abc".to_string(),
        "active".to_string(),
        conn
    ).await?;
    
    Ok(())
}
```

## 🛠️ Configuration

### Environment Variables

```env
# Redis Connection
REDIS_URL=redis://username:password@localhost:6379
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_USERNAME=your_username
REDIS_PASSWORD=your_password

# Cache Configuration
CACHE_ENABLED=true
CACHE_TTL_SECONDS=3600
IDEMPOTENT_EXPIRY_IN_SEC=120
```

### Connection Options

```rust
use shared_redis::config::{get_redis_conn_manager, get_redis_conn_manager_optional};

// Required connection (fails if Redis unavailable)
let conn = get_redis_conn_manager().await?;

// Optional connection (continues without cache if Redis unavailable)
let conn = get_redis_conn_manager_optional().await;
```

## 📊 Cache Management

### Cache Operations

```rust
use shared_redis::cache::CacheManager;

async fn cache_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache_manager = CacheManager::new().await;
    
    // Delete specific key
    cache_manager.delete("old_key").await?;
    
    // Clear cache by pattern
    let deleted = cache_manager.clear_pattern("hotel_search:*").await?;
    println!("Deleted {} cache entries", deleted);
    
    // Get cache statistics
    let stats = cache_manager.get_cache_info().await?;
    println!("Cache stats: {:?}", stats);
    
    Ok(())
}
```

### Cache Key Generation

```rust
use shared_redis::cache::CacheManager;

// Generate cache key from request data
let request_data = json!({
    "query": "hotel search",
    "location": "Jakarta",
    "dates": "2025-01-01"
});

let cache_key = CacheManager::generate_cache_key("hotel_search", &request_data)?;
println!("Generated cache key: {}", cache_key);
```

## 📡 Pub/Sub Examples

### Real-time Notifications

```rust
use shared_redis::operations::{broadcasting_data, subscribe_data};
use serde_json::json;

// Send notification
async fn send_notification(user_id: i32, message: String) -> Result<(), Box<dyn std::error::Error>> {
    let notification = json!({
        "user_id": user_id,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    broadcasting_data(
        format!("notifications:{}", user_id),
        notification.to_string()
    ).await?;
    
    Ok(())
}

// Listen for notifications
async fn listen_notifications(user_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    let mut pubsub = subscribe_data(format!("notifications:{}", user_id)).await?;
    
    while let Some(msg) = pubsub.on_message().next().await {
        let payload: String = msg.get_payload()?;
        let notification: serde_json::Value = serde_json::from_str(&payload)?;
        
        println!("New notification: {}", notification["message"]);
    }
    
    Ok(())
}
```

### Event Broadcasting

```rust
use shared_redis::operations::{broadcasting_data, subscribe_data};

// Broadcast event
async fn broadcast_event(event_type: &str, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let event = json!({
        "type": event_type,
        "data": data,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    broadcasting_data("events".to_string(), event.to_string()).await?;
    Ok(())
}

// Listen for events
async fn listen_events() -> Result<(), Box<dyn std::error::Error>> {
    let mut pubsub = subscribe_data("events".to_string()).await?;
    
    while let Some(msg) = pubsub.on_message().next().await {
        let payload: String = msg.get_payload()?;
        let event: serde_json::Value = serde_json::from_str(&payload)?;
        
        match event["type"].as_str() {
            Some("user_login") => println!("User logged in: {}", event["data"]["user_id"]),
            Some("booking_created") => println!("New booking: {}", event["data"]["booking_id"]),
            _ => println!("Unknown event: {}", event["type"]),
        }
    }
    
    Ok(())
}
```

## 💾 Data Storage Examples

### Session Management

```rust
use shared_redis::operations::{set_data, get_data, set_if_not_exist};
use shared_redis::config::get_redis_conn_manager;

async fn manage_session(session_id: &str, user_data: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = get_redis_conn_manager().await?;
    
    // Create session if not exists
    let created = set_if_not_exist(
        format!("session:{}", session_id),
        user_data.to_string(),
        conn.clone()
    ).await?;
    
    if created {
        println!("New session created: {}", session_id);
    } else {
        println!("Session already exists: {}", session_id);
    }
    
    // Get session data
    if let Some(session_data) = get_data::<String>(
        format!("session:{}", session_id),
        conn
    ).await? {
        println!("Session data: {}", session_data);
    }
    
    Ok(())
}
```

### User Preferences

```rust
use shared_redis::operations::{set_data, get_data};
use shared_redis::config::get_redis_conn_manager;
use serde_json::json;

async fn manage_user_preferences(user_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    let conn = get_redis_conn_manager().await?;
    
    // Set user preferences
    let preferences = json!({
        "theme": "dark",
        "language": "en",
        "notifications": true
    });
    
    set_data(
        format!("user:{}:preferences", user_id),
        preferences.to_string(),
        conn.clone()
    ).await?;
    
    // Get user preferences
    if let Some(prefs) = get_data::<String>(
        format!("user:{}:preferences", user_id),
        conn
    ).await? {
        let prefs: serde_json::Value = serde_json::from_str(&prefs)?;
        println!("User theme: {}", prefs["theme"]);
    }
    
    Ok(())
}
```

## 🚨 Common Issues

### 1. Redis Connection Failed

```bash
# Check Redis URL
echo $REDIS_URL

# Test Redis connection
redis-cli ping

# Check Redis logs
docker logs redis-container
```

### 2. Cache Not Working

```rust
// Check if cache is enabled
println!("Cache enabled: {}", is_cache_enabled());

// Check cache availability
let mut cache_manager = CacheManager::new().await;
println!("Cache available: {}", cache_manager.is_available());

// Check cache TTL
println!("Cache TTL: {} seconds", get_cache_ttl());
```

### 3. Memory Usage

```rust
// Get cache statistics
let stats = cache_manager.get_cache_info().await?;
println!("Cache stats: {:?}", stats);

// Clear old cache entries
cache_manager.clear_pattern("old_prefix:*").await?;

// Monitor memory usage
let info = cache_manager.get_cache_info().await?;
if let Some(memory_usage) = info.get("used_memory_human") {
    println!("Memory usage: {}", memory_usage);
}
```

### 4. Pub/Sub Not Working

```rust
// Check if Redis supports pub/sub
let conn = get_redis_conn_manager().await?;
let info: String = conn.info("server").await?;
println!("Redis info: {}", info);

// Test pub/sub connection
match subscribe_data("test_channel".to_string()).await {
    Ok(_) => println!("Pub/sub working"),
    Err(e) => println!("Pub/sub error: {}", e),
}
```

## 📚 Next Steps

1. **Read the full documentation**: [README.md](README.md)
2. **Explore examples**: Check the `examples/` directory
3. **Run tests**: `cargo test`
4. **Deploy**: Follow deployment guides in the main documentation

## 🔧 Development

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_cache_operations
```

### Local Development

```bash
# Start Redis locally
docker run -d -p 6379:6379 redis:alpine

# Set environment variables
export REDIS_URL=redis://localhost:6379
export CACHE_ENABLED=true

# Run your application
cargo run
```

---

**Need help?** Check the [main documentation](README.md) or create an issue on GitHub!

**📧 Contact**: [Eshya](mailto:achmadayas@gmail.com) 