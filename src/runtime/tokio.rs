//! Tokio runtime integration
//!
//! This module provides automatic tracking for Tokio tasks.

use crate::inspector::Inspector;
use crate::instrument::{clear_current_task_id, set_current_task_id};
use crate::task::TaskId;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

/// Spawn a task with automatic tracking
///
/// This is a drop-in replacement for `tokio::spawn()` that automatically
/// tracks the spawned task.
///
/// # Examples
///
/// ```rust,ignore
/// use async_inspect::runtime::tokio::spawn_tracked;
///
/// spawn_tracked("background_task", async {
///     // Your code here - automatically tracked!
///     println!("Task running");
/// });
/// ```
pub fn spawn_tracked<F, T>(name: T, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
    T: Into<String>,
{
    use crate::instrument::current_task_id;

    let task_name = name.into();

    // Check if there's a parent task
    let task_id = if let Some(parent_id) = current_task_id() {
        Inspector::global().register_child_task(task_name, parent_id)
    } else {
        Inspector::global().register_task(task_name)
    };

    tokio::spawn(async move {
        // Set task context for this task
        set_current_task_id(task_id);

        // Wrap execution to track completion
        let result = future.await;

        // Mark as completed
        Inspector::global().task_completed(task_id);

        // Clear context
        clear_current_task_id();

        result
    })
}

/// A future wrapper that automatically tracks execution
///
/// This wrapper tracks polls, completion, and can be used with any future.
pub struct TrackedFuture<F> {
    future: F,
    task_id: TaskId,
    started: bool,
    poll_start: Option<Instant>,
}

impl<F> TrackedFuture<F> {
    /// Create a new tracked future
    pub fn new(future: F, name: String) -> Self {
        let task_id = Inspector::global().register_task(name);

        Self {
            future,
            task_id,
            started: false,
            poll_start: None,
        }
    }

    /// Get the task ID
    pub fn task_id(&self) -> TaskId {
        self.task_id
    }
}

impl<F: Future> Future for TrackedFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: We don't move the future
        let this = unsafe { self.get_unchecked_mut() };

        // Set task context
        set_current_task_id(this.task_id);

        // Record poll start
        if !this.started {
            this.started = true;
        }

        let poll_start = Instant::now();
        this.poll_start = Some(poll_start);

        Inspector::global().poll_started(this.task_id);

        // Poll the inner future
        // SAFETY: We're pinning the projection
        let result = unsafe { Pin::new_unchecked(&mut this.future).poll(cx) };

        // Record poll end
        let poll_duration = poll_start.elapsed();
        Inspector::global().poll_ended(this.task_id, poll_duration);

        match result {
            Poll::Ready(output) => {
                // Task completed
                Inspector::global().task_completed(this.task_id);
                clear_current_task_id();
                Poll::Ready(output)
            }
            Poll::Pending => {
                // Still pending
                Poll::Pending
            }
        }
    }
}

/// Extension trait for futures to enable `.inspect()` syntax
///
/// # Examples
///
/// ```rust,ignore
/// use async_inspect::runtime::tokio::InspectExt;
///
/// let result = fetch_data()
///     .inspect("fetch_data")
///     .await;
/// ```
pub trait InspectExt: Future + Sized {
    /// Wrap this future with automatic tracking
    fn inspect(self, name: impl Into<String>) -> TrackedFuture<Self> {
        TrackedFuture::new(self, name.into())
    }

    /// Spawn this future on Tokio with tracking
    fn spawn_tracked(self, name: impl Into<String>) -> tokio::task::JoinHandle<Self::Output>
    where
        Self: Send + 'static,
        Self::Output: Send + 'static,
    {
        spawn_tracked(name, self)
    }
}

// Implement for all futures
impl<F: Future> InspectExt for F {}

/// Spawn a local task with automatic tracking (for !Send futures)
///
/// This is similar to `spawn_tracked` but for `!Send` futures on a LocalSet.
///
/// # Examples
///
/// ```rust,ignore
/// use async_inspect::runtime::tokio::spawn_local_tracked;
///
/// tokio::task::LocalSet::new().run_until(async {
///     spawn_local_tracked("local_task", async {
///         // !Send future
///     });
/// }).await;
/// ```
#[cfg(feature = "tokio")]
pub fn spawn_local_tracked<F, T>(name: T, future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + 'static,
    F::Output: 'static,
    T: Into<String>,
{
    let task_name = name.into();
    let task_id = Inspector::global().register_task(task_name);

    tokio::task::spawn_local(async move {
        set_current_task_id(task_id);

        let result = future.await;

        Inspector::global().task_completed(task_id);
        clear_current_task_id();

        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_tracked() {
        let handle = spawn_tracked("test_spawn_tracked_task", async { 42 });

        let result = handle.await.unwrap();
        assert_eq!(result, 42);

        // Verify task was tracked
        let tasks = Inspector::global().get_all_tasks();
        assert!(tasks.iter().any(|t| t.name == "test_spawn_tracked_task"));
    }

    #[tokio::test]
    async fn test_inspect_ext() {
        async fn example_task() -> i32 {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            123
        }

        let result = example_task().inspect("test_inspect_ext_task").await;

        assert_eq!(result, 123);

        // Verify task was tracked
        let tasks = Inspector::global().get_all_tasks();
        assert!(tasks.iter().any(|t| t.name == "test_inspect_ext_task"));
    }

    #[tokio::test]
    async fn test_tracked_future() {
        let future = async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            "done"
        };

        let tracked = TrackedFuture::new(future, "test_tracked_future_task".to_string());
        let task_id = tracked.task_id();

        let result = tracked.await;
        assert_eq!(result, "done");

        let task = Inspector::global().get_task(task_id).unwrap();
        assert!(task.poll_count > 0);
    }

    #[tokio::test]
    async fn test_spawn_tracked_multiple() {
        let handles: Vec<_> = (0..5)
            .map(|i| spawn_tracked(format!("test_multi_task_{}", i), async move { i * 2 }))
            .collect();

        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert_eq!(result, i * 2);
        }

        // Verify all tasks were tracked
        let tasks = Inspector::global().get_all_tasks();
        for i in 0..5 {
            assert!(tasks
                .iter()
                .any(|t| t.name == format!("test_multi_task_{}", i)));
        }
    }
}
