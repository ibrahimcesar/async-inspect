//! Code instrumentation utilities
//!
//! This module provides macros and helpers for instrumenting async code.

use crate::inspector::Inspector;
use crate::task::TaskId;
use std::time::Instant;

/// Context for tracking async operations
pub struct InspectContext {
    /// Task ID being tracked
    pub task_id: TaskId,
    /// Start time of current operation
    pub start_time: Instant,
}

impl InspectContext {
    /// Create a new inspect context
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            start_time: Instant::now(),
        }
    }

    /// Get elapsed time since context creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Helper for tracking poll operations
pub struct PollGuard {
    task_id: TaskId,
    start: Instant,
}

impl PollGuard {
    /// Create a new poll guard
    pub fn new(task_id: TaskId) -> Self {
        Inspector::global().poll_started(task_id);
        Self {
            task_id,
            start: Instant::now(),
        }
    }
}

impl Drop for PollGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        Inspector::global().poll_ended(self.task_id, duration);
    }
}

/// Helper for tracking await operations
pub struct AwaitGuard {
    task_id: TaskId,
    await_point: String,
    start: Instant,
}

impl AwaitGuard {
    /// Create a new await guard
    pub fn new(task_id: TaskId, await_point: String) -> Self {
        Inspector::global().await_started(task_id, await_point.clone(), None);
        Self {
            task_id,
            await_point,
            start: Instant::now(),
        }
    }
}

impl Drop for AwaitGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        Inspector::global().await_ended(self.task_id, self.await_point.clone(), duration);
    }
}

/// Record an inspection point in async code
///
/// # Examples
///
/// ```ignore
/// use async_inspect::inspect_point;
///
/// async fn my_function() {
///     inspect_point!("start");
///
///     let data = fetch_data().await;
///
///     inspect_point!("data_fetched", format!("Got {} items", data.len()));
///
///     process(data).await;
///
///     inspect_point!("done");
/// }
/// ```
#[macro_export]
macro_rules! inspect_point {
    ($label:expr) => {{
        if let Some(task_id) = $crate::instrument::current_task_id() {
            $crate::inspector::Inspector::global().inspection_point(
                task_id,
                $label.to_string(),
                None,
            );
        }
    }};
    ($label:expr, $message:expr) => {{
        if let Some(task_id) = $crate::instrument::current_task_id() {
            $crate::inspector::Inspector::global().inspection_point(
                task_id,
                $label.to_string(),
                Some($message.to_string()),
            );
        }
    }};
}

/// Begin tracking an async task
///
/// # Examples
///
/// ```ignore
/// use async_inspect::inspect_task_start;
///
/// async fn my_task() {
///     let task_id = inspect_task_start!("my_task");
///
///     // Your async code here
///
///     // Task will be marked as completed when task_id is dropped
/// }
/// ```
#[macro_export]
macro_rules! inspect_task_start {
    ($name:expr) => {{
        let task_id = $crate::inspector::Inspector::global().register_task($name.to_string());
        $crate::instrument::set_current_task_id(task_id);
        task_id
    }};
}

/// Mark current task as completed
#[macro_export]
macro_rules! inspect_task_complete {
    ($task_id:expr) => {{
        $crate::inspector::Inspector::global().task_completed($task_id);
    }};
}

/// Mark current task as failed
#[macro_export]
macro_rules! inspect_task_failed {
    ($task_id:expr) => {{
        $crate::inspector::Inspector::global().task_failed($task_id, None);
    }};
    ($task_id:expr, $error:expr) => {{
        $crate::inspector::Inspector::global().task_failed($task_id, Some($error.to_string()));
    }};
}

// Thread-local storage for current task ID
thread_local! {
    static CURRENT_TASK_ID: std::cell::RefCell<Option<TaskId>> = std::cell::RefCell::new(None);
}

/// Get the current task ID
pub fn current_task_id() -> Option<TaskId> {
    CURRENT_TASK_ID.with(|id| *id.borrow())
}

/// Set the current task ID
pub fn set_current_task_id(task_id: TaskId) {
    CURRENT_TASK_ID.with(|id| *id.borrow_mut() = Some(task_id));
}

/// Clear the current task ID
pub fn clear_current_task_id() {
    CURRENT_TASK_ID.with(|id| *id.borrow_mut() = None);
}

/// RAII guard for task tracking
pub struct TaskGuard {
    task_id: TaskId,
}

impl TaskGuard {
    /// Create a new task guard
    pub fn new(name: String) -> Self {
        let task_id = Inspector::global().register_task(name);
        set_current_task_id(task_id);
        Self { task_id }
    }

    /// Get the task ID
    pub fn task_id(&self) -> TaskId {
        self.task_id
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        Inspector::global().task_completed(self.task_id);
        clear_current_task_id();
    }
}

/// Helper function for await point instrumentation
pub fn inspect_await_start(label: impl Into<String>, location: Option<String>) {
    if let Some(task_id) = current_task_id() {
        Inspector::global().add_event(
            task_id,
            crate::timeline::EventKind::AwaitStarted {
                await_point: label.into(),
                location,
            },
        );
    }
}

/// Helper function for await point completion
pub fn inspect_await_end(label: impl Into<String>) {
    if let Some(task_id) = current_task_id() {
        // Calculate duration would require storing start time
        // For now, just record completion
        Inspector::global().add_event(
            task_id,
            crate::timeline::EventKind::AwaitEnded {
                await_point: label.into(),
                duration: std::time::Duration::from_micros(0), // TODO: track actual duration
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_task_id() {
        let task_id = TaskId::new();
        set_current_task_id(task_id);
        assert_eq!(current_task_id(), Some(task_id));
        clear_current_task_id();
        assert_eq!(current_task_id(), None);
    }

    #[test]
    fn test_task_guard() {
        let guard = TaskGuard::new("test".to_string());
        let task_id = guard.task_id();
        assert_eq!(current_task_id(), Some(task_id));
        drop(guard);
        assert_eq!(current_task_id(), None);
    }
}
