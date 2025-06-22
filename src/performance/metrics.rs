//! Performance metrics collection and analysis.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A single performance metric measurement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Name of the operation
    pub name: String,
    /// Duration of the operation in milliseconds
    pub duration_ms: u64,
    /// Timestamp when the operation started
    pub timestamp: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Metric {
    /// Create a new metric.
    pub fn new(name: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            duration_ms: duration.as_millis() as u64,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the metric.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Aggregated performance metrics for an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Operation name
    pub operation: String,
    /// Total number of measurements
    pub count: u64,
    /// Average duration in milliseconds
    pub avg_duration_ms: f64,
    /// Minimum duration in milliseconds
    pub min_duration_ms: u64,
    /// Maximum duration in milliseconds
    pub max_duration_ms: u64,
    /// 95th percentile duration in milliseconds
    pub p95_duration_ms: u64,
    /// 99th percentile duration in milliseconds
    pub p99_duration_ms: u64,
    /// Total duration in milliseconds
    pub total_duration_ms: u64,
    /// Operations per second
    pub ops_per_second: f64,
}

impl PerformanceMetrics {
    /// Create empty performance metrics.
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            count: 0,
            avg_duration_ms: 0.0,
            min_duration_ms: 0,
            max_duration_ms: 0,
            p95_duration_ms: 0,
            p99_duration_ms: 0,
            total_duration_ms: 0,
            ops_per_second: 0.0,
        }
    }

    /// Calculate metrics from a list of durations.
    pub fn from_durations(operation: impl Into<String>, durations: &[u64]) -> Self {
        if durations.is_empty() {
            return Self::new(operation);
        }

        let mut sorted_durations = durations.to_vec();
        sorted_durations.sort_unstable();

        let count = durations.len() as u64;
        let total_duration_ms = durations.iter().sum::<u64>();
        let avg_duration_ms = total_duration_ms as f64 / count as f64;
        let min_duration_ms = sorted_durations[0];
        let max_duration_ms = sorted_durations[sorted_durations.len() - 1];

        let p95_index = ((sorted_durations.len() as f64) * 0.95) as usize;
        let p99_index = ((sorted_durations.len() as f64) * 0.99) as usize;
        let p95_duration_ms = sorted_durations
            .get(p95_index)
            .copied()
            .unwrap_or(max_duration_ms);
        let p99_duration_ms = sorted_durations
            .get(p99_index)
            .copied()
            .unwrap_or(max_duration_ms);

        // Calculate ops per second based on average duration
        let ops_per_second = if avg_duration_ms > 0.0 {
            1000.0 / avg_duration_ms
        } else {
            0.0
        };

        Self {
            operation: operation.into(),
            count,
            avg_duration_ms,
            min_duration_ms,
            max_duration_ms,
            p95_duration_ms,
            p99_duration_ms,
            total_duration_ms,
            ops_per_second,
        }
    }
}

