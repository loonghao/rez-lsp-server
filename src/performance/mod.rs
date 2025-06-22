//! Performance monitoring and optimization for the Rez LSP server.

pub mod cache;
pub mod metrics;
pub mod profiler;

pub use cache::{CacheManager, CacheStats};
pub use metrics::{MetricsCollector, PerformanceMetrics};
pub use profiler::{Profiler, ProfilerGuard};

use std::time::{Duration, Instant};

/// Performance configuration settings.
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Enable caching
    pub enable_caching: bool,
    /// Cache size limit in MB
    pub cache_size_mb: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable profiling
    pub enable_profiling: bool,
    /// Maximum number of metrics to keep in memory
    pub max_metrics_history: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            enable_caching: true,
            cache_size_mb: 100,
            cache_ttl_seconds: 300,  // 5 minutes
            enable_profiling: false, // Disabled by default for production
            max_metrics_history: 1000,
        }
    }
}

/// A simple timer for measuring operation duration.
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    /// Start a new timer with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
        }
    }

    /// Get the elapsed time since the timer was started.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get the elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    /// Get the timer name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.elapsed_ms();
        if elapsed > 100 {
            // Log slow operations
            tracing::warn!("Slow operation '{}' took {}ms", self.name, elapsed);
        } else {
            tracing::debug!("Operation '{}' took {}ms", self.name, elapsed);
        }
    }
}

/// Macro for timing operations.
#[macro_export]
macro_rules! time_operation {
    ($name:expr, $block:block) => {{
        let _timer = $crate::performance::Timer::new($name);
        $block
    }};
}

/// Macro for timing async operations.
#[macro_export]
macro_rules! time_async_operation {
    ($name:expr, $block:block) => {{
        let _timer = $crate::performance::Timer::new($name);
        $block
    }};
}

/// Performance optimization hints.
#[derive(Debug, Clone)]
pub enum OptimizationHint {
    /// Cache this result
    Cache { key: String, ttl_seconds: u64 },
    /// Parallelize this operation
    Parallelize { max_concurrency: usize },
    /// Use incremental processing
    Incremental { checkpoint_interval: usize },
    /// Debounce this operation
    Debounce { delay_ms: u64 },
}

/// Performance statistics for the entire system.
#[derive(Debug, Clone)]
pub struct SystemPerformanceStats {
    /// Total number of operations performed
    pub total_operations: u64,
    /// Average operation time in milliseconds
    pub avg_operation_time_ms: f64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub cache_hit_ratio: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Number of active profiling sessions
    pub active_profiles: usize,
}

impl SystemPerformanceStats {
    /// Create empty performance stats.
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            avg_operation_time_ms: 0.0,
            cache_hits: 0,
            cache_misses: 0,
            cache_hit_ratio: 0.0,
            memory_usage_mb: 0.0,
            active_profiles: 0,
        }
    }

    /// Update cache statistics.
    pub fn update_cache_stats(&mut self, hits: u64, misses: u64) {
        self.cache_hits = hits;
        self.cache_misses = misses;
        let total = hits + misses;
        self.cache_hit_ratio = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
    }

    /// Update operation statistics.
    pub fn update_operation_stats(&mut self, total_ops: u64, avg_time_ms: f64) {
        self.total_operations = total_ops;
        self.avg_operation_time_ms = avg_time_ms;
    }

    /// Update memory usage.
    pub fn update_memory_usage(&mut self, usage_mb: f64) {
        self.memory_usage_mb = usage_mb;
    }

    /// Update active profiles count.
    pub fn update_active_profiles(&mut self, count: usize) {
        self.active_profiles = count;
    }
}

impl Default for SystemPerformanceStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new("test_operation");
        assert_eq!(timer.name(), "test_operation");
        assert!(timer.elapsed().as_nanos() > 0);
    }

    #[test]
    fn test_timer_elapsed() {
        let timer = Timer::new("test");
        thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10);
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert!(config.enable_monitoring);
        assert!(config.enable_caching);
        assert_eq!(config.cache_size_mb, 100);
        assert_eq!(config.cache_ttl_seconds, 300);
        assert!(!config.enable_profiling);
        assert_eq!(config.max_metrics_history, 1000);
    }

    #[test]
    fn test_system_performance_stats() {
        let mut stats = SystemPerformanceStats::new();

        stats.update_cache_stats(80, 20);
        assert_eq!(stats.cache_hits, 80);
        assert_eq!(stats.cache_misses, 20);
        assert_eq!(stats.cache_hit_ratio, 0.8);

        stats.update_operation_stats(1000, 25.5);
        assert_eq!(stats.total_operations, 1000);
        assert_eq!(stats.avg_operation_time_ms, 25.5);

        stats.update_memory_usage(150.5);
        assert_eq!(stats.memory_usage_mb, 150.5);

        stats.update_active_profiles(3);
        assert_eq!(stats.active_profiles, 3);
    }

    #[test]
    fn test_cache_hit_ratio_calculation() {
        let mut stats = SystemPerformanceStats::new();

        // Test with no operations
        stats.update_cache_stats(0, 0);
        assert_eq!(stats.cache_hit_ratio, 0.0);

        // Test with perfect hit ratio
        stats.update_cache_stats(100, 0);
        assert_eq!(stats.cache_hit_ratio, 1.0);

        // Test with no hits
        stats.update_cache_stats(0, 100);
        assert_eq!(stats.cache_hit_ratio, 0.0);

        // Test with mixed results
        stats.update_cache_stats(75, 25);
        assert_eq!(stats.cache_hit_ratio, 0.75);
    }

    #[tokio::test]
    async fn test_time_operation_macro() {
        let result = time_operation!("test_macro", {
            thread::sleep(Duration::from_millis(1));
            42
        });
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_time_async_operation_macro() {
        let result = time_async_operation!("test_async_macro", {
            tokio::time::sleep(Duration::from_millis(1)).await;
            "hello"
        });
        assert_eq!(result, "hello");
    }
}
