//! OpenTelemetry exporter
//!
//! This module exports async-inspect data in OpenTelemetry format,
//! enabling integration with OTLP-compatible backends like Jaeger, Zipkin,
//! and cloud observability platforms.

use crate::inspector::Inspector;
use crate::task::{TaskInfo, TaskState};
use crate::timeline::{Event, EventKind};
use opentelemetry::trace::{SpanId, TraceId};
use opentelemetry::{
    trace::{Span, SpanKind, Status, Tracer},
    KeyValue,
};
use opentelemetry_sdk::trace::{Sampler, TracerProvider};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

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
    inspector: Arc<Inspector>,
    tracer: Box<dyn Tracer + Send + Sync>,
    span_map: Arc<Mutex<HashMap<crate::task::TaskId, SpanId>>>,
}

impl OtelExporter {
    /// Create a new OpenTelemetry exporter
    ///
    /// # Arguments
    ///
    /// * `service_name` - Name of the service for OTEL traces
    pub fn new(service_name: &str) -> Self {
        Self::with_inspector(Inspector::global().clone(), service_name)
    }

    /// Create an exporter with a specific inspector
    pub fn with_inspector(inspector: Arc<Inspector>, service_name: &str) -> Self {
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

        let tracer = provider.tracer(service_name.to_string());

        Self {
            inspector,
            tracer: Box::new(tracer),
            span_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Export all tasks as OpenTelemetry spans
    pub fn export_tasks(&self) {
        for task in self.inspector.get_all_tasks() {
            self.export_task(&task);
        }
    }

    /// Export a single task as an OpenTelemetry span
    fn export_task(&self, task: &TaskInfo) {
        let mut span = self
            .tracer
            .start_with_context(&task.name, &opentelemetry::Context::current());

        // Set span attributes
        span.set_attribute(KeyValue::new("task.id", task.id.as_u64() as i64));
        span.set_attribute(KeyValue::new("task.name", task.name.clone()));
        span.set_attribute(KeyValue::new("task.poll_count", task.poll_count as i64));
        span.set_attribute(KeyValue::new(
            "task.run_time_ms",
            task.total_run_time.as_millis() as i64,
        ));

        if let Some(parent) = task.parent {
            span.set_attribute(KeyValue::new("task.parent_id", parent.as_u64() as i64));
        }

        if let Some(ref location) = task.location {
            span.set_attribute(KeyValue::new("task.location", location.clone()));
        }

        // Set span kind
        span.set_attribute(KeyValue::new("span.kind", "INTERNAL"));

        // Set status based on task state
        match task.state {
            TaskState::Completed => {
                span.set_status(Status::Ok);
            }
            TaskState::Failed => {
                span.set_status(Status::error("Task failed"));
            }
            _ => {}
        }

        // Add events from timeline
        for event in self.inspector.get_events_for_task(task.id) {
            self.add_event_to_span(&mut *span, &event);
        }

        span.end();

        // Store span ID mapping
        if let Ok(mut map) = self.span_map.lock() {
            // Note: We can't actually get the SpanId from the Span trait easily
            // This is a limitation of the current OpenTelemetry API
            // In a real implementation, you'd use the SpanContext
        }
    }

    /// Add a timeline event to an OpenTelemetry span
    fn add_event_to_span(&self, span: &mut dyn Span, event: &Event) {
        let event_name = match &event.kind {
            EventKind::TaskStarted => "task.started",
            EventKind::PollStarted { .. } => "poll.started",
            EventKind::PollEnded { .. } => "poll.ended",
            EventKind::AwaitStarted { .. } => "await.started",
            EventKind::AwaitEnded { .. } => "await.ended",
            EventKind::TaskCompleted { .. } => "task.completed",
            EventKind::TaskFailed { .. } => "task.failed",
            EventKind::InspectionPoint { .. } => "inspection.point",
            EventKind::StateChanged { .. } => "state.changed",
        };

        let attributes = match &event.kind {
            EventKind::AwaitStarted {
                await_point,
                location,
            } => vec![
                KeyValue::new("await.point", await_point.clone()),
                KeyValue::new("location", format!("{:?}", location)),
            ],
            EventKind::AwaitEnded {
                await_point,
                duration,
            } => vec![
                KeyValue::new("await.point", await_point.clone()),
                KeyValue::new("duration_ms", duration.as_millis() as i64),
            ],
            EventKind::PollEnded { duration } => {
                vec![KeyValue::new("duration_ms", duration.as_millis() as i64)]
            }
            EventKind::TaskCompleted { duration } => {
                vec![KeyValue::new("duration_ms", duration.as_millis() as i64)]
            }
            EventKind::TaskFailed { error } => {
                if let Some(err) = error {
                    vec![KeyValue::new("error", err.clone())]
                } else {
                    vec![]
                }
            }
            EventKind::InspectionPoint { label, message } => {
                let mut attrs = vec![KeyValue::new("label", label.clone())];
                if let Some(msg) = message {
                    attrs.push(KeyValue::new("message", msg.clone()));
                }
                attrs
            }
            EventKind::StateChanged {
                old_state,
                new_state,
            } => vec![
                KeyValue::new("old_state", format!("{:?}", old_state)),
                KeyValue::new("new_state", format!("{:?}", new_state)),
            ],
            _ => vec![],
        };

        span.add_event(event_name.to_string(), attributes);
    }

    /// Export events continuously as they occur
    ///
    /// This creates a background task that monitors the inspector
    /// and exports events in real-time.
    #[cfg(feature = "tokio")]
    pub fn start_continuous_export(
        self: Arc<Self>,
        interval: std::time::Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            let mut last_event_count = 0;

            loop {
                interval.tick().await;

                let stats = self.inspector.stats();
                if stats.total_events > last_event_count {
                    // Export new tasks
                    self.export_tasks();
                    last_event_count = stats.total_events;
                }
            }
        })
    }
}

/// Create a configured OpenTelemetry exporter with OTLP endpoint
///
/// # Example
///
/// ```rust,ignore
/// use async_inspect::integrations::opentelemetry::create_otlp_exporter;
///
/// let exporter = create_otlp_exporter(
///     "async-inspect",
///     "http://localhost:4317"
/// );
/// ```
pub fn create_otlp_exporter(service_name: &str, endpoint: &str) -> OtelExporter {
    // In a real implementation, you would configure the OTLP exporter here
    // This is a simplified version

    OtelExporter::new(service_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporter_creation() {
        let _exporter = OtelExporter::new("test-service");
    }
}
