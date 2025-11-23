//! Performance profiling and analysis
//!
//! This module provides tools for analyzing async task performance,
//! identifying bottlenecks, and generating performance reports.

pub mod comparison;
pub mod reporter;

use crate::deadlock::ResourceId;
use crate::task::TaskId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub use reporter::PerformanceReporter;

/// Performance metrics for a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Task ID
    pub task_id: TaskId,

    /// Task name
    pub name: String,

    /// Total execution duration
    pub total_duration: Duration,

    /// Time spent in running state
    pub running_time: Duration,

    /// Time spent blocked
    pub blocked_time: Duration,

    /// Number of times the task was polled
    pub poll_count: u64,

    /// Number of await points
    pub await_count: u64,

    /// Durations of each await point
    pub await_durations: Vec<Duration>,

    /// Average duration per poll
    pub avg_poll_duration: Duration,

    /// Whether the task completed successfully
    pub completed: bool,
}

impl TaskMetrics {
    /// Create new task metrics
    #[must_use] 
    pub fn new(task_id: TaskId, name: String) -> Self {
        Self {
            task_id,
            name,
            total_duration: Duration::ZERO,
            running_time: Duration::ZERO,
            blocked_time: Duration::ZERO,
            poll_count: 0,
            await_count: 0,
            await_durations: Vec::new(),
            avg_poll_duration: Duration::ZERO,
            completed: false,
        }
    }

    /// Calculate efficiency (running time / total time)
    #[must_use] 
    pub fn efficiency(&self) -> f64 {
        if self.total_duration.is_zero() {
            return 0.0;
        }
        self.running_time.as_secs_f64() / self.total_duration.as_secs_f64()
    }

    /// Check if this task is a potential bottleneck
    #[must_use] 
    pub fn is_bottleneck(&self, threshold_ms: u64) -> bool {
        self.total_duration.as_millis() > u128::from(threshold_ms)
    }
}

/// Statistical percentiles for durations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationStats {
    /// Minimum duration
    pub min: Duration,

    /// Maximum duration
    pub max: Duration,

    /// Mean (average) duration
    pub mean: Duration,

    /// Median (p50)
    pub median: Duration,

    /// 95th percentile
    pub p95: Duration,

    /// 99th percentile
    pub p99: Duration,

    /// Standard deviation
    pub std_dev: f64,

    /// Total count
    pub count: usize,
}

impl DurationStats {
    /// Calculate statistics from a collection of durations
    #[must_use] 
    pub fn from_durations(mut durations: Vec<Duration>) -> Self {
        if durations.is_empty() {
            return Self {
                min: Duration::ZERO,
                max: Duration::ZERO,
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                std_dev: 0.0,
                count: 0,
            };
        }

        durations.sort();
        let count = durations.len();

        let min = durations[0];
        let max = durations[count - 1];

        // Calculate mean
        let sum: Duration = durations.iter().copied().sum();
        let mean = sum / count as u32;

        // Calculate median (p50)
        let median = if count % 2 == 0 {
            (durations[count / 2 - 1] + durations[count / 2]) / 2
        } else {
            durations[count / 2]
        };

        // Calculate percentiles
        let p95_idx = (count as f64 * 0.95) as usize;
        let p99_idx = (count as f64 * 0.99) as usize;
        let p95 = durations[p95_idx.min(count - 1)];
        let p99 = durations[p99_idx.min(count - 1)];

        // Calculate standard deviation
        let mean_secs = mean.as_secs_f64();
        let variance: f64 = durations
            .iter()
            .map(|d| {
                let diff = d.as_secs_f64() - mean_secs;
                diff * diff
            })
            .sum::<f64>()
            / count as f64;
        let std_dev = variance.sqrt();

        Self {
            min,
            max,
            mean,
            median,
            p95,
            p99,
            std_dev,
            count,
        }
    }
}

/// Hot path - a frequently executed code path
#[derive(Debug, Clone)]
pub struct HotPath {
    /// Path identifier (e.g., function name or call chain)
    pub path: String,

    /// Number of times this path was executed
    pub execution_count: u64,

    /// Total time spent in this path
    pub total_time: Duration,

    /// Average time per execution
    pub avg_time: Duration,
}

/// Lock contention metrics for a specific resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockContentionMetrics {
    /// Resource ID
    pub resource_id: ResourceId,

    /// Resource name
    pub name: String,

    /// Number of acquire attempts
    pub acquire_attempts: u64,

    /// Number of successful acquisitions
    pub successful_acquisitions: u64,

    /// Number of times tasks had to wait
    pub contention_count: u64,

    /// Total time spent waiting for this lock
    pub total_wait_time: Duration,

    /// Average wait time per contention
    pub avg_wait_time: Duration,

    /// Maximum wait time observed
    pub max_wait_time: Duration,

    /// Tasks that waited for this resource
    pub waiting_tasks: Vec<TaskId>,

    /// Contention rate (contentions / acquisitions)
    pub contention_rate: f64,
}

