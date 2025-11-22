//! Execution timeline tracking
//!
//! This module provides event tracking and timeline management for async operations.

use crate::task::{TaskId, TaskState};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, Instant};

/// Unique identifier for an event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(u64);

impl EventId {
    /// Create a new event ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Type of event that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    /// A new task was spawned
    TaskSpawned {
        /// Task name
        name: String,
        /// Parent task, if any
        parent: Option<TaskId>,
        /// Source location
        location: Option<String>,
    },

    /// Task started being polled
    PollStarted,

    /// Task finished being polled
    PollEnded {
        /// Time spent in this poll
        duration: Duration,
    },

    /// Task started waiting at an await point
    AwaitStarted {
        /// Name/description of what we're waiting for
        await_point: String,
        /// Source location
        location: Option<String>,
    },

    /// Task finished waiting at an await point
    AwaitEnded {
        /// Name of the await point
        await_point: String,
        /// How long we waited
        duration: Duration,
    },

    /// Task completed successfully
    TaskCompleted {
        /// Total task duration
        duration: Duration,
    },

    /// Task failed or was cancelled
    TaskFailed {
        /// Error message, if any
        error: Option<String>,
    },

    /// Custom inspection point
    InspectionPoint {
        /// Label for this point
        label: String,
        /// Optional message
        message: Option<String>,
    },

    /// State change event
    StateChanged {
        /// Previous state
        old_state: TaskState,
        /// New state
        new_state: TaskState,
    },
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskSpawned { name, .. } => write!(f, "Spawned: {}", name),
            Self::PollStarted => write!(f, "Poll started"),
            Self::PollEnded { duration } => {
                write!(f, "Poll ended ({:.2}ms)", duration.as_secs_f64() * 1000.0)
            }
            Self::AwaitStarted { await_point, .. } => write!(f, "Await started: {}", await_point),
            Self::AwaitEnded {
                await_point,
                duration,
            } => {
                write!(
                    f,
                    "Await ended: {} ({:.2}ms)",
                    await_point,
                    duration.as_secs_f64() * 1000.0
                )
            }
            Self::TaskCompleted { duration } => {
                write!(f, "Completed ({:.2}s)", duration.as_secs_f64())
            }
            Self::TaskFailed { error } => {
                if let Some(err) = error {
                    write!(f, "Failed: {}", err)
                } else {
                    write!(f, "Failed")
                }
            }
            Self::InspectionPoint { label, message } => {
                if let Some(msg) = message {
                    write!(f, "Inspection[{}]: {}", label, msg)
                } else {
                    write!(f, "Inspection[{}]", label)
                }
            }
            Self::StateChanged {
                old_state,
                new_state,
            } => {
                write!(f, "State: {} → {}", old_state, new_state)
            }
        }
    }
}

/// An event that occurred during async execution
#[derive(Debug, Clone)]
pub struct Event {
    /// Unique event identifier
    pub id: EventId,

    /// Task this event belongs to
    pub task_id: TaskId,

    /// When the event occurred
    pub timestamp: Instant,

    /// Type and details of the event
    pub kind: EventKind,
}

impl Event {
    /// Create a new event
    pub fn new(id: u64, task_id: TaskId, kind: EventKind) -> Self {
        Self {
            id: EventId::new(id),
            task_id,
            timestamp: Instant::now(),
            kind,
        }
    }

    /// Get the age of this event
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:.3}s] Task {}: {}",
            self.age().as_secs_f64(),
            self.task_id,
            self.kind
        )
    }
}

/// Timeline of events
#[derive(Debug, Default)]
pub struct Timeline {
    /// All events in chronological order
    events: Vec<Event>,

    /// Start time of the timeline
    start_time: Option<Instant>,
}

impl Timeline {
    /// Create a new timeline
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            start_time: None,
        }
    }

    /// Add an event to the timeline
    pub fn add_event(&mut self, event: Event) {
        if self.start_time.is_none() {
            self.start_time = Some(event.timestamp);
        }
        self.events.push(event);
    }

    /// Get all events
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Get events for a specific task
    pub fn events_for_task(&self, task_id: TaskId) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.task_id == task_id)
            .collect()
    }

    /// Get the total duration of the timeline
    pub fn duration(&self) -> Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Get number of events
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if timeline is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.start_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskId;

    #[test]
    fn test_timeline_creation() {
        let timeline = Timeline::new();
        assert!(timeline.is_empty());
        assert_eq!(timeline.len(), 0);
    }

    #[test]
    fn test_add_event() {
        let mut timeline = Timeline::new();
        let task_id = TaskId::new();
        let event = Event::new(
            1,
            task_id,
            EventKind::TaskSpawned {
                name: "test".to_string(),
                parent: None,
                location: None,
            },
        );

        timeline.add_event(event);
        assert_eq!(timeline.len(), 1);
    }

    #[test]
    fn test_events_for_task() {
        let mut timeline = Timeline::new();
        let task1 = TaskId::new();
        let task2 = TaskId::new();

        timeline.add_event(Event::new(1, task1, EventKind::PollStarted));
        timeline.add_event(Event::new(2, task2, EventKind::PollStarted));
        timeline.add_event(Event::new(
            3,
            task1,
            EventKind::PollEnded {
                duration: Duration::from_millis(10),
            },
        ));

        let task1_events = timeline.events_for_task(task1);
        assert_eq!(task1_events.len(), 2);
    }
}
