//! Chrome Trace Event Format exporter
//!
//! Exports async-inspect data to Chrome Trace Event Format for viewing in <chrome://tracing>
//! or compatible tools like Perfetto UI.
//!
//! Format specification: <https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU>/

use crate::inspector::Inspector;
use crate::timeline::EventKind;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Instant;

/// Chrome Trace Event
#[derive(Debug, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Event name
    pub name: String,

    /// Event category (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cat: Option<String>,

    /// Event type: B (begin), E (end), X (complete), i (instant), M (metadata)
    pub ph: String,

    /// Timestamp in microseconds
    pub ts: u64,

    /// Duration in microseconds (for 'X' events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dur: Option<u64>,

    /// Process ID
    pub pid: u32,

    /// Thread ID
    pub tid: u64,

    /// Additional arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
}

impl TraceEvent {
    /// Create a complete event (X) with duration
    #[must_use]
    pub fn complete(
        name: String,
        cat: &str,
        ts_us: u64,
        dur_us: u64,
        tid: u64,
        args: Option<serde_json::Value>,
    ) -> Self {
        Self {
            name,
            cat: Some(cat.to_string()),
            ph: "X".to_string(),
            ts: ts_us,
            dur: Some(dur_us),
            pid: std::process::id(),
            tid,
            args,
        }
    }

    /// Create an instant event (i)
    #[must_use]
    pub fn instant(
        name: String,
        cat: &str,
        ts_us: u64,
        tid: u64,
        args: Option<serde_json::Value>,
    ) -> Self {
        Self {
            name,
            cat: Some(cat.to_string()),
            ph: "i".to_string(),
            ts: ts_us,
            dur: None,
            pid: std::process::id(),
            tid,
            args,
        }
    }

    /// Create a metadata event (M) for thread name
    #[must_use]
    pub fn thread_name(tid: u64, name: String) -> Self {
        Self {
            name: "thread_name".to_string(),
            cat: None,
            ph: "M".to_string(),
            ts: 0,
            dur: None,
            pid: std::process::id(),
            tid,
            args: Some(serde_json::json!({ "name": name })),
        }
    }

    /// Create a metadata event (M) for process name
    #[must_use]
    pub fn process_name(name: String) -> Self {
        Self {
            name: "process_name".to_string(),
            cat: None,
            ph: "M".to_string(),
            ts: 0,
            dur: None,
            pid: std::process::id(),
            tid: 0,
            args: Some(serde_json::json!({ "name": name })),
        }
    }
}

/// Chrome Trace Event Format document
#[derive(Debug, Serialize, Deserialize)]
pub struct TraceDocument {
    /// Display time unit (default: "ms")
    #[serde(rename = "displayTimeUnit")]
    pub display_time_unit: String,

    /// Array of trace events
    #[serde(rename = "traceEvents")]
    pub trace_events: Vec<TraceEvent>,

    /// Metadata about the trace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Default for TraceDocument {
    fn default() -> Self {
        Self {
            display_time_unit: "ms".to_string(),
            trace_events: Vec::new(),
            metadata: None,
        }
    }
}

/// Chrome Trace Event Format exporter
pub struct ChromeTraceExporter;

