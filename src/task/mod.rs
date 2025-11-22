//! Task tracking and monitoring
//!
//! This module provides the core data structures for tracking async tasks,
//! including task IDs, states, and metadata.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(u64);

impl TaskId {
    /// Create a new unique task ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Create a TaskId from a raw u64 value (for testing/examples)
    pub fn from_u64(id: u64) -> Self {
        Self(id)
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Current state of a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Task has been spawned but not yet polled
    Pending,
    /// Task is currently being polled
    Running,
    /// Task is waiting on an async operation
    Blocked {
        /// Name of the await point
        await_point: String,
    },
    /// Task has completed successfully
    Completed,
    /// Task was cancelled or panicked
    Failed,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Running => write!(f, "RUNNING"),
            Self::Blocked { await_point } => write!(f, "BLOCKED({})", await_point),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Failed => write!(f, "FAILED"),
        }
    }
}

/// Information about a task
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Unique task identifier
    pub id: TaskId,

    /// Human-readable task name
    pub name: String,

    /// Current state of the task
    pub state: TaskState,

    /// When the task was created
    pub created_at: Instant,

    /// When the task last changed state
    pub last_updated: Instant,

    /// Number of times the task has been polled
    pub poll_count: u64,

    /// Total time spent in running state
    pub total_run_time: Duration,

    /// Parent task ID, if any
    pub parent: Option<TaskId>,

    /// Source location (file:line)
    pub location: Option<String>,
}

impl TaskInfo {
    /// Create a new task info
    pub fn new(name: String) -> Self {
        let now = Instant::now();
        Self {
            id: TaskId::new(),
            name,
            state: TaskState::Pending,
            created_at: now,
            last_updated: now,
            poll_count: 0,
            total_run_time: Duration::ZERO,
            parent: None,
            location: None,
        }
    }

    /// Update the task state
    pub fn update_state(&mut self, new_state: TaskState) {
        self.state = new_state;
        self.last_updated = Instant::now();
    }

    /// Record a poll
    pub fn record_poll(&mut self, duration: Duration) {
        self.poll_count += 1;
        self.total_run_time += duration;
        self.last_updated = Instant::now();
    }

    /// Get the age of the task
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get time since last update
    pub fn time_since_update(&self) -> Duration {
        self.last_updated.elapsed()
    }

    /// Set the parent task
    pub fn with_parent(mut self, parent: TaskId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Set the source location
    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }
}

impl fmt::Display for TaskInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task {} [{}]: {} (polls: {}, runtime: {:.2}s, age: {:.2}s)",
            self.id,
            self.name,
            self.state,
            self.poll_count,
            self.total_run_time.as_secs_f64(),
            self.age().as_secs_f64()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_uniqueness() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_info_creation() {
        let task = TaskInfo::new("test_task".to_string());
        assert_eq!(task.name, "test_task");
        assert_eq!(task.state, TaskState::Pending);
        assert_eq!(task.poll_count, 0);
    }

    #[test]
    fn test_task_state_update() {
        let mut task = TaskInfo::new("test".to_string());
        task.update_state(TaskState::Running);
        assert_eq!(task.state, TaskState::Running);
    }

    #[test]
    fn test_task_poll_recording() {
        let mut task = TaskInfo::new("test".to_string());
        task.record_poll(Duration::from_millis(100));
        assert_eq!(task.poll_count, 1);
        assert_eq!(task.total_run_time, Duration::from_millis(100));
    }
}
