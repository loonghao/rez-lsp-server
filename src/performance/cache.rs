//! Caching system for performance optimization.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// A cache entry with expiration time.
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// A thread-safe cache with TTL support.
pub struct Cache<K, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
    max_size: usize,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new cache with the given default TTL and maximum size.
    pub fn new(default_ttl: Duration, max_size: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_size,
        }
    }

    /// Get a value from the cache.
    pub async fn get(&self, key: &K) -> Option<V> {
        let data = self.data.read().await;
        if let Some(entry) = data.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Put a value into the cache with default TTL.
    pub async fn put(&self, key: K, value: V) {
        self.put_with_ttl(key, value, self.default_ttl).await;
    }

    /// Put a value into the cache with custom TTL.
    pub async fn put_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut data = self.data.write().await;
        
        // Remove expired entries if we're at capacity
        if data.len() >= self.max_size {
            self.cleanup_expired(&mut data);
            
            // If still at capacity, remove oldest entry
            if data.len() >= self.max_size {
                if let Some(oldest_key) = self.find_oldest_key(&data) {
                    data.remove(&oldest_key);
                }
            }
        }
        
        data.insert(key, CacheEntry::new(value, ttl));
    }

    /// Remove a value from the cache.
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().await;
        data.remove(key).map(|entry| entry.value)
    }

    /// Clear all entries from the cache.
    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let data = self.data.read().await;
        let total_entries = data.len();
        let expired_entries = data.values().filter(|entry| entry.is_expired()).count();
        let active_entries = total_entries - expired_entries;
        
        CacheStats {
            total_entries,
            active_entries,
            expired_entries,
            max_size: self.max_size,
            hit_ratio: 0.0, // This would need to be tracked separately
        }
    }

    /// Clean up expired entries.
    fn cleanup_expired(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        data.retain(|_, entry| !entry.is_expired());
    }

    /// Find the oldest entry key.
    fn find_oldest_key(&self, data: &HashMap<K, CacheEntry<V>>) -> Option<K> {
        data.iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(key, _)| key.clone())
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of entries (including expired)
    pub total_entries: usize,
    /// Number of active (non-expired) entries
    pub active_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
}

/// Cache manager that handles multiple caches.
pub struct CacheManager {
    /// Cache for package discovery results
    package_cache: Cache<String, Vec<crate::core::types::Package>>,
    /// Cache for validation results
    validation_cache: Cache<String, crate::validation::ValidationResult>,
    /// Cache for completion results
    completion_cache: Cache<String, Vec<String>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheManagerStats>>,
}

#[derive(Debug, Clone)]
struct CacheManagerStats {
    hits: u64,
    misses: u64,
    puts: u64,
    evictions: u64,
}

impl CacheManager {
    /// Create a new cache manager.
    pub fn new(config: &super::PerformanceConfig) -> Self {
        let ttl = Duration::from_secs(config.cache_ttl_seconds);
        let max_size = config.cache_size_mb * 1024 / 10; // Rough estimate of entries per MB

        Self {
            package_cache: Cache::new(ttl, max_size),
            validation_cache: Cache::new(ttl, max_size / 2),
            completion_cache: Cache::new(Duration::from_secs(60), max_size / 4), // Shorter TTL for completions
            stats: Arc::new(RwLock::new(CacheManagerStats {
                hits: 0,
                misses: 0,
                puts: 0,
                evictions: 0,
            })),
        }
    }

    /// Get packages from cache.
    pub async fn get_packages(&self, key: &str) -> Option<Vec<crate::core::types::Package>> {
        let result = self.package_cache.get(&key.to_string()).await;
        self.update_stats(result.is_some()).await;
        result
    }

    /// Put packages into cache.
    pub async fn put_packages(&self, key: String, packages: Vec<crate::core::types::Package>) {
        self.package_cache.put(key, packages).await;
        self.increment_puts().await;
    }

