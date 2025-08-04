# 🔄 Shared Redis Service

A centralized Redis library for microservices architecture, providing caching, pub/sub messaging, and data persistence capabilities with automatic connection management and error handling.

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Performance](#performance)
- [Contributing](#contributing)

## 🎯 Overview

The `shared-redis` library provides a unified Redis interface for microservices, enabling efficient caching, real-time messaging, and data persistence across your entire application ecosystem. It handles connection management, automatic retries, and provides a clean API for Redis operations.

### Key Benefits

- **⚡ High Performance**: Optimized connection pooling and caching strategies
- **🔄 Real-time Messaging**: Pub/sub capabilities for microservice communication
- **💾 Smart Caching**: Automatic cache key generation and TTL management
- **🛡️ Error Resilience**: Graceful fallbacks when Redis is unavailable
- **🔧 Easy Integration**: Simple API for common Redis operations
- **📊 Monitoring**: Built-in logging and connection status tracking

## 🏗️ Architecture

```mermaid
graph TB
    subgraph "Microservice"
        A[Application Logic] --> B[CacheManager]
        A --> C[Redis Operations]
        A --> D[Pub/Sub Client]
    end
    
    subgraph "Shared Redis Library"
        B --> E[Cache Layer]
        C --> F[Data Operations]
        D --> G[Messaging Layer]
        E --> H[Connection Manager]
        F --> H
        G --> H
    end
    
    subgraph "Redis Infrastructure"
        H --> I[Redis Connection Pool]
        I --> J[Redis Server]
        J --> K[Cache Storage]
        J --> L[Pub/Sub Channels]
        J --> M[Data Storage]
    end
    
    subgraph "Environment"
        N[REDIS_URL] --> H
        O[CACHE_ENABLED] --> E
        P[CACHE_TTL_SECONDS] --> E
    end
```

### Component Flow

```mermaid
sequenceDiagram
    participant App
    participant CacheManager
    participant ConnectionManager
    participant Redis
    
    App->>CacheManager: Cache Request
    CacheManager->>ConnectionManager: Get Connection
    ConnectionManager->>Redis: Execute Command
    
    alt Cache Hit
        Redis-->>ConnectionManager: Return Data
        ConnectionManager-->>CacheManager: Cached Response
        CacheManager-->>App: Return Data
    else Cache Miss
        Redis-->>ConnectionManager: Not Found
        ConnectionManager-->>CacheManager: Cache Miss
        CacheManager-->>App: Generate Data
        App->>CacheManager: Store Data
        CacheManager->>Redis: Set with TTL
    end
```

## ✨ Features

### 🔄 Caching System
- **Automatic Key Generation**: SHA256-based cache keys from request data
- **TTL Management**: Configurable expiration times
- **Cache Hit/Miss Tracking**: Built-in logging and monitoring
- **Graceful Degradation**: Continues operation when Redis is unavailable
- **Data Serialization**: Automatic JSON serialization/deserialization

### 📡 Pub/Sub Messaging
- **Real-time Broadcasting**: Send messages to multiple subscribers
- **Channel Management**: Easy subscription and unsubscription
- **Message Persistence**: Reliable message delivery
- **Connection Pooling**: Efficient connection management

### 💾 Data Operations
- **Key-Value Storage**: Simple get/set operations
- **Conditional Operations**: Set-if-not-exist with expiration
- **Bulk Operations**: Efficient batch processing
- **Idempotent Operations**: Safe retry mechanisms

### 🔧 Connection Management
- **Connection Pooling**: Efficient resource utilization
- **Automatic Reconnection**: Handles connection failures
- **TLS Support**: Secure connections with certificates
- **Environment Configuration**: Flexible deployment options

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
shared-redis = { git = "https://github.com/Bliink-dev/shared-redis", branch = "main" }
```

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `REDIS_URL` | Full Redis connection string | - | ✅ |
| `REDIS_HOST` | Redis host address | `127.0.0.1` | ❌ |
| `REDIS_PORT` | Redis port number | `6379` | ❌ |
| `REDIS_USERNAME` | Redis username | - | ❌ |
| `REDIS_PASSWORD` | Redis password | - | ❌ |
| `CACHE_ENABLED` | Enable/disable caching | `true` | ❌ |
| `CACHE_TTL_SECONDS` | Cache expiration time | `3600` | ❌ |
| `IDEMPOTENT_EXPIRY_IN_SEC` | Idempotent key expiry | `120` | ❌ |

### Example `.env` file:

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

## 🚀 Usage

### Basic Caching

```rust
use shared_redis::cache::CacheManager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct SearchRequest {
    query: String,
    filters: Vec<String>,
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

### Pub/Sub Messaging

```rust
use shared_redis::operations::{broadcasting_data, subscribe_data};

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

### Advanced Caching

```rust
use shared_redis::cache::{CacheManager, CachedResponse};

async fn advanced_caching() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache_manager = CacheManager::new().await;
    
    // Check if cache is available
    if !cache_manager.is_available() {
        println!("Cache not available, continuing without caching");
        return Ok(());
    }
    
    // Generate custom cache key
    let request_data = json!({
        "query": "hotel search",
        "location": "Jakarta",
        "dates": "2025-01-01"
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
    
    // Clear cache by pattern
    cache_manager.clear_pattern("hotel_search:*").await?;
    
    Ok(())
}
```

## 📚 API Reference

### Core Types

#### `CacheManager`
Main cache management service.

```rust
impl CacheManager {
    // Create new cache manager
    pub async fn new() -> Self
    
    // Check if cache is available
    pub fn is_available(&self) -> bool
    
    // Generate cache key from request data
    pub fn generate_cache_key<T: Serialize>(prefix: &str, request_data: &T) -> AnyResult<String>
    
    // Get cached response
    pub async fn get<T>(&mut self, key: &str) -> AnyResult<Option<CachedResponse<T>>>
    
    // Set cached response
    pub async fn set<T>(&mut self, key: &str, data: &CachedResponse<T>) -> AnyResult<bool>
    
    // Cache response with automatic key generation
    pub async fn cache_response<T, R>(
        &mut self,
        cache_prefix: &str,
        request_data: &R,
        response_data: T,
    ) -> AnyResult<CachedResponse<T>>
    
    // Get cached response with automatic key generation
    pub async fn get_cached_response<T, R>(
        &mut self,
        cache_prefix: &str,
        request_data: &R,
    ) -> AnyResult<Option<CachedResponse<T>>>
    
    // Delete cache entry
    pub async fn delete(&mut self, key: &str) -> AnyResult<bool>
    
    // Clear cache by pattern
    pub async fn clear_pattern(&mut self, pattern: &str) -> AnyResult<u32>
    
    // Get cache statistics
    pub async fn get_cache_info(&mut self) -> AnyResult<HashMap<String, String>>
}
```

#### `CachedResponse`
Cached data structure with metadata.

```rust
pub struct CachedResponse<T> {
    pub data: T,                                    // Cached data
    pub cached_at: chrono::DateTime<chrono::Utc>,  // Cache timestamp
    pub cache_key: String,                          // Cache key
}
```

### Operations Module

```rust
// Pub/Sub operations
pub async fn broadcasting_data(db_channel: String, data: String) -> AnyResult<()>
pub async fn subscribe_data(db_channel: String) -> AnyResult<PubSub>

// Data operations
pub async fn set_data<T>(key: String, data: T, conn: AsyncConnManager) -> AnyResult<bool>
pub async fn get_data<T>(key: String, conn: AsyncConnManager) -> AnyResult<Option<T>>
pub async fn set_if_not_exist<T>(key: String, data: T, conn: AsyncConnManager) -> AnyResult<bool>
pub async fn set_with_options<T>(key: String, data: T, conn: AsyncConnManager) -> AnyResult<bool>
```

### Configuration Module

```rust
// Connection management
pub async fn create_redis_pool(redis_uri: &str) -> AnyResult<AsyncConnection>
pub async fn get_redis_pool() -> AnyResult<AsyncConnection>
pub async fn create_redis_conn_manager(redis_uri: &str) -> AnyResult<AsyncConnManager>
pub async fn get_redis_conn_manager() -> AnyResult<AsyncConnManager>
pub async fn get_redis_conn_manager_optional() -> Option<AsyncConnManager>

// Configuration helpers
pub fn is_cache_enabled() -> bool
pub fn get_cache_ttl() -> u64
```

## 📝 Examples

### Hotel Search Caching

```rust
use shared_redis::cache::CacheManager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct HotelSearchRequest {
    check_in: String,
    check_out: String,
    location: String,
    guests: i32,
}

#[derive(Serialize, Deserialize, Clone)]
struct HotelSearchResponse {
    hotels: Vec<Hotel>,
    total: usize,
    session_id: String,
}

async fn search_hotels(request: HotelSearchRequest) -> Result<HotelSearchResponse, Box<dyn std::error::Error>> {
    let mut cache_manager = CacheManager::new().await;
    
    // Try to get cached results
    if let Ok(Some(cached)) = cache_manager.get_cached_response::<HotelSearchResponse, HotelSearchRequest>(
        "hotel_search",
        &request
    ).await {
        log::info!("Cache HIT for hotel search");
        return Ok(cached.data);
    }
    
    // Cache miss - perform API call
    log::info!("Cache MISS for hotel search");
    let response = call_hotel_api(&request).await?;
    
    // Cache the response
    let _ = cache_manager.cache_response(
        "hotel_search",
        &request,
        response.clone()
    ).await;
    
    Ok(response)
}
```

### Real-time Notifications

```rust
use shared_redis::operations::{broadcasting_data, subscribe_data};
use tokio::stream::StreamExt;

// Notification service
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

// Notification listener
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

## 📈 Performance

### Caching Strategy

```mermaid
graph LR
    A[Request] --> B{Cache Hit?}
    B -->|Yes| C[Return Cached Data]
    B -->|No| D[Generate Data]
    D --> E[Cache Data]
    E --> F[Return Data]
```

### Performance Optimizations

1. **Connection Pooling**
   - Reuses connections for better performance
   - Configurable pool size
   - Automatic connection health checks

2. **Cache Key Generation**
   - SHA256 hashing for consistent keys
   - Prefix-based organization
   - Automatic key collision prevention

3. **TTL Management**
   - Configurable expiration times
   - Automatic cleanup of expired entries
   - Memory-efficient storage

4. **Error Handling**
   - Graceful degradation when Redis is unavailable
   - Automatic retry mechanisms
   - Comprehensive error logging

## 🔧 Configuration Options

### Redis Connection

```rust
// Basic connection
let conn = get_redis_conn_manager().await?;

// Optional connection (continues without cache if Redis unavailable)
let conn = get_redis_conn_manager_optional().await;
```

### Cache Settings

```rust
// Check if caching is enabled
if is_cache_enabled() {
    // Use cache
} else {
    // Skip cache
}

// Get cache TTL
let ttl = get_cache_ttl(); // Default: 3600 seconds
```

### Environment Configuration

```rust
// Set Redis URL
std::env::set_var("REDIS_URL", "redis://localhost:6379");

// Enable/disable cache
std::env::set_var("CACHE_ENABLED", "true");

// Set cache TTL
std::env::set_var("CACHE_TTL_SECONDS", "1800"); // 30 minutes
```

## 🚨 Common Issues

### 1. Redis Connection Failed

```bash
# Check Redis URL
echo $REDIS_URL

# Test Redis connection
redis-cli ping
```

### 2. Cache Not Working

```rust
// Check if cache is enabled
println!("Cache enabled: {}", is_cache_enabled());

// Check cache availability
let mut cache_manager = CacheManager::new().await;
println!("Cache available: {}", cache_manager.is_available());
```

### 3. Memory Usage

```rust
// Get cache statistics
let stats = cache_manager.get_cache_info().await?;
println!("Cache stats: {:?}", stats);

// Clear old cache entries
cache_manager.clear_pattern("old_prefix:*").await?;
```

## 🤝 Contributing

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/Bliink-dev/shared-redis.git
   cd shared-redis
   ```

2. **Set up environment**
   ```bash
   # Install Redis
   docker run -d -p 6379:6379 redis:alpine
   
   # Set environment variables
   export REDIS_URL=redis://localhost:6379
   export CACHE_ENABLED=true
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

### Code Style

- Follow Rust coding conventions
- Add tests for new features
- Update documentation for API changes
- Use meaningful commit messages

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_cache_operations
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

For support and questions:

- 📧 Email: support@bliink.id

---

**🔄 Fast • 💾 Reliable • 🔧 Simple**

Author: [Eshya](mailto:achmadayas@gmail.com) 