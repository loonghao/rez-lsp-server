//! Performance profiling utilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A profiling session that tracks nested operations.
#[derive(Debug)]
pub struct ProfilerGuard {
    name: String,
    start_time: Instant,
    profiler: Arc<Profiler>,
    parent_id: Option<u64>,
    session_id: u64,
}

impl ProfilerGuard {
    /// Create a new profiler guard.
    fn new(name: String, profiler: Arc<Profiler>, parent_id: Option<u64>) -> Self {
        let session_id = profiler.next_session_id();
        let start_time = Instant::now();

        Self {
            name,
            start_time,
            profiler,
            parent_id,
            session_id,
        }
    }

    /// Get the elapsed time since this guard was created.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get the session ID.
    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Create a child profiler guard.
    pub fn child(&self, name: impl Into<String>) -> ProfilerGuard {
        ProfilerGuard::new(name.into(), self.profiler.clone(), Some(self.session_id))
    }
}

impl Drop for ProfilerGuard {
    fn drop(&mut self) {
        let duration = self.elapsed();
        let profile_entry = ProfileEntry {
            name: self.name.clone(),
            duration_ms: duration.as_millis() as u64,
            start_time: self.start_time,
            parent_id: self.parent_id,
            session_id: self.session_id,
        };

        // Record the profile entry asynchronously
        let profiler = self.profiler.clone();
        tokio::spawn(async move {
            profiler.record_entry(profile_entry).await;
        });
    }
}

/// A single profiling entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEntry {
    /// Name of the operation
    pub name: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Start time of the operation (not serialized)
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
    /// Parent session ID (for nested operations)
    pub parent_id: Option<u64>,
    /// Unique session ID
    pub session_id: u64,
}

/// A profiling session containing multiple entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSession {
    /// Root operation name
    pub root_operation: String,
    /// All entries in this session
    pub entries: Vec<ProfileEntry>,
    /// Total duration of the session
    pub total_duration_ms: u64,
    /// Session start time
    pub start_timestamp: u64,
}

impl ProfileSession {
    /// Get the call tree for this session.
    pub fn call_tree(&self) -> CallTreeNode {
        let mut root_entries: Vec<&ProfileEntry> = self
            .entries
            .iter()
            .filter(|e| e.parent_id.is_none())
            .collect();

        if root_entries.is_empty() {
            return CallTreeNode {
                name: self.root_operation.clone(),
                duration_ms: self.total_duration_ms,
                children: Vec::new(),
                self_time_ms: self.total_duration_ms,
            };
        }

        // Sort by start time to get the actual root
        root_entries.sort_by_key(|e| e.start_time);
        let root = root_entries[0];

        self.build_call_tree_node(root)
    }

    fn build_call_tree_node(&self, entry: &ProfileEntry) -> CallTreeNode {
        let children: Vec<CallTreeNode> = self
            .entries
            .iter()
            .filter(|e| e.parent_id == Some(entry.session_id))
            .map(|child| self.build_call_tree_node(child))
            .collect();

        let children_time: u64 = children.iter().map(|c| c.duration_ms).sum();
        let self_time_ms = entry.duration_ms.saturating_sub(children_time);

        CallTreeNode {
            name: entry.name.clone(),
            duration_ms: entry.duration_ms,
            children,
            self_time_ms,
        }
    }
}

/// A node in the call tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallTreeNode {
    /// Operation name
    pub name: String,
    /// Total duration including children
    pub duration_ms: u64,
    /// Time spent in this operation only (excluding children)
    pub self_time_ms: u64,
    /// Child operations
    pub children: Vec<CallTreeNode>,
}

