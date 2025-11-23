//! Performance comparison between runs
//!
//! This module provides functionality to compare performance metrics across multiple runs,
//! detect regressions, and identify performance improvements.

use super::{DurationStats, LockContentionMetrics, TaskMetrics};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// A snapshot of performance metrics from a single run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Timestamp when the snapshot was taken
    pub timestamp: u64,

    /// Run identifier
    pub run_id: String,

    /// Overall task statistics
    pub task_stats: DurationStats,

    /// All task metrics
    pub task_metrics: Vec<TaskMetrics>,

    /// Lock contention metrics
    pub lock_metrics: Vec<LockContentionMetrics>,

    /// Total tasks executed
    pub total_tasks: usize,

    /// Average task duration
    pub avg_task_duration: Duration,

    /// Total execution time
    pub total_execution_time: Duration,
}

impl PerformanceSnapshot {
    /// Create a new performance snapshot
    #[must_use]
    pub fn new(run_id: String) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            run_id,
            task_stats: DurationStats {
                min: Duration::ZERO,
                max: Duration::ZERO,
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                std_dev: 0.0,
                count: 0,
            },
            task_metrics: Vec::new(),
            lock_metrics: Vec::new(),
            total_tasks: 0,
            avg_task_duration: Duration::ZERO,
            total_execution_time: Duration::ZERO,
        }
    }

    /// Save snapshot to file
    pub fn save_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load snapshot from file
    pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        let snapshot: Self = serde_json::from_str(&json)?;
        Ok(snapshot)
    }
}

/// Comparison result between two runs
#[derive(Debug, Clone)]
pub struct PerformanceComparison {
    /// Baseline snapshot
    pub baseline: PerformanceSnapshot,

    /// Current snapshot
    pub current: PerformanceSnapshot,

    /// Task duration changes
    pub task_duration_change: DurationChange,

    /// Lock contention changes
    pub lock_contention_changes: Vec<LockContentionChange>,

    /// Overall regression/improvement status
    pub status: ComparisonStatus,

    /// Detailed findings
    pub findings: Vec<Finding>,
}

/// Status of the comparison
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonStatus {
    /// Performance improved
    Improved,
    /// Performance degraded (regression)
    Regressed,
    /// Performance is similar
    Similar,
    /// Mixed results
    Mixed,
}

/// Change in duration metrics
#[derive(Debug, Clone)]
pub struct DurationChange {
    /// Mean duration change
    pub mean_change: f64,

    /// Median duration change
    pub median_change: f64,

    /// P95 duration change
    pub p95_change: f64,

    /// P99 duration change
    pub p99_change: f64,

    /// Whether this represents an improvement
    pub is_improvement: bool,
}

/// Change in lock contention
#[derive(Debug, Clone)]
pub struct LockContentionChange {
    /// Resource name
    pub resource_name: String,

    /// Baseline contention rate
    pub baseline_rate: f64,

    /// Current contention rate
    pub current_rate: f64,

    /// Change in contention rate
    pub rate_change: f64,

    /// Change in wait time
    pub wait_time_change: f64,

    /// Whether this represents an improvement
    pub is_improvement: bool,
}

/// A specific finding from the comparison
#[derive(Debug, Clone)]
pub struct Finding {
    /// Severity of the finding
    pub severity: FindingSeverity,

    /// Category of the finding
    pub category: FindingCategory,

    /// Description
    pub description: String,

    /// Impact percentage (positive = improvement, negative = regression)
    pub impact_percent: f64,
}

/// Severity of a finding
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingSeverity {
    /// Critical regression (>50% degradation)
    Critical,
    /// Major issue (20-50% degradation)
    Major,
    /// Minor issue (5-20% degradation)
    Minor,
    /// Informational
    Info,
}

/// Category of a finding
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingCategory {
    /// Task execution time
    TaskDuration,
    /// Lock contention
    LockContention,
    /// Overall throughput
    Throughput,
    /// Other
    Other,
}

impl PerformanceComparison {
    /// Compare two performance snapshots
    #[must_use]
    pub fn compare(baseline: PerformanceSnapshot, current: PerformanceSnapshot) -> Self {
        let task_duration_change = Self::compare_durations(&baseline.task_stats, &current.task_stats);
        let lock_contention_changes = Self::compare_locks(&baseline.lock_metrics, &current.lock_metrics);
        let findings = Self::generate_findings(&baseline, &current, &task_duration_change, &lock_contention_changes);
        let status = Self::determine_status(&findings);

        Self {
            baseline,
            current,
            task_duration_change,
            lock_contention_changes,
            status,
            findings,
        }
    }

    fn compare_durations(baseline: &DurationStats, current: &DurationStats) -> DurationChange {
        let mean_change = Self::calculate_change_percent(
            baseline.mean.as_secs_f64(),
            current.mean.as_secs_f64(),
        );

        let median_change = Self::calculate_change_percent(
            baseline.median.as_secs_f64(),
            current.median.as_secs_f64(),
        );

        let p95_change = Self::calculate_change_percent(
            baseline.p95.as_secs_f64(),
            current.p95.as_secs_f64(),
        );

        let p99_change = Self::calculate_change_percent(
            baseline.p99.as_secs_f64(),
            current.p99.as_secs_f64(),
        );

        let is_improvement = mean_change < 0.0 && median_change < 0.0;

        DurationChange {
            mean_change,
            median_change,
            p95_change,
            p99_change,
            is_improvement,
        }
    }

