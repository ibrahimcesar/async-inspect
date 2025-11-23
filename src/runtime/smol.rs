//! smol runtime integration
//!
//! This module provides integration with the smol runtime,
//! allowing automatic task tracking for smol tasks.

use crate::inspector::Inspector;
use crate::instrument::{clear_current_task_id, set_current_task_id};
use crate::task::TaskId;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

/// Spawn a tracked task on the smol runtime
///
/// This function spawns a task and automatically tracks it in the global Inspector.
///
/// # Examples
///
/// ```no_run
/// use async_inspect::runtime::smol::spawn_tracked;
///
/// smol::block_on(async {
///     let task = spawn_tracked("my_task", async {
///         // Your async code here
///         42
///     });
///
///     let result = task.await;
///     println!("Result: {}", result);
/// });
/// ```
pub fn spawn_tracked<F, T>(name: T, future: F) -> smol::Task<F::Output>
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

    smol::spawn(async move {
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

/// Extension trait for smol futures to add tracking
///
/// # Examples
///
/// ```no_run
/// use async_inspect::runtime::smol::InspectExt;
///
/// smol::block_on(async {
///     let result = async {
///         // Your async code
///         42
///     }
///     .inspect("my_operation")
///     .await;
/// });
/// ```
pub trait InspectExt: Future + Sized {
    /// Wrap this future with automatic tracking
    fn inspect(self, name: impl Into<String>) -> TrackedFuture<Self> {
        TrackedFuture::new(self, name.into())
    }

    /// Spawn this future on smol with tracking
    fn spawn_tracked(self, name: impl Into<String>) -> smol::Task<Self::Output>
    where
        Self: Send + 'static,
        Self::Output: Send + 'static,
    {
        spawn_tracked(name, self)
    }
}

// Implement for all futures
impl<F: Future> InspectExt for F {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_tracked() {
        smol::block_on(async {
            let task = spawn_tracked("test_task", async { 42 });
            let result = task.await;
            assert_eq!(result, 42);
        });
    }

    #[test]
    fn test_inspect_ext() {
        smol::block_on(async {
            let result = async { 42 }.inspect("test_operation").await;
            assert_eq!(result, 42);
        });
    }
}
