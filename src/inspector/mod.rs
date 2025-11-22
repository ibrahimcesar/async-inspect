//! Core inspection functionality
//!
//! This module provides the main `Inspector` type that manages task tracking
//! and event collection.

use crate::task::{TaskId, TaskInfo, TaskState};
use crate::timeline::{Event, EventKind, Timeline};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Global inspector instance
static GLOBAL_INSPECTOR: once_cell::sync::Lazy<Inspector> =
    once_cell::sync::Lazy::new(Inspector::new);

/// Main inspector for tracking async execution
#[derive(Clone)]
pub struct Inspector {
    /// Shared state
    state: Arc<InspectorState>,
}

struct InspectorState {
    /// All tracked tasks
    tasks: RwLock<HashMap<TaskId, TaskInfo>>,

    /// Timeline of events
    timeline: RwLock<Timeline>,

    /// Event counter for unique IDs
    event_counter: AtomicU64,

    /// Whether the inspector is enabled
    enabled: RwLock<bool>,
}

impl Inspector {
    /// Create a new inspector
    pub fn new() -> Self {
        Self {
            state: Arc::new(InspectorState {
                tasks: RwLock::new(HashMap::new()),
                timeline: RwLock::new(Timeline::new()),
                event_counter: AtomicU64::new(1),
                enabled: RwLock::new(true),
            }),
        }
    }