    fn compare_locks(baseline: &[LockContentionMetrics], current: &[LockContentionMetrics]) -> Vec<LockContentionChange> {
        let mut changes = Vec::new();
        let baseline_map: HashMap<_, _> = baseline.iter().map(|m| (&m.name, m)).collect();

        for current_metric in current {
            if let Some(baseline_metric) = baseline_map.get(&current_metric.name) {
                let rate_change = Self::calculate_change_percent(
                    baseline_metric.contention_rate,
                    current_metric.contention_rate,
                );

                let wait_time_change = Self::calculate_change_percent(
                    baseline_metric.avg_wait_time.as_secs_f64(),
                    current_metric.avg_wait_time.as_secs_f64(),
                );

                changes.push(LockContentionChange {
                    resource_name: current_metric.name.clone(),
                    baseline_rate: baseline_metric.contention_rate,
                    current_rate: current_metric.contention_rate,
                    rate_change,
                    wait_time_change,
                    is_improvement: rate_change < 0.0 && wait_time_change < 0.0,
                });
            }
        }

        changes
    }

    fn generate_findings(
        baseline: &PerformanceSnapshot,
        current: &PerformanceSnapshot,
        duration_change: &DurationChange,
        lock_changes: &[LockContentionChange],
    ) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Check task duration changes
        if duration_change.mean_change.abs() > 5.0 {
            let severity = if duration_change.mean_change.abs() > 50.0 {
                FindingSeverity::Critical
            } else if duration_change.mean_change.abs() > 20.0 {
                FindingSeverity::Major
            } else {
                FindingSeverity::Minor
            };

            findings.push(Finding {
                severity,
                category: FindingCategory::TaskDuration,
                description: format!(
                    "Mean task duration changed by {:.1}%",
                    duration_change.mean_change
                ),
                impact_percent: -duration_change.mean_change, // Negative change is improvement
            });
        }

        // Check lock contention changes
        for lock_change in lock_changes {
            if lock_change.rate_change.abs() > 10.0 {
                let severity = if lock_change.rate_change.abs() > 50.0 {
                    FindingSeverity::Major
                } else {
                    FindingSeverity::Minor
                };

                findings.push(Finding {
                    severity,
                    category: FindingCategory::LockContention,
                    description: format!(
                        "Lock '{}' contention changed by {:.1}%",
                        lock_change.resource_name, lock_change.rate_change
                    ),
                    impact_percent: -lock_change.rate_change,
                });
            }
        }

        // Check throughput
        if baseline.total_tasks > 0 && current.total_tasks > 0 {
            let throughput_change = Self::calculate_change_percent(
                baseline.total_tasks as f64,
                current.total_tasks as f64,
            );

            if throughput_change.abs() > 5.0 {
                findings.push(Finding {
                    severity: FindingSeverity::Info,
                    category: FindingCategory::Throughput,
                    description: format!("Throughput changed by {:.1}%", throughput_change),
                    impact_percent: throughput_change,
                });
            }
        }

        findings
    }

    fn determine_status(findings: &[Finding]) -> ComparisonStatus {
        let regressions = findings.iter().filter(|f| f.impact_percent < 0.0).count();
        let improvements = findings.iter().filter(|f| f.impact_percent > 0.0).count();

        if regressions == 0 && improvements > 0 {
            ComparisonStatus::Improved
        } else if regressions > 0 && improvements == 0 {
            ComparisonStatus::Regressed
        } else if regressions == 0 && improvements == 0 {
            ComparisonStatus::Similar
        } else {
            ComparisonStatus::Mixed
        }
    }

    fn calculate_change_percent(baseline: f64, current: f64) -> f64 {
        if baseline == 0.0 {
            return 0.0;
        }
        ((current - baseline) / baseline) * 100.0
    }

    /// Check if there are any regressions
    #[must_use]
    pub fn has_regressions(&self) -> bool {
        self.findings.iter().any(|f| {
            matches!(f.severity, FindingSeverity::Critical | FindingSeverity::Major)
                && f.impact_percent < 0.0
        })
    }

    /// Get all regressions
    #[must_use]
    pub fn get_regressions(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.impact_percent < 0.0)
            .collect()
    }

    /// Get all improvements
    #[must_use]
    pub fn get_improvements(&self) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|f| f.impact_percent > 0.0)
            .collect()
    }

    /// Generate a summary report
    #[must_use]
    pub fn summary(&self) -> String {
        let mut report = String::from("Performance Comparison Summary\n");
        report.push_str("==============================\n\n");

        report.push_str(&format!("Status: {:?}\n\n", self.status));

        report.push_str("Task Duration Changes:\n");
        report.push_str(&format!("  Mean: {:.1}%\n", self.task_duration_change.mean_change));
        report.push_str(&format!("  Median: {:.1}%\n", self.task_duration_change.median_change));
        report.push_str(&format!("  P95: {:.1}%\n", self.task_duration_change.p95_change));
        report.push_str(&format!("  P99: {:.1}%\n\n", self.task_duration_change.p99_change));

        if !self.lock_contention_changes.is_empty() {
            report.push_str("Lock Contention Changes:\n");
            for change in &self.lock_contention_changes {
                report.push_str(&format!(
                    "  {}: {:.1}% (wait time: {:.1}%)\n",
                    change.resource_name, change.rate_change, change.wait_time_change
                ));
            }
            report.push('\n');
        }

        if !self.findings.is_empty() {
            report.push_str("Findings:\n");
            for finding in &self.findings {
                report.push_str(&format!(
                    "  [{:?}] {}\n",
                    finding.severity, finding.description
                ));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_change_percent() {
        assert_eq!(PerformanceComparison::calculate_change_percent(100.0, 110.0), 10.0);
        assert_eq!(PerformanceComparison::calculate_change_percent(100.0, 90.0), -10.0);
        assert_eq!(PerformanceComparison::calculate_change_percent(0.0, 10.0), 0.0);
    }

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = PerformanceSnapshot::new("test_run".to_string());
        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: PerformanceSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot.run_id, deserialized.run_id);
    }
}