/// Collects and analyzes performance metrics.
pub struct MetricsCollector {
    /// Raw metrics data
    metrics: Arc<RwLock<HashMap<String, VecDeque<Metric>>>>,
    /// Maximum number of metrics to keep per operation
    max_metrics_per_operation: usize,
    /// Start time for calculating rates
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new(max_metrics_per_operation: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            max_metrics_per_operation,
            start_time: Instant::now(),
        }
    }

    /// Record a metric.
    pub async fn record(&self, metric: Metric) {
        let mut metrics = self.metrics.write().await;
        let operation_metrics = metrics
            .entry(metric.name.clone())
            .or_insert_with(VecDeque::new);

        operation_metrics.push_back(metric);

        // Keep only the most recent metrics
        while operation_metrics.len() > self.max_metrics_per_operation {
            operation_metrics.pop_front();
        }
    }

    /// Record a duration for an operation.
    pub async fn record_duration(&self, operation: impl Into<String>, duration: Duration) {
        let metric = Metric::new(operation, duration);
        self.record(metric).await;
    }

    /// Record a duration with metadata.
    pub async fn record_duration_with_metadata(
        &self,
        operation: impl Into<String>,
        duration: Duration,
        metadata: HashMap<String, String>,
    ) {
        let mut metric = Metric::new(operation, duration);
        metric.metadata = metadata;
        self.record(metric).await;
    }

    /// Get performance metrics for an operation.
    pub async fn get_metrics(&self, operation: &str) -> Option<PerformanceMetrics> {
        let metrics = self.metrics.read().await;
        if let Some(operation_metrics) = metrics.get(operation) {
            let durations: Vec<u64> = operation_metrics.iter().map(|m| m.duration_ms).collect();
            Some(PerformanceMetrics::from_durations(operation, &durations))
        } else {
            None
        }
    }

    /// Get metrics for all operations.
    pub async fn get_all_metrics(&self) -> HashMap<String, PerformanceMetrics> {
        let metrics = self.metrics.read().await;
        let mut result = HashMap::new();

        for (operation, operation_metrics) in metrics.iter() {
            let durations: Vec<u64> = operation_metrics.iter().map(|m| m.duration_ms).collect();
            let perf_metrics = PerformanceMetrics::from_durations(operation, &durations);
            result.insert(operation.clone(), perf_metrics);
        }

        result
    }

    /// Get the top slowest operations.
    pub async fn get_slowest_operations(&self, limit: usize) -> Vec<PerformanceMetrics> {
        let all_metrics = self.get_all_metrics().await;
        let mut metrics_vec: Vec<PerformanceMetrics> = all_metrics.into_values().collect();

        metrics_vec.sort_by(|a, b| {
            b.avg_duration_ms
                .partial_cmp(&a.avg_duration_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        metrics_vec.truncate(limit);

        metrics_vec
    }

    /// Get operations with high variance (inconsistent performance).
    pub async fn get_high_variance_operations(&self, limit: usize) -> Vec<(String, f64)> {
        let metrics = self.metrics.read().await;
        let mut variances = Vec::new();

        for (operation, operation_metrics) in metrics.iter() {
            if operation_metrics.len() < 2 {
                continue;
            }

            let durations: Vec<f64> = operation_metrics
                .iter()
                .map(|m| m.duration_ms as f64)
                .collect();
            let mean = durations.iter().sum::<f64>() / durations.len() as f64;
            let variance =
                durations.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / durations.len() as f64;

            variances.push((operation.clone(), variance));
        }

        variances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        variances.truncate(limit);

        variances
    }

    /// Clear all metrics.
    pub async fn clear(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();
    }

    /// Get the total number of recorded metrics.
    pub async fn total_metrics_count(&self) -> usize {
        let metrics = self.metrics.read().await;
        metrics.values().map(|v| v.len()).sum()
    }

    /// Get uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get recent metrics for an operation (last N measurements).
    pub async fn get_recent_metrics(&self, operation: &str, count: usize) -> Vec<Metric> {
        let metrics = self.metrics.read().await;
        if let Some(operation_metrics) = metrics.get(operation) {
            operation_metrics
                .iter()
                .rev()
                .take(count)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get metrics summary for all operations.
    pub async fn get_summary(&self) -> MetricsSummary {
        let all_metrics = self.get_all_metrics().await;
        let total_operations = all_metrics.len();
        let total_measurements = self.total_metrics_count().await;

        let avg_duration = if !all_metrics.is_empty() {
            all_metrics.values().map(|m| m.avg_duration_ms).sum::<f64>() / all_metrics.len() as f64
        } else {
            0.0
        };

        let slowest_operation = all_metrics
            .values()
            .max_by(|a, b| {
                a.avg_duration_ms
                    .partial_cmp(&b.avg_duration_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|m| m.operation.clone());

        let fastest_operation = all_metrics
            .values()
            .min_by(|a, b| {
                a.avg_duration_ms
                    .partial_cmp(&b.avg_duration_ms)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|m| m.operation.clone());

        MetricsSummary {
            total_operations,
            total_measurements,
            avg_duration_ms: avg_duration,
            uptime_seconds: self.uptime_seconds(),
            slowest_operation,
            fastest_operation,
        }
    }
}

/// Summary of all metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Total number of different operations
    pub total_operations: usize,
    /// Total number of measurements
    pub total_measurements: usize,
    /// Average duration across all operations
    pub avg_duration_ms: f64,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Name of the slowest operation
    pub slowest_operation: Option<String>,
    /// Name of the fastest operation
    pub fastest_operation: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new("test_op", Duration::from_millis(100));
        assert_eq!(metric.name, "test_op");
        assert_eq!(metric.duration_ms, 100);
        assert!(metric.timestamp > 0);
    }

    #[test]
    fn test_metric_with_metadata() {
        let metric = Metric::new("test_op", Duration::from_millis(100))
            .with_metadata("key1", "value1")
            .with_metadata("key2", "value2");

        assert_eq!(metric.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(metric.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_performance_metrics_calculation() {
        let durations = vec![100, 200, 150, 300, 250];
        let metrics = PerformanceMetrics::from_durations("test_op", &durations);

        assert_eq!(metrics.count, 5);
        assert_eq!(metrics.min_duration_ms, 100);
        assert_eq!(metrics.max_duration_ms, 300);
        assert_eq!(metrics.avg_duration_ms, 200.0);
        assert_eq!(metrics.total_duration_ms, 1000);
        assert!(metrics.ops_per_second > 0.0);
    }

    #[test]
    fn test_empty_performance_metrics() {
        let metrics = PerformanceMetrics::from_durations("test_op", &[]);
        assert_eq!(metrics.count, 0);
        assert_eq!(metrics.avg_duration_ms, 0.0);
        assert_eq!(metrics.ops_per_second, 0.0);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new(100);

        // Record some metrics
        collector
            .record_duration("operation1", Duration::from_millis(100))
            .await;
        collector
            .record_duration("operation1", Duration::from_millis(200))
            .await;
        collector
            .record_duration("operation2", Duration::from_millis(50))
            .await;

        // Get metrics for operation1
        let metrics = collector.get_metrics("operation1").await.unwrap();
        assert_eq!(metrics.count, 2);
        assert_eq!(metrics.avg_duration_ms, 150.0);

        // Get all metrics
        let all_metrics = collector.get_all_metrics().await;
        assert_eq!(all_metrics.len(), 2);

        // Get summary
        let summary = collector.get_summary().await;
        assert_eq!(summary.total_operations, 2);
        assert_eq!(summary.total_measurements, 3);
    }

    #[tokio::test]
    async fn test_metrics_collector_limit() {
        let collector = MetricsCollector::new(2);

        // Record more metrics than the limit
        for i in 0..5 {
            collector
                .record_duration("test_op", Duration::from_millis(i * 10))
                .await;
        }

        let recent_metrics = collector.get_recent_metrics("test_op", 10).await;
        assert_eq!(recent_metrics.len(), 2); // Should be limited to 2
    }

    #[tokio::test]
    async fn test_slowest_operations() {
        let collector = MetricsCollector::new(100);

        collector
            .record_duration("fast_op", Duration::from_millis(10))
            .await;
        collector
            .record_duration("slow_op", Duration::from_millis(1000))
            .await;
        collector
            .record_duration("medium_op", Duration::from_millis(100))
            .await;

        let slowest = collector.get_slowest_operations(2).await;
        assert_eq!(slowest.len(), 2);
        assert_eq!(slowest[0].operation, "slow_op");
        assert_eq!(slowest[1].operation, "medium_op");
    }
}
