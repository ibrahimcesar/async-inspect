//! Prometheus metrics exporter
//!
//! This module exports async-inspect metrics in Prometheus format,
//! allowing integration with Prometheus monitoring and Grafana dashboards.

use crate::inspector::Inspector;
use crate::task::TaskState;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Opts, Registry,
};
use std::sync::Arc;

/// Prometheus metrics exporter for async-inspect
///
/// Exports task metrics in Prometheus format for monitoring and alerting.
///
/// # Example
///
/// ```rust,ignore
/// use async_inspect::integrations::prometheus::PrometheusExporter;
///
/// let exporter = PrometheusExporter::new();
/// exporter.update(); // Update metrics from Inspector
///
/// // Get metrics in Prometheus format
/// let metrics = exporter.gather();
/// ```
pub struct PrometheusExporter {
    inspector: Arc<Inspector>,
    registry: Registry,

    // Task counters
    tasks_total: Counter,
    tasks_completed: Counter,
    tasks_failed: Counter,

    // Task state gauges
    tasks_by_state: GaugeVec,

    // Task duration histogram
    task_duration: HistogramVec,

    // Event counters
    events_total: Counter,

    // Poll counters
    poll_count: CounterVec,

    // Runtime gauges
    active_tasks: Gauge,
    blocked_tasks: Gauge,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new() -> prometheus::Result<Self> {
        Self::with_inspector(Inspector::global().clone())
    }

    /// Create an exporter with a specific inspector
    pub fn with_inspector(inspector: Arc<Inspector>) -> prometheus::Result<Self> {
        let registry = Registry::new();

        // Task counters
        let tasks_total = Counter::with_opts(Opts::new(
            "async_inspect_tasks_total",
            "Total number of tasks created",
        ))?;
        registry.register(Box::new(tasks_total.clone()))?;

        let tasks_completed = Counter::with_opts(Opts::new(
            "async_inspect_tasks_completed_total",
            "Total number of tasks completed",
        ))?;
        registry.register(Box::new(tasks_completed.clone()))?;

        let tasks_failed = Counter::with_opts(Opts::new(
            "async_inspect_tasks_failed_total",
            "Total number of tasks that failed",
        ))?;
        registry.register(Box::new(tasks_failed.clone()))?;

        // Task state gauges
        let tasks_by_state = GaugeVec::new(
            Opts::new("async_inspect_tasks_by_state", "Number of tasks by state"),
            &["state"],
        )?;
        registry.register(Box::new(tasks_by_state.clone()))?;

        // Task duration histogram
        let task_duration = HistogramVec::new(
            HistogramOpts::new(
                "async_inspect_task_duration_seconds",
                "Task execution duration in seconds",
            )
            .buckets(vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ]),
            &["task_name"],
        )?;
        registry.register(Box::new(task_duration.clone()))?;

        // Event counter
        let events_total = Counter::with_opts(Opts::new(
            "async_inspect_events_total",
            "Total number of events recorded",
        ))?;
        registry.register(Box::new(events_total.clone()))?;

        // Poll counter
        let poll_count = CounterVec::new(
            Opts::new(
                "async_inspect_task_polls_total",
                "Total number of task polls",
            ),
            &["task_name"],
        )?;
        registry.register(Box::new(poll_count.clone()))?;

        // Runtime gauges
        let active_tasks = Gauge::with_opts(Opts::new(
            "async_inspect_active_tasks",
            "Number of currently active tasks",
        ))?;
        registry.register(Box::new(active_tasks.clone()))?;

        let blocked_tasks = Gauge::with_opts(Opts::new(
            "async_inspect_blocked_tasks",
            "Number of currently blocked tasks",
        ))?;
        registry.register(Box::new(blocked_tasks.clone()))?;

        Ok(Self {
            inspector,
            registry,
            tasks_total,
            tasks_completed,
            tasks_failed,
            tasks_by_state,
            task_duration,
            events_total,
            poll_count,
            active_tasks,
            blocked_tasks,
        })
    }

    /// Update all metrics from the inspector
    pub fn update(&self) {
        let stats = self.inspector.stats();

        // Update counters (these are cumulative, so we need to set them carefully)
        // Note: Prometheus counters can only increase, so we track changes

        // Update state-based gauges
        self.tasks_by_state
            .with_label_values(&["running"])
            .set(stats.running_tasks as f64);
        self.tasks_by_state
            .with_label_values(&["completed"])
            .set(stats.completed_tasks as f64);
        self.tasks_by_state
            .with_label_values(&["failed"])
            .set(stats.failed_tasks as f64);
        self.tasks_by_state
            .with_label_values(&["blocked"])
            .set(stats.blocked_tasks as f64);

        // Update runtime gauges
        self.active_tasks.set(stats.running_tasks as f64);
        self.blocked_tasks.set(stats.blocked_tasks as f64);

        // Update task durations and polls
        for task in self.inspector.get_all_tasks() {
            // Update task duration histogram for completed tasks
            if matches!(task.state, TaskState::Completed | TaskState::Failed) {
                self.task_duration
                    .with_label_values(&[&task.name])
                    .observe(task.total_run_time.as_secs_f64());
            }

            // Update poll count
            self.poll_count
                .with_label_values(&[&task.name])
                .inc_by(task.poll_count as f64);
        }

        // Update event count
        self.events_total.inc_by(stats.total_events as f64);
    }

    /// Get the Prometheus registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Gather metrics in Prometheus text format
    pub fn gather(&self) -> String {
        use prometheus::Encoder;

        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();

        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        String::from_utf8(buffer).unwrap()
    }

    /// Start a background metrics updater that updates metrics periodically
    #[cfg(feature = "tokio")]
    pub fn start_background_updater(
        self: Arc<Self>,
        interval: std::time::Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                self.update();
            }
        })
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new().expect("Failed to create Prometheus exporter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporter_creation() {
        let exporter = PrometheusExporter::new().unwrap();
        exporter.update();
        let _metrics = exporter.gather();
    }
}
