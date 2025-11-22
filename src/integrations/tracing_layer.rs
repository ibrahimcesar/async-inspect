//! Tracing subscriber layer integration
//!
//! This module provides a tracing-subscriber Layer that automatically
//! captures async task events and feeds them into async-inspect.

use crate::inspector::Inspector;
use crate::task::{TaskId, TaskState};
use crate::timeline::EventKind;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::span::{Attributes, Id};
use tracing::{Event as TracingEvent, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

/// Tracing layer that integrates with async-inspect
///
/// This layer automatically captures span enter/exit events and creates
/// corresponding async-inspect tasks and timeline events.
///
/// # Example
///
/// ```rust,ignore
/// use async_inspect::integrations::tracing_layer::AsyncInspectLayer;
/// use tracing_subscriber::prelude::*;
///
/// tracing_subscriber::registry()
///     .with(AsyncInspectLayer::new())
///     .init();
/// ```
pub struct AsyncInspectLayer {
    span_map: Arc<Mutex<HashMap<Id, TaskId>>>,
}

impl AsyncInspectLayer {
    /// Create a new tracing layer
    pub fn new() -> Self {
        Self {
            span_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for AsyncInspectLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for AsyncInspectLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        let metadata = attrs.metadata();
        let name = metadata.name();

        // Check if this is an async task span
        if metadata.is_span() && (name.starts_with("async") || name.contains("task")) {
            // Register the task with async-inspect
            let task_id = Inspector::global().register_task(name.to_string());

            // Map span ID to task ID
            if let Ok(mut map) = self.span_map.lock() {
                map.insert(id.clone(), task_id);
            }

            // Record task started event
            Inspector::global().add_event(
                task_id,
                EventKind::StateChanged {
                    old_state: TaskState::Pending,
                    new_state: TaskState::Pending,
                },
            );
        }
    }

    fn on_enter(&self, id: &Id, _ctx: Context<'_, S>) {
        if let Ok(map) = self.span_map.lock() {
            if let Some(&task_id) = map.get(id) {
                // Update task state to running
                let old_state = Inspector::global()
                    .get_task(task_id)
                    .map(|t| t.state)
                    .unwrap_or(TaskState::Pending);

                Inspector::global().update_task_state(task_id, TaskState::Running);

                // Record state change event
                Inspector::global().add_event(
                    task_id,
                    EventKind::StateChanged {
                        old_state,
                        new_state: TaskState::Running,
                    },
                );
            }
        }
    }

    fn on_exit(&self, id: &Id, _ctx: Context<'_, S>) {
        if let Ok(map) = self.span_map.lock() {
            if let Some(&task_id) = map.get(id) {
                // Task is yielding/awaiting
                let old_state = Inspector::global()
                    .get_task(task_id)
                    .map(|t| t.state)
                    .unwrap_or(TaskState::Running);

                // Don't change state if already completed/failed
                if !matches!(old_state, TaskState::Completed | TaskState::Failed) {
                    Inspector::global().update_task_state(task_id, TaskState::Pending);

                    Inspector::global().add_event(
                        task_id,
                        EventKind::StateChanged {
                            old_state,
                            new_state: TaskState::Pending,
                        },
                    );
                }
            }
        }
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        if let Ok(mut map) = self.span_map.lock() {
            if let Some(task_id) = map.remove(&id) {
                // Mark task as completed
                let duration = Inspector::global()
                    .get_task(task_id)
                    .map(|t| t.created_at.elapsed())
                    .unwrap_or_default();

                Inspector::global().update_task_state(task_id, TaskState::Completed);

                Inspector::global().add_event(task_id, EventKind::TaskCompleted { duration });
            }
        }
    }

    fn on_event(&self, event: &TracingEvent<'_>, _ctx: Context<'_, S>) {
        // Capture tracing events as inspection points
        let metadata = event.metadata();

        // Try to get the current span's task_id
        if let Some(id) = _ctx.current_span().id() {
            if let Ok(map) = self.span_map.lock() {
                if let Some(&task_id) = map.get(&id) {
                    Inspector::global().add_event(
                        task_id,
                        EventKind::InspectionPoint {
                            label: metadata.name().to_string(),
                            message: Some(format!("{:?}", event)),
                        },
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let _layer = AsyncInspectLayer::new();
    }
}