    /// Get the global inspector instance
    pub fn global() -> &'static Self {
        &GLOBAL_INSPECTOR
    }

    /// Check if the inspector is enabled
    pub fn is_enabled(&self) -> bool {
        *self.state.enabled.read()
    }

    /// Enable the inspector
    pub fn enable(&self) {
        *self.state.enabled.write() = true;
    }

    /// Disable the inspector
    pub fn disable(&self) {
        *self.state.enabled.write() = false;
    }

    /// Register a new task
    pub fn register_task(&self, name: String) -> TaskId {
        if !self.is_enabled() {
            return TaskId::new();
        }

        let task = TaskInfo::new(name.clone());
        let task_id = task.id;

        // Add event
        self.add_event(
            task_id,
            EventKind::TaskSpawned {
                name,
                parent: None,
                location: None,
            },
        );

        // Store task
        self.state.tasks.write().insert(task_id, task);

        task_id
    }

    /// Register a child task with a parent
    pub fn register_child_task(&self, name: String, parent_id: TaskId) -> TaskId {
        if !self.is_enabled() {
            return TaskId::new();
        }

        let mut task = TaskInfo::new(name.clone());
        task.parent = Some(parent_id);
        let task_id = task.id;

        // Add event
        self.add_event(
            task_id,
            EventKind::TaskSpawned {
                name,
                parent: Some(parent_id),
                location: None,
            },
        );

        // Store task
        self.state.tasks.write().insert(task_id, task);

        task_id
    }

    /// Register a task with additional metadata
    pub fn register_task_with_info(&self, task: TaskInfo) -> TaskId {
        if !self.is_enabled() {
            return task.id;
        }

        let task_id = task.id;

        // Add event
        self.add_event(
            task_id,
            EventKind::TaskSpawned {
                name: task.name.clone(),
                parent: task.parent,
                location: task.location.clone(),
            },
        );

        // Store task
        self.state.tasks.write().insert(task_id, task);

        task_id
    }

    /// Update task state
    pub fn update_task_state(&self, task_id: TaskId, new_state: TaskState) {
        if !self.is_enabled() {
            return;
        }

        if let Some(task) = self.state.tasks.write().get_mut(&task_id) {
            let old_state = task.state.clone();
            task.update_state(new_state.clone());

            // Add event
            self.add_event(
                task_id,
                EventKind::StateChanged {
                    old_state,
                    new_state,
                },
            );
        }
    }

    /// Record a poll start
    pub fn poll_started(&self, task_id: TaskId) {
        if !self.is_enabled() {
            return;
        }

        self.update_task_state(task_id, TaskState::Running);
        self.add_event(task_id, EventKind::PollStarted);
    }

    /// Record a poll end
    pub fn poll_ended(&self, task_id: TaskId, duration: Duration) {
        if !self.is_enabled() {
            return;
        }

        if let Some(task) = self.state.tasks.write().get_mut(&task_id) {
            task.record_poll(duration);
        }

        self.add_event(task_id, EventKind::PollEnded { duration });
    }

    /// Record an await start
    pub fn await_started(&self, task_id: TaskId, await_point: String, location: Option<String>) {
        if !self.is_enabled() {
            return;
        }

        self.update_task_state(
            task_id,
            TaskState::Blocked {
                await_point: await_point.clone(),
            },
        );

        self.add_event(
            task_id,
            EventKind::AwaitStarted {
                await_point,
                location,
            },
        );
    }

    /// Record an await end
    pub fn await_ended(&self, task_id: TaskId, await_point: String, duration: Duration) {
        if !self.is_enabled() {
            return;
        }

        self.add_event(
            task_id,
            EventKind::AwaitEnded {
                await_point,
                duration,
            },
        );
    }

    /// Mark task as completed
    pub fn task_completed(&self, task_id: TaskId) {
        if !self.is_enabled() {
            return;
        }

        // Get duration while holding read lock, then release it
        let duration = { self.state.tasks.read().get(&task_id).map(|task| task.age()) };

        if let Some(duration) = duration {
            self.update_task_state(task_id, TaskState::Completed);
            self.add_event(task_id, EventKind::TaskCompleted { duration });
        }
    }

    /// Mark task as failed
    pub fn task_failed(&self, task_id: TaskId, error: Option<String>) {
        if !self.is_enabled() {
            return;
        }

        self.update_task_state(task_id, TaskState::Failed);
        self.add_event(task_id, EventKind::TaskFailed { error });
    }

    /// Record an inspection point
    pub fn inspection_point(&self, task_id: TaskId, label: String, message: Option<String>) {
        if !self.is_enabled() {
            return;
        }

        self.add_event(task_id, EventKind::InspectionPoint { label, message });
    }

    /// Add an event to the timeline
    pub fn add_event(&self, task_id: TaskId, kind: EventKind) {
        let event_id = self.state.event_counter.fetch_add(1, Ordering::Relaxed);
        let event = Event::new(event_id, task_id, kind);
        self.state.timeline.write().add_event(event);
    }

    /// Get a task by ID
    pub fn get_task(&self, task_id: TaskId) -> Option<TaskInfo> {
        self.state.tasks.read().get(&task_id).cloned()
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> Vec<TaskInfo> {
        self.state.tasks.read().values().cloned().collect()
    }

    /// Get all events
    pub fn get_events(&self) -> Vec<Event> {
        self.state.timeline.read().events().to_vec()
    }

    /// Get events for a specific task
    pub fn get_task_events(&self, task_id: TaskId) -> Vec<Event> {
        self.state
            .timeline
            .read()
            .events_for_task(task_id)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Build a performance profiler from collected data
    pub fn build_profiler(&self) -> crate::profile::Profiler {
        use crate::profile::{Profiler, TaskMetrics};
        use crate::timeline::EventKind;

        let mut profiler = Profiler::new();
        let tasks = self.state.tasks.read();
        let timeline = self.state.timeline.read();

        for task in tasks.values() {
            let mut metrics = TaskMetrics::new(task.id, task.name.clone());

            // Calculate durations
            metrics.total_duration = task.age();
            metrics.running_time = task.total_run_time;
            metrics.blocked_time = if metrics.total_duration > task.total_run_time {
                metrics.total_duration - task.total_run_time
            } else {
                Duration::ZERO
            };

            // Set poll count
            metrics.poll_count = task.poll_count;

            // Calculate average poll duration
            if task.poll_count > 0 {
                metrics.avg_poll_duration = task.total_run_time / task.poll_count as u32;
            }

            // Check if completed
            metrics.completed = matches!(task.state, TaskState::Completed);

            // Collect await durations from events
            let task_events: Vec<&Event> = timeline
                .events()
                .into_iter()
                .filter(|e| e.task_id == task.id)
                .collect();

            let mut await_start_times: HashMap<String, std::time::Instant> = HashMap::new();

            for event in task_events {
                match &event.kind {
                    EventKind::AwaitStarted { await_point, .. } => {
                        await_start_times.insert(await_point.clone(), event.timestamp);
                    }
                    EventKind::AwaitEnded { await_point, .. } => {
                        if let Some(start_time) = await_start_times.remove(&await_point.clone()) {
                            let duration = event.timestamp.duration_since(start_time);
                            metrics.await_durations.push(duration);
                            metrics.await_count += 1;
                        }
                    }
                    _ => {}
                }
            }

            profiler.record_task(metrics);
        }

        profiler
    }

    /// Get statistics
    pub fn stats(&self) -> InspectorStats {
        let tasks = self.state.tasks.read();
        let timeline = self.state.timeline.read();

        let total = tasks.len();
        let pending = tasks
            .values()
            .filter(|t| matches!(t.state, TaskState::Pending))
            .count();
        let running = tasks
            .values()
            .filter(|t| matches!(t.state, TaskState::Running))
            .count();
        let blocked = tasks
            .values()
            .filter(|t| matches!(t.state, TaskState::Blocked { .. }))
            .count();
        let completed = tasks
            .values()
            .filter(|t| matches!(t.state, TaskState::Completed))
            .count();
        let failed = tasks
            .values()
            .filter(|t| matches!(t.state, TaskState::Failed))
            .count();

        InspectorStats {
            total_tasks: total,
            pending_tasks: pending,
            running_tasks: running,
            blocked_tasks: blocked,
            completed_tasks: completed,
            failed_tasks: failed,
            total_events: timeline.len(),
            timeline_duration: timeline.duration(),
        }
    }

    /// Clear all data
    pub fn clear(&self) {
        self.state.tasks.write().clear();
        self.state.timeline.write().clear();
        self.state.event_counter.store(1, Ordering::Relaxed);
    }

    /// Reset the inspector
    pub fn reset(&self) {
        self.clear();
        self.enable();
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}

/// Inspector statistics
#[derive(Debug, Clone)]
pub struct InspectorStats {
    /// Total number of tasks
    pub total_tasks: usize,
    /// Tasks in pending state
    pub pending_tasks: usize,
    /// Tasks in running state
    pub running_tasks: usize,
    /// Tasks in blocked state
    pub blocked_tasks: usize,
    /// Completed tasks
    pub completed_tasks: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Total number of events
    pub total_events: usize,
    /// Total timeline duration
    pub timeline_duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_creation() {
        let inspector = Inspector::new();
        assert!(inspector.is_enabled());
    }

    #[test]
    fn test_register_task() {
        let inspector = Inspector::new();
        let task_id = inspector.register_task("test_task".to_string());
        let task = inspector.get_task(task_id).unwrap();
        assert_eq!(task.name, "test_task");
    }

    #[test]
    fn test_task_lifecycle() {
        let inspector = Inspector::new();
        let task_id = inspector.register_task("test".to_string());

        inspector.poll_started(task_id);
        inspector.poll_ended(task_id, Duration::from_millis(10));
        inspector.task_completed(task_id);

        let task = inspector.get_task(task_id).unwrap();
        assert_eq!(task.state, TaskState::Completed);
        assert_eq!(task.poll_count, 1);
    }

    #[test]
    fn test_stats() {
        let inspector = Inspector::new();
        inspector.register_task("task1".to_string());
        inspector.register_task("task2".to_string());

        let stats = inspector.stats();
        assert_eq!(stats.total_tasks, 2);
    }
}
