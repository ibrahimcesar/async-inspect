//! Tracing subscriber layer integration
//!
//! This module provides a tracing-subscriber Layer that automatically
//! captures async task events and feeds them into async-inspect.

use crate::inspector::Inspector;
use crate::task::{TaskId, TaskInfo, TaskState};
use crate::timeline::{Event, EventKind};
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
    inspector: Arc<Inspector>,
    span_map: Arc<Mutex<HashMap<Id, TaskId>>>,
}

impl AsyncInspectLayer {
    /// Create a new tracing layer
    pub fn new() -> Self {
        Self {
            inspector: Inspector::global().clone(),
            span_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a layer with a specific inspector instance
    pub fn with_inspector(inspector: Arc<Inspector>) -> Self {
        Self {
            inspector,
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
        if metadata.is_span() && name.starts_with("async") || name.contains("task") {
            // Create a new task in async-inspect
            let task_info = TaskInfo::new(name.to_string());
            let task_id = task_info.id;

            // Register the task
            self.inspector.register_task(task_info);

            // Map span ID to task ID
            if let Ok(mut map) = self.span_map.lock() {
                map.insert(id.clone(), task_id);
            }

            // Record task started event
            self.inspector.record_event(Event {
                task_id,
                timestamp: Instant::now(),
                kind: EventKind::StateChanged {
                    old_state: TaskState::Pending,
                    new_state: TaskState::Pending,
                },
            });
        }
    }

    fn on_enter(&self, id: &Id, _ctx: Context<'_, S>) {
        if let Ok(map) = self.span_map.lock() {
            if let Some(&task_id) = map.get(id) {
                // Update task state to running
                if let Some(mut task) = self.inspector.get_task_mut(task_id) {
                    let old_state = task.state.clone();
                    task.update_state(TaskState::Running);

                    // Record state change event
                    self.inspector.record_event(Event {
                        task_id,
                        timestamp: Instant::now(),
                        kind: EventKind::StateChanged {
                            old_state,
                            new_state: TaskState::Running,
                        },
                    });
                }
            }
        }
    }

    fn on_exit(&self, id: &Id, _ctx: Context<'_, S>) {
        if let Ok(map) = self.span_map.lock() {
            if let Some(&task_id) = map.get(id) {
                // Task is yielding/awaiting
                if let Some(mut task) = self.inspector.get_task_mut(task_id) {
                    let old_state = task.state.clone();

                    // Don't change state if already completed/failed
                    if !matches!(old_state, TaskState::Completed | TaskState::Failed) {
                        task.update_state(TaskState::Pending);

                        self.inspector.record_event(Event {
                            task_id,
                            timestamp: Instant::now(),
                            kind: EventKind::StateChanged {
                                old_state,
                                new_state: TaskState::Pending,
                            },
                        });
                    }
                }
            }
        }
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        if let Ok(mut map) = self.span_map.lock() {
            if let Some(task_id) = map.remove(&id) {
                // Mark task as completed
                if let Some(mut task) = self.inspector.get_task_mut(task_id) {
                    let created_at = task.created_at;
                    task.update_state(TaskState::Completed);

                    self.inspector.record_event(Event {
                        task_id,
                        timestamp: Instant::now(),
                        kind: EventKind::TaskCompleted {
                            duration: created_at.elapsed(),
                        },
                    });
                }
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
                    self.inspector.record_event(Event {
                        task_id,
                        timestamp: Instant::now(),
                        kind: EventKind::InspectionPoint {
                            label: metadata.name().to_string(),
                            message: Some(format!("{:?}", event)),
                        },
                    });
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