impl ChromeTraceExporter {
    /// Export to Chrome Trace Event Format JSON file
    pub fn export_to_file<P: AsRef<Path>>(inspector: &Inspector, path: P) -> io::Result<()> {
        let document = Self::prepare_trace_document(inspector);
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &document)?;
        Ok(())
    }

    /// Export to Chrome Trace Event Format JSON string
    pub fn export_to_string(inspector: &Inspector) -> serde_json::Result<String> {
        let document = Self::prepare_trace_document(inspector);
        serde_json::to_string_pretty(&document)
    }

    fn prepare_trace_document(inspector: &Inspector) -> TraceDocument {
        let mut document = TraceDocument::default();

        // Add process name metadata
        document
            .trace_events
            .push(TraceEvent::process_name("async-inspect".to_string()));

        // Get timeline baseline (earliest event timestamp)
        let events = inspector.get_events();
        let baseline = events.first().map_or_else(Instant::now, |e| e.timestamp);

        // Track task names for thread metadata
        let mut task_names = std::collections::HashMap::new();

        // Convert events to trace events
        for event in events {
            let task_id = event.task_id.as_u64();
            let ts_us = event
                .timestamp
                .saturating_duration_since(baseline)
                .as_micros() as u64;

            match &event.kind {
                EventKind::TaskSpawned {
                    name,
                    parent,
                    location,
                } => {
                    // Store task name for metadata
                    task_names.insert(task_id, name.clone());

                    // Add thread name metadata
                    document
                        .trace_events
                        .push(TraceEvent::thread_name(task_id, name.clone()));

                    // Add instant event for task spawn
                    document.trace_events.push(TraceEvent::instant(
                        format!("spawn: {name}"),
                        "task",
                        ts_us,
                        task_id,
                        Some(serde_json::json!({
                            "parent": parent.map(|p| p.as_u64()),
                            "location": location,
                        })),
                    ));
                }

                EventKind::PollStarted => {
                    // We'll use PollEnded to create complete events
                }

                EventKind::PollEnded { duration } => {
                    let dur_us = duration.as_micros() as u64;
                    let start_ts = ts_us.saturating_sub(dur_us);

                    document.trace_events.push(TraceEvent::complete(
                        "poll".to_string(),
                        "runtime",
                        start_ts,
                        dur_us,
                        task_id,
                        None,
                    ));
                }

                EventKind::AwaitStarted {
                    await_point: _,
                    location: _,
                } => {
                    // We'll use AwaitEnded to create complete events
                    // Store await point name for later
                }

                EventKind::AwaitEnded {
                    await_point,
                    duration,
                } => {
                    let dur_us = duration.as_micros() as u64;
                    let start_ts = ts_us.saturating_sub(dur_us);

                    document.trace_events.push(TraceEvent::complete(
                        await_point.clone(),
                        "await",
                        start_ts,
                        dur_us,
                        task_id,
                        Some(serde_json::json!({
                            "await_point": await_point,
                        })),
                    ));
                }

                EventKind::TaskCompleted { duration } => {
                    let dur_us = duration.as_micros() as u64;
                    let start_ts = ts_us.saturating_sub(dur_us);

                    let task_name = task_names
                        .get(&task_id)
                        .cloned()
                        .unwrap_or_else(|| format!("task_{task_id}"));

                    document.trace_events.push(TraceEvent::complete(
                        task_name,
                        "task",
                        start_ts,
                        dur_us,
                        task_id,
                        Some(serde_json::json!({
                            "status": "completed",
                        })),
                    ));
                }

                EventKind::TaskFailed { error } => {
                    document.trace_events.push(TraceEvent::instant(
                        "task_failed".to_string(),
                        "task",
                        ts_us,
                        task_id,
                        Some(serde_json::json!({
                            "error": error,
                        })),
                    ));
                }

                EventKind::InspectionPoint { label, message } => {
                    document.trace_events.push(TraceEvent::instant(
                        label.clone(),
                        "inspection",
                        ts_us,
                        task_id,
                        Some(serde_json::json!({
                            "message": message,
                        })),
                    ));
                }

                EventKind::StateChanged {
                    old_state,
                    new_state,
                } => {
                    document.trace_events.push(TraceEvent::instant(
                        "state_change".to_string(),
                        "task",
                        ts_us,
                        task_id,
                        Some(serde_json::json!({
                            "old_state": format!("{:?}", old_state),
                            "new_state": format!("{:?}", new_state),
                        })),
                    ));
                }
            }
        }

        // Add metadata
        let stats = inspector.stats();
        document.metadata = Some(serde_json::json!({
            "async-inspect-version": env!("CARGO_PKG_VERSION"),
            "total_tasks": stats.total_tasks,
            "total_events": stats.total_events,
            "duration_ms": stats.timeline_duration.as_secs_f64() * 1000.0,
        }));

        document
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_event_complete() {
        let event = TraceEvent::complete("test_task".to_string(), "task", 1000, 500, 42, None);

        assert_eq!(event.name, "test_task");
        assert_eq!(event.ph, "X");
        assert_eq!(event.ts, 1000);
        assert_eq!(event.dur, Some(500));
        assert_eq!(event.tid, 42);
    }

    #[test]
    fn test_trace_event_instant() {
        let event = TraceEvent::instant(
            "spawn".to_string(),
            "task",
            2000,
            1,
            Some(serde_json::json!({"parent": 0})),
        );

        assert_eq!(event.name, "spawn");
        assert_eq!(event.ph, "i");
        assert_eq!(event.ts, 2000);
        assert_eq!(event.dur, None);
    }

    #[test]
    fn test_trace_event_metadata() {
        let event = TraceEvent::thread_name(42, "worker-1".to_string());

        assert_eq!(event.name, "thread_name");
        assert_eq!(event.ph, "M");
        assert_eq!(event.tid, 42);
    }
}