    /// Get validation result from cache.
    pub async fn get_validation(&self, key: &str) -> Option<crate::validation::ValidationResult> {
        let result = self.validation_cache.get(&key.to_string()).await;
        self.update_stats(result.is_some()).await;
        result
    }

    /// Put validation result into cache.
    pub async fn put_validation(&self, key: String, result: crate::validation::ValidationResult) {
        self.validation_cache.put(key, result).await;
        self.increment_puts().await;
    }

    /// Get completion results from cache.
    pub async fn get_completions(&self, key: &str) -> Option<Vec<String>> {
        let result = self.completion_cache.get(&key.to_string()).await;
        self.update_stats(result.is_some()).await;
        result
    }

    /// Put completion results into cache.
    pub async fn put_completions(&self, key: String, completions: Vec<String>) {
        self.completion_cache.put(key, completions).await;
        self.increment_puts().await;
    }

    /// Clear all caches.
    pub async fn clear_all(&self) {
        self.package_cache.clear().await;
        self.validation_cache.clear().await;
        self.completion_cache.clear().await;
    }

    /// Get overall cache statistics.
    pub async fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        let total_requests = stats.hits + stats.misses;
        let hit_ratio = if total_requests > 0 {
            stats.hits as f64 / total_requests as f64
        } else {
            0.0
        };

        // Get individual cache stats
        let package_stats = self.package_cache.stats().await;
        let validation_stats = self.validation_cache.stats().await;
        let completion_stats = self.completion_cache.stats().await;

        CacheStats {
            total_entries: package_stats.total_entries + validation_stats.total_entries + completion_stats.total_entries,
            active_entries: package_stats.active_entries + validation_stats.active_entries + completion_stats.active_entries,
            expired_entries: package_stats.expired_entries + validation_stats.expired_entries + completion_stats.expired_entries,
            max_size: package_stats.max_size + validation_stats.max_size + completion_stats.max_size,
            hit_ratio,
        }
    }

    /// Update hit/miss statistics.
    async fn update_stats(&self, is_hit: bool) {
        let mut stats = self.stats.write().await;
        if is_hit {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
    }

    /// Increment put counter.
    async fn increment_puts(&self) {
        let mut stats = self.stats.write().await;
        stats.puts += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = Cache::new(Duration::from_secs(1), 10);
        
        // Test put and get
        cache.put("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        
        // Test non-existent key
        assert_eq!(cache.get(&"key2".to_string()).await, None);
        
        // Test remove
        assert_eq!(cache.remove(&"key1".to_string()).await, Some("value1".to_string()));
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = Cache::new(Duration::from_millis(50), 10);
        
        cache.put("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        
        // Wait for expiration
        sleep(Duration::from_millis(60)).await;
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_size_limit() {
        let cache = Cache::new(Duration::from_secs(10), 2);
        
        cache.put("key1".to_string(), "value1".to_string()).await;
        cache.put("key2".to_string(), "value2".to_string()).await;
        cache.put("key3".to_string(), "value3".to_string()).await;
        
        let stats = cache.stats().await;
        assert!(stats.total_entries <= 2);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = Cache::new(Duration::from_secs(1), 10);
        
        cache.put("key1".to_string(), "value1".to_string()).await;
        cache.put("key2".to_string(), "value2".to_string()).await;
        
        let stats = cache.stats().await;
        assert_eq!(stats.active_entries, 2);
        assert_eq!(stats.max_size, 10);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let config = super::super::PerformanceConfig::default();
        let manager = CacheManager::new(&config);
        
        // Test completion cache
        manager.put_completions("test_key".to_string(), vec!["comp1".to_string(), "comp2".to_string()]).await;
        let completions = manager.get_completions("test_key").await;
        assert_eq!(completions, Some(vec!["comp1".to_string(), "comp2".to_string()]));
        
        // Test stats
        let stats = manager.get_stats().await;
        assert!(stats.hit_ratio > 0.0);
    }
}
