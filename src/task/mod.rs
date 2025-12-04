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
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Create a `TaskId` from a raw u64 value (for testing/examples)
    #[must_use]
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
            Self::Blocked { await_point } => write!(f, "BLOCKED({await_point})"),
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

    /// Source location (<file:line>)
    pub location: Option<String>,
}

impl TaskInfo {
    /// Create a new task info
    #[must_use]
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
    #[must_use]
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get time since last update
    #[must_use]
    pub fn time_since_update(&self) -> Duration {
        self.last_updated.elapsed()
    }

    /// Set the parent task
    #[must_use]
    pub fn with_parent(mut self, parent: TaskId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Set the source location
    #[must_use]
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

/// Filter for querying tasks
///
/// Use the builder pattern to construct filters:
///
/// ```rust
/// use async_inspect::task::{TaskFilter, TaskState};
/// use std::time::Duration;
///
/// let filter = TaskFilter::new()
///     .with_state(TaskState::Running)
///     .with_name_pattern("fetch")
///     .with_min_duration(Duration::from_secs(1));
/// ```
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    /// Filter by task state
    pub state: Option<TaskState>,
    /// Filter by name pattern (substring match)
    pub name_pattern: Option<String>,
    /// Filter by minimum age/duration
    pub min_duration: Option<Duration>,
    /// Filter by maximum age/duration
    pub max_duration: Option<Duration>,
    /// Filter by minimum poll count
    pub min_polls: Option<u64>,
    /// Filter by maximum poll count
    pub max_polls: Option<u64>,
    /// Filter by parent task
    pub parent: Option<TaskId>,
    /// Only show tasks without a parent (root tasks)
    pub root_only: bool,
}

impl TaskFilter {
    /// Create a new empty filter (matches all tasks)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by task state
    #[must_use]
    pub fn with_state(mut self, state: TaskState) -> Self {
        self.state = Some(state);
        self
    }

    /// Filter by name pattern (case-insensitive substring match)
    #[must_use]
    pub fn with_name_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.name_pattern = Some(pattern.into());
        self
    }

    /// Filter by minimum age/duration
    #[must_use]
    pub fn with_min_duration(mut self, duration: Duration) -> Self {
        self.min_duration = Some(duration);
        self
    }

    /// Filter by maximum age/duration
    #[must_use]
    pub fn with_max_duration(mut self, duration: Duration) -> Self {
        self.max_duration = Some(duration);
        self
    }

    /// Filter by minimum poll count
    #[must_use]
    pub fn with_min_polls(mut self, count: u64) -> Self {
        self.min_polls = Some(count);
        self
    }

    /// Filter by maximum poll count
    #[must_use]
    pub fn with_max_polls(mut self, count: u64) -> Self {
        self.max_polls = Some(count);
        self
    }

    /// Filter by parent task
    #[must_use]
    pub fn with_parent(mut self, parent: TaskId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Only show root tasks (no parent)
    #[must_use]
    pub fn root_only(mut self) -> Self {
        self.root_only = true;
        self
    }

    /// Check if a task matches this filter
    #[must_use]
    pub fn matches(&self, task: &TaskInfo) -> bool {
        // Check state
        if let Some(ref state) = self.state {
            if !self.state_matches(&task.state, state) {
                return false;
            }
        }

        // Check name pattern (case-insensitive)
        if let Some(ref pattern) = self.name_pattern {
            if !task.name.to_lowercase().contains(&pattern.to_lowercase()) {
                return false;
            }
        }

        // Check min duration
        if let Some(min) = self.min_duration {
            if task.age() < min {
                return false;
            }
        }

        // Check max duration
        if let Some(max) = self.max_duration {
            if task.age() > max {
                return false;
            }
        }

        // Check min polls
        if let Some(min) = self.min_polls {
            if task.poll_count < min {
                return false;
            }
        }

        // Check max polls
        if let Some(max) = self.max_polls {
            if task.poll_count > max {
                return false;
            }
        }

        // Check parent
        if let Some(parent) = self.parent {
            if task.parent != Some(parent) {
                return false;
            }
        }

        // Check root only
        if self.root_only && task.parent.is_some() {
            return false;
        }

        true
    }

    /// Check if task state matches the filter state
    fn state_matches(&self, task_state: &TaskState, filter_state: &TaskState) -> bool {
        match (task_state, filter_state) {
            (TaskState::Pending, TaskState::Pending) => true,
            (TaskState::Running, TaskState::Running) => true,
            (TaskState::Blocked { .. }, TaskState::Blocked { .. }) => true,
            (TaskState::Completed, TaskState::Completed) => true,
            (TaskState::Failed, TaskState::Failed) => true,
            _ => false,
        }
    }

    /// Filter a collection of tasks
    pub fn filter<'a>(&self, tasks: impl IntoIterator<Item = &'a TaskInfo>) -> Vec<&'a TaskInfo> {
        tasks.into_iter().filter(|t| self.matches(t)).collect()
    }

    /// Filter and clone a collection of tasks
    pub fn filter_cloned(&self, tasks: impl IntoIterator<Item = TaskInfo>) -> Vec<TaskInfo> {
        tasks.into_iter().filter(|t| self.matches(t)).collect()
    }
}

/// Sorting options for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TaskSortBy {
    /// Sort by task ID (creation order)
    #[default]
    Id,
    /// Sort by task name alphabetically
    Name,
    /// Sort by task age (oldest first)
    Age,
    /// Sort by poll count
    Polls,
    /// Sort by total run time
    RunTime,
    /// Sort by state
    State,
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    /// Ascending order
    #[default]
    Ascending,
    /// Descending order
    Descending,
}

/// Sort tasks by the given criteria
pub fn sort_tasks(tasks: &mut [TaskInfo], sort_by: TaskSortBy, direction: SortDirection) {
    tasks.sort_by(|a, b| {
        let cmp = match sort_by {
            TaskSortBy::Id => a.id.as_u64().cmp(&b.id.as_u64()),
            TaskSortBy::Name => a.name.cmp(&b.name),
            TaskSortBy::Age => a.created_at.cmp(&b.created_at),
            TaskSortBy::Polls => a.poll_count.cmp(&b.poll_count),
            TaskSortBy::RunTime => a.total_run_time.cmp(&b.total_run_time),
            TaskSortBy::State => state_order(&a.state).cmp(&state_order(&b.state)),
        };

        match direction {
            SortDirection::Ascending => cmp,
            SortDirection::Descending => cmp.reverse(),
        }
    });
}

/// Get sort order for task states
fn state_order(state: &TaskState) -> u8 {
    match state {
        TaskState::Running => 0,
        TaskState::Blocked { .. } => 1,
        TaskState::Pending => 2,
        TaskState::Completed => 3,
        TaskState::Failed => 4,
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
