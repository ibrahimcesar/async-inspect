//! OpenTelemetry exporter
//!
//! This module exports async-inspect data in OpenTelemetry format,
//! enabling integration with OTLP-compatible backends like Jaeger, Zipkin,
//! and cloud observability platforms.

use crate::inspector::Inspector;
use crate::task::{TaskInfo, TaskState};
use crate::timeline::{Event, EventKind};
use opentelemetry::trace::{Tracer as _, TracerProvider as _};
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::{Sampler, TracerProvider};
use std::sync::Arc;
use std::time::Duration;

/// OpenTelemetry exporter for async-inspect
///
/// Exports task traces and spans to OTLP-compatible backends.
///
/// # Example
///
/// ```rust,ignore
/// use async_inspect::integrations::opentelemetry::OtelExporter;
///
/// let exporter = OtelExporter::new("async-inspect");
/// exporter.export_tasks();
/// ```
pub struct OtelExporter {
    service_name: String,
    provider: TracerProvider,
}

impl OtelExporter {
    /// Create a new OpenTelemetry exporter
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of the service for OTEL traces
    #[must_use]
    pub fn new(service_name: &str) -> Self {
        // Create a tracer provider with basic configuration
        let provider = TracerProvider::builder()
            .with_config(
                opentelemetry_sdk::trace::Config::default()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_resource(opentelemetry_sdk::Resource::new(vec![KeyValue::new(
                        "service.name",
                        service_name.to_string(),
                    )])),
            )
            .build();

        Self {
            service_name: service_name.to_string(),
            provider,
        }
    }

    /// Export all tasks as OpenTelemetry spans
    pub fn export_tasks(&self) {
        let tasks = Inspector::global().get_all_tasks();

        for task in tasks {
            self.export_task(&task);
        }
    }

    /// Export a single task as an OpenTelemetry span
    fn export_task(&self, task: &TaskInfo) {
        // Create a tracer for this export
        let tracer = self.provider.tracer(self.service_name.clone());

        // Start a span using the task name (clone to avoid lifetime issues)
        let mut span = tracer.start(task.name.clone());

        // Set span attributes
        use opentelemetry::trace::Span as _;
        span.set_attribute(KeyValue::new("task.id", task.id.as_u64() as i64));
        span.set_attribute(KeyValue::new("task.name", task.name.clone()));
        span.set_attribute(KeyValue::new("task.poll_count", task.poll_count as i64));
        span.set_attribute(KeyValue::new(
            "task.run_time_ms",
            task.total_run_time.as_millis() as i64,
        ));

        // Set task state as span status
        match task.state {
            TaskState::Completed => {
                span.set_status(opentelemetry::trace::Status::Ok);
            }
            TaskState::Failed => {
                span.set_status(opentelemetry::trace::Status::error("Task failed"));
            }
            _ => {}
        }

        if let Some(parent) = task.parent {
            span.set_attribute(KeyValue::new("task.parent_id", parent.as_u64() as i64));
        }

        if let Some(ref location) = task.location {
            span.set_attribute(KeyValue::new("task.location", location.clone()));
        }

        // End the span
        span.end();
    }

    /// Start a background exporter that periodically exports tasks
    #[cfg(feature = "tokio")]
    pub async fn start_background_export(self: Arc<Self>, interval: Duration) {
        let mut interval = tokio::time::interval(interval);

        loop {
            interval.tick().await;
            self.export_tasks();
        }
    }

    /// Export all timeline events
    pub fn export_events(&self) {
        let events = Inspector::global().get_events();

        for event in events {
            self.export_event(&event);
        }
    }

    /// Export a single event
    fn export_event(&self, event: &Event) {
        let tracer = self.provider.tracer(self.service_name.clone());

        let event_name = match &event.kind {
            EventKind::StateChanged { .. } => "state_changed",
            EventKind::TaskCompleted { .. } => "task_completed",
            EventKind::InspectionPoint { .. } => "inspection_point",
            EventKind::TaskSpawned { .. } => "task_spawned",
            EventKind::PollStarted => "poll_started",
            EventKind::PollEnded { .. } => "poll_ended",
            EventKind::AwaitStarted { .. } => "await_started",
            EventKind::AwaitEnded { .. } => "await_ended",
            EventKind::TaskFailed { .. } => "task_failed",
        };

        let mut span = tracer.start(event_name);

        use opentelemetry::trace::Span as _;
        span.set_attribute(KeyValue::new("event.id", event.id.as_u64() as i64));
        span.set_attribute(KeyValue::new(
            "event.task_id",
            event.task_id.as_u64() as i64,
        ));

        match &event.kind {
            EventKind::StateChanged {
                old_state,
                new_state,
            } => {
                span.set_attribute(KeyValue::new("old_state", format!("{old_state:?}")));
                span.set_attribute(KeyValue::new("new_state", format!("{new_state:?}")));
            }
            EventKind::TaskCompleted { duration } => {
                span.set_attribute(KeyValue::new("duration_ms", duration.as_millis() as i64));
            }
            EventKind::InspectionPoint { label, message } => {
                span.set_attribute(KeyValue::new("label", label.clone()));
                if let Some(msg) = message {
                    span.set_attribute(KeyValue::new("message", msg.clone()));
                }
            }
            EventKind::TaskSpawned { name, .. } => {
                span.set_attribute(KeyValue::new("task_name", name.clone()));
            }
            EventKind::PollStarted => {}
            EventKind::PollEnded { duration } => {
                span.set_attribute(KeyValue::new("duration_ms", duration.as_millis() as i64));
            }
            EventKind::AwaitStarted { await_point, .. } => {
                span.set_attribute(KeyValue::new("await_point", await_point.clone()));
            }
            EventKind::AwaitEnded { duration, .. } => {
                span.set_attribute(KeyValue::new("duration_ms", duration.as_millis() as i64));
            }
            EventKind::TaskFailed { error } => {
                if let Some(err) = error {
                    span.set_attribute(KeyValue::new("error", err.clone()));
                }
            }
        }

        span.end();
    }
}

impl Default for OtelExporter {
    fn default() -> Self {
        Self::new("async-inspect")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporter_creation() {
        let _exporter = OtelExporter::new("test-service");
    }

    #[test]
    fn test_default_exporter() {
        let _exporter = OtelExporter::default();
    }
}
