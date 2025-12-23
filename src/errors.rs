//! Error types with actionable messages
//!
//! This module provides detailed error types with helpful suggestions
//! for resolving common issues.

use std::fmt;
use thiserror::Error;

/// Main error type for async-inspect
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error with actionable advice
    #[error("{0}")]
    Config(#[from] ConfigError),

    /// Task tracking error
    #[error("{0}")]
    Task(#[from] TaskError),

    /// Inspection error
    #[error("Inspection error: {0}")]
    Inspection(String),

    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Configuration errors with helpful suggestions
#[derive(Debug)]
pub struct ConfigError {
    kind: ConfigErrorKind,
    suggestion: &'static str,
}

#[derive(Debug)]
enum ConfigErrorKind {
    InvalidSamplingRate(usize),
    InvalidRingBufferSize(usize),
    InvalidAdaptiveBounds { min: usize, max: usize },
    InvalidOverheadTarget(f64),
    FeatureNotEnabled(&'static str),
    ConflictingSettings { setting1: &'static str, setting2: &'static str },
}

impl ConfigError {
    /// Create an error for invalid sampling rate
    pub fn invalid_sampling_rate(rate: usize) -> Self {
        Self {
            kind: ConfigErrorKind::InvalidSamplingRate(rate),
            suggestion: "Sampling rate must be >= 1. Use 1 to track all tasks, \
                         or a higher number like 10 or 100 to reduce overhead.",
        }
    }

    /// Create an error for invalid ring buffer size
    pub fn invalid_ring_buffer_size(size: usize) -> Self {
        Self {
            kind: ConfigErrorKind::InvalidRingBufferSize(size),
            suggestion: "Ring buffer size must be > 0. Recommended sizes:\n  \
                         - Development: 10,000 - 100,000\n  \
                         - Production: 1,000 - 10,000\n  \
                         Use Timeline::with_ring_buffer(10_000) for typical production use.",
        }
    }

    /// Create an error for invalid adaptive sampling bounds
    pub fn invalid_adaptive_bounds(min: usize, max: usize) -> Self {
        Self {
            kind: ConfigErrorKind::InvalidAdaptiveBounds { min, max },
            suggestion: "Adaptive sampling bounds must satisfy:\n  \
                         - min_rate >= 1\n  \
                         - max_rate >= min_rate\n  \
                         Example: config.set_adaptive_bounds(1, 1000)",
        }
    }

    /// Create an error for invalid overhead target
    pub fn invalid_overhead_target(target_ms: f64) -> Self {
        Self {
            kind: ConfigErrorKind::InvalidOverheadTarget(target_ms),
            suggestion: "Overhead target must be > 0. Recommended values:\n  \
                         - Low overhead: 0.5 - 1.0 ms\n  \
                         - Balanced: 1.0 - 5.0 ms\n  \
                         - More data: 5.0 - 10.0 ms",
        }
    }

    /// Create an error for missing feature
    pub fn feature_not_enabled(feature: &'static str) -> Self {
        Self {
            kind: ConfigErrorKind::FeatureNotEnabled(feature),
            suggestion: match feature {
                "tokio" => "Add to Cargo.toml:\n  \
                            async-inspect = { version = \"0.1\", features = [\"tokio\"] }",
                "dashboard" => "Add to Cargo.toml:\n  \
                                async-inspect = { version = \"0.1\", features = [\"dashboard\"] }",
                "lsp" => "Add to Cargo.toml:\n  \
                          async-inspect = { version = \"0.1\", features = [\"lsp\"] }",
                "cli" => "Add to Cargo.toml:\n  \
                          async-inspect = { version = \"0.1\", features = [\"cli\"] }",
                _ => "Check Cargo.toml and enable the required feature.",
            },
        }
    }

    /// Create an error for conflicting settings
    pub fn conflicting_settings(setting1: &'static str, setting2: &'static str) -> Self {
        Self {
            kind: ConfigErrorKind::ConflictingSettings { setting1, setting2 },
            suggestion: "These settings cannot be used together. Choose one or the other.",
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ConfigErrorKind::InvalidSamplingRate(rate) => {
                write!(f, "Invalid sampling rate: {}\n\nSuggestion: {}", rate, self.suggestion)
            }
            ConfigErrorKind::InvalidRingBufferSize(size) => {
                write!(f, "Invalid ring buffer size: {}\n\nSuggestion: {}", size, self.suggestion)
            }
            ConfigErrorKind::InvalidAdaptiveBounds { min, max } => {
                write!(f, "Invalid adaptive bounds: min={}, max={}\n\nSuggestion: {}", min, max, self.suggestion)
            }
            ConfigErrorKind::InvalidOverheadTarget(target) => {
                write!(f, "Invalid overhead target: {} ms\n\nSuggestion: {}", target, self.suggestion)
            }
            ConfigErrorKind::FeatureNotEnabled(feature) => {
                write!(f, "Feature '{}' is not enabled\n\nSuggestion: {}", feature, self.suggestion)
            }
            ConfigErrorKind::ConflictingSettings { setting1, setting2 } => {
                write!(f, "Conflicting settings: '{}' and '{}'\n\nSuggestion: {}", setting1, setting2, self.suggestion)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Task-related errors
#[derive(Debug)]
pub struct TaskError {
    kind: TaskErrorKind,
    suggestion: &'static str,
}

#[derive(Debug)]
enum TaskErrorKind {
    TaskNotFound(u64),
    TaskPoolFull { capacity: usize },
    InvalidTaskState { expected: &'static str, actual: &'static str },
    ParentNotFound(u64),
}

impl TaskError {
    /// Create an error for task not found
    pub fn task_not_found(task_id: u64) -> Self {
        Self {
            kind: TaskErrorKind::TaskNotFound(task_id),
            suggestion: "The task may have completed and been removed. Check:\n  \
                         - Task ID is correct\n  \
                         - max_tasks setting isn't too low\n  \
                         - Task hasn't been garbage collected",
        }
    }

    /// Create an error for task pool full
    pub fn task_pool_full(capacity: usize) -> Self {
        Self {
            kind: TaskErrorKind::TaskPoolFull { capacity },
            suggestion: "The task pool has reached capacity. Options:\n  \
                         - Increase max_tasks: config.set_max_tasks(10_000)\n  \
                         - Enable sampling: config.set_sampling_rate(10)\n  \
                         - Use adaptive sampling: config.enable_adaptive_sampling()",
        }
    }

    /// Create an error for invalid task state transition
    pub fn invalid_task_state(expected: &'static str, actual: &'static str) -> Self {
        Self {
            kind: TaskErrorKind::InvalidTaskState { expected, actual },
            suggestion: "This usually indicates a bug in instrumentation. Check that:\n  \
                         - Tasks are properly registered before use\n  \
                         - State transitions follow the expected flow\n  \
                         - No concurrent modifications to task state",
        }
    }

    /// Create an error for parent task not found
    pub fn parent_not_found(parent_id: u64) -> Self {
        Self {
            kind: TaskErrorKind::ParentNotFound(parent_id),
            suggestion: "Parent task was not found when creating child. This may happen if:\n  \
                         - Parent completed before child was registered\n  \
                         - Parent was removed due to max_tasks limit\n  \
                         - Sampling caused the parent to be skipped",
        }
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TaskErrorKind::TaskNotFound(id) => {
                write!(f, "Task not found: {}\n\nSuggestion: {}", id, self.suggestion)
            }
            TaskErrorKind::TaskPoolFull { capacity } => {
                write!(f, "Task pool full (capacity: {})\n\nSuggestion: {}", capacity, self.suggestion)
            }
            TaskErrorKind::InvalidTaskState { expected, actual } => {
                write!(f, "Invalid task state: expected '{}', got '{}'\n\nSuggestion: {}", expected, actual, self.suggestion)
            }
            TaskErrorKind::ParentNotFound(id) => {
                write!(f, "Parent task not found: {}\n\nSuggestion: {}", id, self.suggestion)
            }
        }
    }
}

impl std::error::Error for TaskError {}

/// Diagnostic messages for common issues
pub struct Diagnostics;

impl Diagnostics {
    /// Get a diagnostic message for high memory usage
    pub fn high_memory_usage(current_bytes: usize, threshold_bytes: usize) -> String {
        format!(
            "High memory usage detected: {} bytes (threshold: {} bytes)\n\n\
             Suggestions:\n  \
             - Use ring buffer: Timeline::with_ring_buffer(10_000)\n  \
             - Reduce max_events: config.set_max_events(5_000)\n  \
             - Enable sampling: config.set_sampling_rate(10)\n  \
             - Enable adaptive sampling: config.enable_adaptive_sampling()",
            current_bytes, threshold_bytes
        )
    }

    /// Get a diagnostic message for high overhead
    pub fn high_overhead(overhead_ms: f64, target_ms: f64) -> String {
        format!(
            "Instrumentation overhead is high: {:.2} ms (target: {:.2} ms)\n\n\
             Suggestions:\n  \
             - Increase sampling rate: config.set_sampling_rate(100)\n  \
             - Disable await tracking: config.set_track_awaits(false)\n  \
             - Use production mode: config.production_mode()\n  \
             - Enable adaptive sampling: config.enable_adaptive_sampling()",
            overhead_ms, target_ms
        )
    }

    /// Get a diagnostic message for potential deadlock
    pub fn potential_deadlock(tasks: &[String], locks: &[String]) -> String {
        let task_list = tasks.join(", ");
        let lock_list = locks.join(" -> ");
        format!(
            "Potential deadlock detected!\n\n\
             Tasks involved: {}\n\
             Lock cycle: {}\n\n\
             Suggestions:\n  \
             - Review lock acquisition order\n  \
             - Use try_lock() with timeout\n  \
             - Consider restructuring to avoid nested locks\n  \
             - Use async_inspect::sync::Mutex for automatic detection",
            task_list, lock_list
        )
    }

    /// Get a diagnostic message for slow task
    pub fn slow_task(task_name: &str, duration_ms: f64, threshold_ms: f64) -> String {
        format!(
            "Slow task detected: '{}' took {:.2} ms (threshold: {:.2} ms)\n\n\
             Suggestions:\n  \
             - Check for blocking operations in async code\n  \
             - Review await points for slow external calls\n  \
             - Consider splitting into smaller tasks\n  \
             - Use spawn_blocking() for CPU-intensive work",
            task_name, duration_ms, threshold_ms
        )
    }

    /// Get a diagnostic message for high lock contention
    pub fn high_lock_contention(lock_name: &str, contention_rate: f64) -> String {
        format!(
            "High lock contention on '{}': {:.1}% of acquisitions are contested\n\n\
             Suggestions:\n  \
             - Reduce lock hold time\n  \
             - Consider using RwLock if reads dominate\n  \
             - Shard the lock if possible\n  \
             - Use lock-free data structures",
            lock_name, contention_rate * 100.0
        )
    }

    /// Get a diagnostic message for channel backpressure
    pub fn channel_backpressure(channel_name: &str, pending: usize, capacity: usize) -> String {
        format!(
            "Channel backpressure on '{}': {}/{} messages pending\n\n\
             Suggestions:\n  \
             - Increase consumer throughput\n  \
             - Add more consumers\n  \
             - Increase channel capacity\n  \
             - Consider bounded backpressure strategy",
            channel_name, pending, capacity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::invalid_sampling_rate(0);
        let msg = err.to_string();
        assert!(msg.contains("Invalid sampling rate"));
        assert!(msg.contains("Suggestion"));
    }

    #[test]
    fn test_task_error_display() {
        let err = TaskError::task_not_found(42);
        let msg = err.to_string();
        assert!(msg.contains("Task not found: 42"));
        assert!(msg.contains("Suggestion"));
    }

    #[test]
    fn test_diagnostics_high_memory() {
        let msg = Diagnostics::high_memory_usage(100_000_000, 50_000_000);
        assert!(msg.contains("High memory usage"));
        assert!(msg.contains("ring buffer"));
    }

    #[test]
    fn test_diagnostics_potential_deadlock() {
        let tasks = vec!["task1".to_string(), "task2".to_string()];
        let locks = vec!["mutex_a".to_string(), "mutex_b".to_string()];
        let msg = Diagnostics::potential_deadlock(&tasks, &locks);
        assert!(msg.contains("deadlock"));
        assert!(msg.contains("task1, task2"));
    }

    #[test]
    fn test_feature_not_enabled_suggestions() {
        let err = ConfigError::feature_not_enabled("tokio");
        assert!(err.to_string().contains("Cargo.toml"));
        assert!(err.to_string().contains("tokio"));
    }
}