impl CallTreeNode {
    /// Get the percentage of time spent in this node relative to the total.
    pub fn percentage(&self, total_duration_ms: u64) -> f64 {
        if total_duration_ms > 0 {
            (self.duration_ms as f64 / total_duration_ms as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get the self-time percentage.
    pub fn self_percentage(&self, total_duration_ms: u64) -> f64 {
        if total_duration_ms > 0 {
            (self.self_time_ms as f64 / total_duration_ms as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Find the most expensive child operations.
    pub fn most_expensive_children(&self, limit: usize) -> Vec<&CallTreeNode> {
        let mut children: Vec<&CallTreeNode> = self.children.iter().collect();
        children.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
        children.truncate(limit);
        children
    }
}

/// Main profiler that manages profiling sessions.
#[derive(Debug)]
pub struct Profiler {
    /// Active profiling sessions
    sessions: Arc<RwLock<HashMap<u64, Vec<ProfileEntry>>>>,
    /// Completed sessions
    completed_sessions: Arc<RwLock<Vec<ProfileSession>>>,
    /// Next session ID
    next_id: Arc<RwLock<u64>>,
    /// Maximum number of completed sessions to keep
    max_sessions: usize,
    /// Whether profiling is enabled
    enabled: bool,
}

impl Profiler {
    /// Create a new profiler.
    pub fn new(max_sessions: usize, enabled: bool) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            completed_sessions: Arc::new(RwLock::new(Vec::new())),
            next_id: Arc::new(RwLock::new(1)),
            max_sessions,
            enabled,
        }
    }

    /// Start profiling an operation.
    pub fn profile(&self, name: impl Into<String>) -> Option<ProfilerGuard> {
        if !self.enabled {
            return None;
        }

        Some(ProfilerGuard::new(
            name.into(),
            Arc::new(self.clone()),
            None,
        ))
    }

    /// Get the next session ID.
    fn next_session_id(&self) -> u64 {
        let mut next_id = futures::executor::block_on(self.next_id.write());
        let id = *next_id;
        *next_id += 1;
        id
    }

    /// Record a profile entry.
    async fn record_entry(&self, entry: ProfileEntry) {
        if !self.enabled {
            return;
        }

        let mut sessions = self.sessions.write().await;
        let session_entries = sessions.entry(entry.session_id).or_insert_with(Vec::new);
        session_entries.push(entry);
    }

    /// Complete a profiling session.
    pub async fn complete_session(&self, root_session_id: u64, root_operation: String) {
        if !self.enabled {
            return;
        }

        let mut sessions = self.sessions.write().await;
        if let Some(entries) = sessions.remove(&root_session_id) {
            if !entries.is_empty() {
                let total_duration_ms = entries.iter().map(|e| e.duration_ms).max().unwrap_or(0);
                let start_timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let session = ProfileSession {
                    root_operation,
                    entries,
                    total_duration_ms,
                    start_timestamp,
                };

                let mut completed = self.completed_sessions.write().await;
                completed.push(session);

                // Keep only the most recent sessions
                while completed.len() > self.max_sessions {
                    completed.remove(0);
                }
            }
        }
    }

    /// Get all completed sessions.
    pub async fn get_completed_sessions(&self) -> Vec<ProfileSession> {
        let completed = self.completed_sessions.read().await;
        completed.clone()
    }

    /// Get the most recent session.
    pub async fn get_latest_session(&self) -> Option<ProfileSession> {
        let completed = self.completed_sessions.read().await;
        completed.last().cloned()
    }

    /// Get sessions by operation name.
    pub async fn get_sessions_by_operation(&self, operation: &str) -> Vec<ProfileSession> {
        let completed = self.completed_sessions.read().await;
        completed
            .iter()
            .filter(|s| s.root_operation == operation)
            .cloned()
            .collect()
    }

    /// Clear all sessions.
    pub async fn clear(&self) {
        let mut sessions = self.sessions.write().await;
        let mut completed = self.completed_sessions.write().await;
        sessions.clear();
        completed.clear();
    }

    /// Get profiling statistics.
    pub async fn get_stats(&self) -> ProfilerStats {
        let sessions = self.sessions.read().await;
        let completed = self.completed_sessions.read().await;

        let active_sessions = sessions.len();
        let completed_sessions = completed.len();
        let total_entries = sessions.values().map(|v| v.len()).sum::<usize>()
            + completed.iter().map(|s| s.entries.len()).sum::<usize>();

        let avg_session_duration = if !completed.is_empty() {
            completed.iter().map(|s| s.total_duration_ms).sum::<u64>() as f64
                / completed.len() as f64
        } else {
            0.0
        };

        ProfilerStats {
            active_sessions,
            completed_sessions,
            total_entries,
            avg_session_duration_ms: avg_session_duration,
            enabled: self.enabled,
        }
    }

    /// Enable or disable profiling.
    pub async fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Clone for Profiler {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            completed_sessions: self.completed_sessions.clone(),
            next_id: self.next_id.clone(),
            max_sessions: self.max_sessions,
            enabled: self.enabled,
        }
    }
}

/// Profiler statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilerStats {
    /// Number of active profiling sessions
    pub active_sessions: usize,
    /// Number of completed sessions
    pub completed_sessions: usize,
    /// Total number of profile entries
    pub total_entries: usize,
    /// Average session duration in milliseconds
    pub avg_session_duration_ms: f64,
    /// Whether profiling is enabled
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_profiler_basic() {
        let profiler = Profiler::new(10, true);

        {
            let _guard = profiler.profile("test_operation");
            sleep(Duration::from_millis(10)).await;
        }

        // Wait a bit for async recording to complete
        sleep(Duration::from_millis(50)).await;

        let stats = profiler.get_stats().await;
        assert!(stats.total_entries > 0);
    }

    #[tokio::test]
    async fn test_profiler_disabled() {
        let profiler = Profiler::new(10, false);

        let guard = profiler.profile("test_operation");
        assert!(guard.is_none());
    }

    #[tokio::test]
    async fn test_nested_profiling() {
        let profiler = Profiler::new(10, true);

        {
            let parent_guard = profiler.profile("parent_operation").unwrap();
            {
                let _child_guard = parent_guard.child("child_operation");
                sleep(Duration::from_millis(5)).await;
            }
            sleep(Duration::from_millis(5)).await;
        }

        // Wait a bit for async recording to complete
        sleep(Duration::from_millis(50)).await;

        let stats = profiler.get_stats().await;
        assert!(stats.total_entries >= 2);
    }

    #[test]
    fn test_call_tree_node() {
        let node = CallTreeNode {
            name: "test".to_string(),
            duration_ms: 100,
            self_time_ms: 60,
            children: vec![
                CallTreeNode {
                    name: "child1".to_string(),
                    duration_ms: 30,
                    self_time_ms: 30,
                    children: Vec::new(),
                },
                CallTreeNode {
                    name: "child2".to_string(),
                    duration_ms: 10,
                    self_time_ms: 10,
                    children: Vec::new(),
                },
            ],
        };

        assert_eq!(node.percentage(200), 50.0);
        assert_eq!(node.self_percentage(200), 30.0);

        let expensive_children = node.most_expensive_children(1);
        assert_eq!(expensive_children.len(), 1);
        assert_eq!(expensive_children[0].name, "child1");
    }

    #[tokio::test]
    async fn test_profiler_session_limit() {
        let profiler = Profiler::new(2, true);

        // Create more sessions than the limit
        for i in 0..5 {
            profiler
                .complete_session(i, format!("operation_{}", i))
                .await;
        }

        let sessions = profiler.get_completed_sessions().await;
        assert!(sessions.len() <= 2);
    }
}
