//! Cache Performance Benchmarks
//! 
//! This benchmark suite measures the performance of shared-redis caching operations
//! including cache hits, misses, and key generation.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shared_redis::cache::CacheManager;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Clone)]
struct BenchmarkData {
    id: u32,
    name: String,
    data: Vec<u8>,
    metadata: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct BenchmarkRequest {
    query: String,
    filters: Vec<String>,
    limit: u32,
}

fn create_benchmark_data() -> BenchmarkData {
    BenchmarkData {
        id: 12345,
        name: "Benchmark Test Data".to_string(),
        data: vec![0u8; 1024], // 1KB of data
        metadata: {
            let mut map = std::collections::HashMap::new();
            for i in 0..10 {
                map.insert(format!("key_{}", i), format!("value_{}", i));
            }
            map
        },
    }
}

fn create_benchmark_request() -> BenchmarkRequest {
    BenchmarkRequest {
        query: "SELECT * FROM users WHERE active = true".to_string(),
        filters: vec!["active".to_string(), "verified".to_string()],
        limit: 100,
    }
}

fn cache_key_generation_benchmark(c: &mut Criterion) {
    let request = create_benchmark_request();
    
    c.bench_function("cache_key_generation", |b| {
        b.iter(|| {
            CacheManager::generate_cache_key(
                black_box("benchmark_test"),
                black_box(&request)
            ).unwrap();
        });
    });
}

fn cache_set_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let data = create_benchmark_data();
    
    c.bench_function("cache_set", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut cache_manager = CacheManager::new().await;
                let cached_response = shared_redis::cache::CachedResponse::new(
                    data.clone(),
                    "benchmark_key".to_string()
                );
                let _ = cache_manager.set("benchmark_key", &cached_response).await;
            });
        });
    });
}

fn cache_get_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let data = create_benchmark_data();
    
    // Pre-populate cache
    rt.block_on(async {
        let mut cache_manager = CacheManager::new().await;
        let cached_response = shared_redis::cache::CachedResponse::new(
            data,
            "benchmark_get_key".to_string()
        );
        let _ = cache_manager.set("benchmark_get_key", &cached_response).await;
    });
    
    c.bench_function("cache_get", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut cache_manager = CacheManager::new().await;
                let _ = cache_manager.get::<BenchmarkData>("benchmark_get_key").await;
            });
        });
    });
}

fn cache_hit_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let request = create_benchmark_request();
    let data = create_benchmark_data();
    
    // Pre-populate cache
    rt.block_on(async {
        let mut cache_manager = CacheManager::new().await;
        let _ = cache_manager.cache_response(
            "benchmark_hit_test",
            &request,
            data
        ).await;
    });
    
    c.bench_function("cache_hit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut cache_manager = CacheManager::new().await;
                let _ = cache_manager.get_cached_response::<BenchmarkData, BenchmarkRequest>(
                    "benchmark_hit_test",
                    &request
                ).await;
            });
        });
    });
}

fn cache_miss_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let request = create_benchmark_request();
    
    c.bench_function("cache_miss", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut cache_manager = CacheManager::new().await;
                let _ = cache_manager.get_cached_response::<BenchmarkData, BenchmarkRequest>(
                    "benchmark_miss_test",
                    &request
                ).await;
            });
        });
    });
}

fn bulk_cache_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("bulk_cache_operations", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut cache_manager = CacheManager::new().await;
                
                // Set multiple cache entries
                for i in 0..100 {
                    let data = BenchmarkData {
                        id: i,
                        name: format!("Bulk Test {}", i),
                        data: vec![0u8; 100],
                        metadata: std::collections::HashMap::new(),
                    };
                    
                    let cached_response = shared_redis::cache::CachedResponse::new(
                        data,
                        format!("bulk_key_{}", i)
                    );
                    
                    let _ = cache_manager.set(&format!("bulk_key_{}", i), &cached_response).await;
                }
                
                // Get multiple cache entries
                for i in 0..100 {
                    let _ = cache_manager.get::<BenchmarkData>(&format!("bulk_key_{}", i)).await;
                }
            });
        });
    });
}

criterion_group!(
    benches,
    cache_key_generation_benchmark,
    cache_set_benchmark,
    cache_get_benchmark,
    cache_hit_benchmark,
    cache_miss_benchmark,
    bulk_cache_operations_benchmark
);

criterion_main!(benches);