impl LockContentionMetrics {
    /// Create new lock contention metrics
    #[must_use]
    pub fn new(resource_id: ResourceId, name: String) -> Self {
        Self {
            resource_id,
            name,
            acquire_attempts: 0,
            successful_acquisitions: 0,
            contention_count: 0,
            total_wait_time: Duration::ZERO,
            avg_wait_time: Duration::ZERO,
            max_wait_time: Duration::ZERO,
            waiting_tasks: Vec::new(),
            contention_rate: 0.0,
        }
    }

    /// Record a wait event
    pub fn record_wait(&mut self, wait_time: Duration, task_id: TaskId) {
        self.contention_count += 1;
        self.total_wait_time += wait_time;

        if wait_time > self.max_wait_time {
            self.max_wait_time = wait_time;
        }

        if !self.waiting_tasks.contains(&task_id) {
            self.waiting_tasks.push(task_id);
        }

        self.update_averages();
    }

    /// Record an acquisition
    pub fn record_acquisition(&mut self) {
        self.successful_acquisitions += 1;
        self.update_averages();
    }

    /// Record an acquire attempt
    pub fn record_attempt(&mut self) {
        self.acquire_attempts += 1;
    }

    /// Update calculated averages
    fn update_averages(&mut self) {
        if self.contention_count > 0 {
            self.avg_wait_time = self.total_wait_time / self.contention_count as u32;
        }

        if self.successful_acquisitions > 0 {
            self.contention_rate = self.contention_count as f64 / self.successful_acquisitions as f64;
        }
    }

    /// Check if this lock is highly contended
    #[must_use]
    pub fn is_highly_contended(&self, threshold: f64) -> bool {
        self.contention_rate > threshold
    }
}

/// Performance profiler
pub struct Profiler {
    /// Metrics for each task
    task_metrics: HashMap<TaskId, TaskMetrics>,

    /// Hot paths (frequently executed code paths)
    hot_paths: HashMap<String, HotPath>,

    /// Lock contention metrics
    lock_contention: HashMap<ResourceId, LockContentionMetrics>,

    /// Bottleneck threshold in milliseconds
    bottleneck_threshold: u64,

    /// High contention threshold (default: 0.3 = 30% contention rate)
    contention_threshold: f64,
}

impl Profiler {
    /// Create a new profiler
    #[must_use]
    pub fn new() -> Self {
        Self {
            task_metrics: HashMap::new(),
            hot_paths: HashMap::new(),
            lock_contention: HashMap::new(),
            bottleneck_threshold: 100, // 100ms default
            contention_threshold: 0.3, // 30% contention rate
        }
    }

    /// Set bottleneck detection threshold
    pub fn set_bottleneck_threshold(&mut self, threshold_ms: u64) {
        self.bottleneck_threshold = threshold_ms;
    }

    /// Set contention threshold
    pub fn set_contention_threshold(&mut self, threshold: f64) {
        self.contention_threshold = threshold;
    }

    /// Record lock wait event
    pub fn record_lock_wait(&mut self, resource_id: ResourceId, name: String, wait_time: Duration, task_id: TaskId) {
        let metrics = self
            .lock_contention
            .entry(resource_id)
            .or_insert_with(|| LockContentionMetrics::new(resource_id, name));

        metrics.record_wait(wait_time, task_id);
    }

    /// Record lock acquisition
    pub fn record_lock_acquisition(&mut self, resource_id: ResourceId, name: String) {
        let metrics = self
            .lock_contention
            .entry(resource_id)
            .or_insert_with(|| LockContentionMetrics::new(resource_id, name));

        metrics.record_acquisition();
    }

    /// Record lock acquire attempt
    pub fn record_lock_attempt(&mut self, resource_id: ResourceId, name: String) {
        let metrics = self
            .lock_contention
            .entry(resource_id)
            .or_insert_with(|| LockContentionMetrics::new(resource_id, name));

        metrics.record_attempt();
    }

    /// Get lock contention metrics
    #[must_use]
    pub fn get_lock_metrics(&self, resource_id: &ResourceId) -> Option<&LockContentionMetrics> {
        self.lock_contention.get(resource_id)
    }

    /// Get all lock contention metrics
    #[must_use]
    pub fn all_lock_metrics(&self) -> Vec<&LockContentionMetrics> {
        self.lock_contention.values().collect()
    }

    /// Get most contended locks
    #[must_use]
    pub fn most_contended_locks(&self, count: usize) -> Vec<&LockContentionMetrics> {
        let mut metrics: Vec<_> = self.lock_contention.values().collect();
        metrics.sort_by(|a, b| b.contention_rate.partial_cmp(&a.contention_rate).unwrap_or(std::cmp::Ordering::Equal));
        metrics.into_iter().take(count).collect()
    }

    /// Identify highly contended locks
    #[must_use]
    pub fn identify_highly_contended_locks(&self) -> Vec<&LockContentionMetrics> {
        self.lock_contention
            .values()
            .filter(|m| m.is_highly_contended(self.contention_threshold))
            .collect()
    }

