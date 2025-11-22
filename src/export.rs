//! Export functionality for various formats
//!
//! This module provides exporters for task data in industry-standard formats
//! like JSON, CSV, and others.

use crate::inspector::Inspector;
use crate::task::TaskInfo;
use crate::timeline::{Event, EventKind};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Serializable task data
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportTask {
    /// Unique task identifier
    pub id: u64,
    /// Task name (function name)
    pub name: String,
    /// Current task state (Running, Blocked, etc.)
    pub state: String,
    /// Task creation timestamp in milliseconds
    pub created_at_ms: u128,
    /// Total duration in milliseconds
    pub duration_ms: f64,
    /// Number of times the task was polled
    pub poll_count: u64,
    /// Total time spent running (not waiting) in milliseconds
    pub run_time_ms: f64,
    /// Parent task ID if this is a spawned task
    pub parent_id: Option<u64>,
}

impl From<&TaskInfo> for ExportTask {
    fn from(task: &TaskInfo) -> Self {
        Self {
            id: task.id.as_u64(),
            name: task.name.clone(),
            state: format!("{:?}", task.state),
            created_at_ms: task.created_at.elapsed().as_millis(),
            duration_ms: task.age().as_secs_f64() * 1000.0,
            poll_count: task.poll_count,
            run_time_ms: task.total_run_time.as_secs_f64() * 1000.0,
            parent_id: task.parent.map(|id| id.as_u64()),
        }
    }
}

/// Serializable event data
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportEvent {
    /// Unique event identifier
    pub event_id: u64,
    /// Associated task identifier
    pub task_id: u64,
    /// Event timestamp in milliseconds
    pub timestamp_ms: u128,
    /// Event kind (TaskSpawned, Poll, Wake, etc.)
    pub kind: String,
    /// Additional event details
    pub details: Option<String>,
}

impl From<&Event> for ExportEvent {
    fn from(event: &Event) -> Self {
        let (kind, details) = match &event.kind {
            EventKind::TaskSpawned {
                name,
                parent,
                location,
            } => (
                "TaskSpawned".to_string(),
                Some(format!(
                    "name={}, parent={:?}, location={:?}",
                    name, parent, location
                )),
            ),
            EventKind::PollStarted => ("PollStarted".to_string(), None),
            EventKind::PollEnded { duration } => (
                "PollEnded".to_string(),
                Some(format!("duration={}ms", duration.as_secs_f64() * 1000.0)),
            ),
            EventKind::AwaitStarted {
                await_point,
                location,
            } => (
                "AwaitStarted".to_string(),
                Some(format!("point={}, location={:?}", await_point, location)),
            ),
            EventKind::AwaitEnded {
                await_point,
                duration,
            } => (
                "AwaitEnded".to_string(),
                Some(format!(
                    "point={}, duration={}ms",
                    await_point,
                    duration.as_secs_f64() * 1000.0
                )),
            ),
            EventKind::TaskCompleted { duration } => (
                "TaskCompleted".to_string(),
                Some(format!("duration={}ms", duration.as_secs_f64() * 1000.0)),
            ),
            EventKind::TaskFailed { error } => (
                "TaskFailed".to_string(),
                error.as_ref().map(|e| format!("error={}", e)),
            ),
            EventKind::InspectionPoint { label, message } => (
                "InspectionPoint".to_string(),
                Some(format!("label={}, message={:?}", label, message)),
            ),
            EventKind::StateChanged {
                old_state,
                new_state,
            } => (
                "StateChanged".to_string(),
                Some(format!("old={:?}, new={:?}", old_state, new_state)),
            ),
        };

        Self {
            event_id: 0, // Event IDs are internal, use 0 for export
            task_id: event.task_id.as_u64(),
            timestamp_ms: event.timestamp.elapsed().as_millis(),
            kind,
            details,
        }
    }
}

/// Complete export data
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    /// List of all tasks
    pub tasks: Vec<ExportTask>,
    /// List of all events
    pub events: Vec<ExportEvent>,
    /// Export metadata
    pub metadata: ExportMetadata,
}

/// Export metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// async-inspect version
    pub version: String,
    /// Export timestamp
    pub timestamp: String,
    /// Total number of tasks
    pub total_tasks: usize,
    /// Total number of events
    pub total_events: usize,
    /// Total duration captured in milliseconds
    pub duration_ms: f64,
}

/// JSON exporter
pub struct JsonExporter;

impl JsonExporter {
    /// Export to JSON string
    pub fn export_to_string(inspector: &Inspector) -> serde_json::Result<String> {
        let data = Self::prepare_export_data(inspector);
        serde_json::to_string_pretty(&data)
    }

    /// Export to JSON file
    pub fn export_to_file<P: AsRef<Path>>(inspector: &Inspector, path: P) -> io::Result<()> {
        let data = Self::prepare_export_data(inspector);
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, &data)?;
        Ok(())
    }

    fn prepare_export_data(inspector: &Inspector) -> ExportData {
        let tasks: Vec<ExportTask> = inspector
            .get_all_tasks()
            .iter()
            .map(ExportTask::from)
            .collect();

        let events: Vec<ExportEvent> = inspector
            .get_events()
            .iter()
            .map(ExportEvent::from)
            .collect();

        let stats = inspector.stats();

        ExportData {
            tasks,
            events,
            metadata: ExportMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                total_tasks: stats.total_tasks,
                total_events: stats.total_events,
                duration_ms: stats.timeline_duration.as_secs_f64() * 1000.0,
            },
        }
    }
}

/// CSV exporter
pub struct CsvExporter;

impl CsvExporter {
    /// Export tasks to CSV file
    pub fn export_tasks_to_file<P: AsRef<Path>>(inspector: &Inspector, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;

        // Write header
        writeln!(
            file,
            "id,name,state,created_at_ms,duration_ms,poll_count,run_time_ms,parent_id"
        )?;

        // Write tasks
        for task in inspector.get_all_tasks() {
            let export_task = ExportTask::from(&task);
            writeln!(
                file,
                "{},{},{},{},{},{},{},{}",
                export_task.id,
                Self::escape_csv(&export_task.name),
                export_task.state,
                export_task.created_at_ms,
                export_task.duration_ms,
                export_task.poll_count,
                export_task.run_time_ms,
                export_task
                    .parent_id
                    .map_or("".to_string(), |id| id.to_string())
            )?;
        }

        Ok(())
    }

    /// Export events to CSV file
    pub fn export_events_to_file<P: AsRef<Path>>(inspector: &Inspector, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;

        // Write header
        writeln!(file, "event_id,task_id,timestamp_ms,kind,details")?;

        // Write events
        for event in inspector.get_events() {
            let export_event = ExportEvent::from(&event);
            writeln!(
                file,
                "{},{},{},{},{}",
                export_event.event_id,
                export_event.task_id,
                export_event.timestamp_ms,
                export_event.kind,
                export_event.details.as_deref().unwrap_or("")
            )?;
        }

        Ok(())
    }

    fn escape_csv(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_escape() {
        assert_eq!(CsvExporter::escape_csv("simple"), "simple");
        assert_eq!(CsvExporter::escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(CsvExporter::escape_csv("with\"quote"), "\"with\"\"quote\"");
    }
}