    /// Record task metrics
    pub fn record_task(&mut self, metrics: TaskMetrics) {
        // Update hot paths
        let path = metrics.name.clone();
        let hot_path = self
            .hot_paths
            .entry(path.clone())
            .or_insert_with(|| HotPath {
                path: path.clone(),
                execution_count: 0,
                total_time: Duration::ZERO,
                avg_time: Duration::ZERO,
            });

        hot_path.execution_count += 1;
        hot_path.total_time += metrics.total_duration;
        hot_path.avg_time = hot_path.total_time / hot_path.execution_count as u32;

        self.task_metrics.insert(metrics.task_id, metrics);
    }

    /// Get metrics for a specific task
    #[must_use] 
    pub fn get_task_metrics(&self, task_id: &TaskId) -> Option<&TaskMetrics> {
        self.task_metrics.get(task_id)
    }

    /// Get all task metrics
    #[must_use] 
    pub fn all_metrics(&self) -> Vec<&TaskMetrics> {
        self.task_metrics.values().collect()
    }

    /// Identify bottleneck tasks
    #[must_use] 
    pub fn identify_bottlenecks(&self) -> Vec<&TaskMetrics> {
        self.task_metrics
            .values()
            .filter(|m| m.is_bottleneck(self.bottleneck_threshold))
            .collect()
    }

    /// Get hot paths sorted by execution count
    #[must_use] 
    pub fn get_hot_paths(&self) -> Vec<&HotPath> {
        let mut paths: Vec<_> = self.hot_paths.values().collect();
        paths.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        paths
    }

    /// Calculate overall statistics
    #[must_use] 
    pub fn calculate_stats(&self) -> DurationStats {
        let durations: Vec<Duration> = self
            .task_metrics
            .values()
            .map(|m| m.total_duration)
            .collect();

        DurationStats::from_durations(durations)
    }

    /// Calculate await point statistics
    #[must_use] 
    pub fn await_stats(&self) -> DurationStats {
        let mut all_await_durations = Vec::new();

        for metrics in self.task_metrics.values() {
            all_await_durations.extend(metrics.await_durations.iter().copied());
        }

        DurationStats::from_durations(all_await_durations)
    }

    /// Find slowest tasks
    #[must_use] 
    pub fn slowest_tasks(&self, count: usize) -> Vec<&TaskMetrics> {
        let mut metrics: Vec<_> = self.task_metrics.values().collect();
        metrics.sort_by(|a, b| b.total_duration.cmp(&a.total_duration));
        metrics.into_iter().take(count).collect()
    }

    /// Find tasks with most polls (busy tasks)
    #[must_use] 
    pub fn busiest_tasks(&self, count: usize) -> Vec<&TaskMetrics> {
        let mut metrics: Vec<_> = self.task_metrics.values().collect();
        metrics.sort_by(|a, b| b.poll_count.cmp(&a.poll_count));
        metrics.into_iter().take(count).collect()
    }

    /// Find least efficient tasks (high blocked time ratio)
    #[must_use]
    pub fn least_efficient_tasks(&self, count: usize) -> Vec<&TaskMetrics> {
        let mut metrics: Vec<_> = self.task_metrics.values().collect();
        metrics.sort_by(|a, b| {
            a.efficiency()
                .partial_cmp(&b.efficiency())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        metrics.into_iter().take(count).collect()
    }

    /// Create a performance snapshot
    #[must_use]
    pub fn create_snapshot(&self, run_id: String) -> comparison::PerformanceSnapshot {
        let task_stats = self.calculate_stats();
        let task_metrics = self.task_metrics.values().cloned().collect();
        let lock_metrics = self.lock_contention.values().cloned().collect();

        let total_tasks = self.task_metrics.len();
        let total_execution_time: Duration = self
            .task_metrics
            .values()
            .map(|m| m.total_duration)
            .sum();

        let avg_task_duration = if total_tasks > 0 {
            total_execution_time / total_tasks as u32
        } else {
            Duration::ZERO
        };

        comparison::PerformanceSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            run_id,
            task_stats,
            task_metrics,
            lock_metrics,
            total_tasks,
            avg_task_duration,
            total_execution_time,
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_stats() {
        let durations = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
            Duration::from_millis(40),
            Duration::from_millis(50),
        ];

        let stats = DurationStats::from_durations(durations);

        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(50));
        assert_eq!(stats.median, Duration::from_millis(30));
        assert_eq!(stats.count, 5);
    }

    #[test]
    fn test_task_efficiency() {
        let mut metrics = TaskMetrics::new(TaskId::new(), "test".to_string());
        metrics.total_duration = Duration::from_millis(100);
        metrics.running_time = Duration::from_millis(80);
        metrics.blocked_time = Duration::from_millis(20);

        let efficiency = metrics.efficiency();
        assert!(
            (efficiency - 0.8).abs() < 0.01,
            "Expected efficiency ~0.8, got {}",
            efficiency
        );
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut metrics = TaskMetrics::new(TaskId::new(), "slow_task".to_string());
        metrics.total_duration = Duration::from_millis(150);

        assert!(metrics.is_bottleneck(100));
        assert!(!metrics.is_bottleneck(200));
    }
}
