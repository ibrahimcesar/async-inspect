//! Execution timeline tracking
//!
//! This module provides event tracking and timeline management for async operations.
//!
//! # Storage Modes
//!
//! The timeline supports two storage modes:
//!
//! - **Unbounded** (default): Events are stored in a `Vec`, growing as needed.
//!   Best for debugging and development.
//!
//! - **Ring Buffer**: Events are stored in a fixed-size ring buffer.
//!   Oldest events are automatically evicted. Best for production.
//!
//! # Example
//!
//! ```rust,ignore
//! use async_inspect::timeline::{Timeline, TimelineConfig};
//!
//! // Production mode with ring buffer
//! let timeline = Timeline::with_config(TimelineConfig::ring_buffer(10_000));
//!
//! // Development mode (unbounded)
//! let timeline = Timeline::new();
//! ```

use crate::ringbuf::RingBuffer;
use crate::task::{TaskId, TaskState};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, Instant};

/// Unique identifier for an event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(u64);

impl EventId {
    /// Create a new event ID
    #[must_use]
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw u64 value
    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.0
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
            Self::TaskSpawned { name, .. } => write!(f, "Spawned: {name}"),
            Self::PollStarted => write!(f, "Poll started"),
            Self::PollEnded { duration } => {
                write!(f, "Poll ended ({:.2}ms)", duration.as_secs_f64() * 1000.0)
            }
            Self::AwaitStarted { await_point, .. } => write!(f, "Await started: {await_point}"),
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
                    write!(f, "Failed: {err}")
                } else {
                    write!(f, "Failed")
                }
            }
            Self::InspectionPoint { label, message } => {
                if let Some(msg) = message {
                    write!(f, "Inspection[{label}]: {msg}")
                } else {
                    write!(f, "Inspection[{label}]")
                }
            }
            Self::StateChanged {
                old_state,
                new_state,
            } => {
                write!(f, "State: {old_state} → {new_state}")
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
    #[must_use]
    pub fn new(id: u64, task_id: TaskId, kind: EventKind) -> Self {
        Self {
            id: EventId::new(id),
            task_id,
            timestamp: Instant::now(),
            kind,
        }
    }

    /// Get the age of this event
    #[must_use]
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

/// Configuration for timeline storage
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    /// Storage mode
    pub mode: StorageMode,
    /// Maximum events (only used for ring buffer mode)
    pub max_events: usize,
}

impl TimelineConfig {
    /// Create unbounded (Vec-based) storage config
    #[must_use]
    pub fn unbounded() -> Self {
        Self {
            mode: StorageMode::Unbounded,
            max_events: 0,
        }
    }

    /// Create ring buffer storage config with specified capacity
    #[must_use]
    pub fn ring_buffer(capacity: usize) -> Self {
        Self {
            mode: StorageMode::RingBuffer,
            max_events: capacity,
        }
    }

    /// Create production config (ring buffer with 10k capacity)
    #[must_use]
    pub fn production() -> Self {
        Self::ring_buffer(10_000)
    }

    /// Create development config (unbounded)
    #[must_use]
    pub fn development() -> Self {
        Self::unbounded()
    }
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self::unbounded()
    }
}

/// Storage mode for the timeline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageMode {
    /// Unbounded Vec storage (for development)
    Unbounded,
    /// Fixed-size ring buffer (for production)
    RingBuffer,
}

/// Internal storage enum
enum TimelineStorage {
    /// Vec-based unbounded storage
    Vec(Vec<Event>),
    /// Ring buffer for bounded storage
    Ring(RingBuffer<Event>),
}

impl std::fmt::Debug for TimelineStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vec(v) => f.debug_tuple("Vec").field(&v.len()).finish(),
            Self::Ring(r) => f.debug_tuple("Ring").field(&r.len()).finish(),
        }
    }
}

/// Timeline of events
///
/// Supports two storage modes:
/// - **Unbounded**: Uses a Vec, grows indefinitely (default)
/// - **Ring Buffer**: Fixed size, oldest events evicted automatically
#[derive(Debug)]
pub struct Timeline {
    /// Event storage
    storage: TimelineStorage,

    /// Start time of the timeline
    start_time: Option<Instant>,

    /// Storage mode
    mode: StorageMode,
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Timeline {
    /// Create a new timeline with unbounded storage
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: TimelineStorage::Vec(Vec::new()),
            start_time: None,
            mode: StorageMode::Unbounded,
        }
    }

    /// Create a new timeline with the specified configuration
    #[must_use]
    pub fn with_config(config: TimelineConfig) -> Self {
        let storage = match config.mode {
            StorageMode::Unbounded => TimelineStorage::Vec(Vec::new()),
            StorageMode::RingBuffer => {
                TimelineStorage::Ring(RingBuffer::new(config.max_events.max(1)))
            }
        };

        Self {
            storage,
            start_time: None,
            mode: config.mode,
        }
    }

    /// Create a ring buffer timeline with specified capacity
    #[must_use]
    pub fn with_ring_buffer(capacity: usize) -> Self {
        Self::with_config(TimelineConfig::ring_buffer(capacity))
    }

    /// Get the storage mode
    #[must_use]
    pub fn storage_mode(&self) -> StorageMode {
        self.mode
    }

    /// Check if using ring buffer mode
    #[must_use]
    pub fn is_ring_buffer(&self) -> bool {
        self.mode == StorageMode::RingBuffer
    }

    /// Add an event to the timeline
    pub fn add_event(&mut self, event: Event) {
        if self.start_time.is_none() {
            self.start_time = Some(event.timestamp);
        }

        match &mut self.storage {
            TimelineStorage::Vec(vec) => vec.push(event),
            TimelineStorage::Ring(ring) => ring.push(event),
        }
    }

    /// Get all events as a slice (only for Vec mode) or Vec (for ring buffer)
    ///
    /// For ring buffer mode, this allocates. Use `events_vec()` for both modes.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        match &self.storage {
            TimelineStorage::Vec(vec) => vec,
            TimelineStorage::Ring(_) => {
                // Note: This is a limitation - ring buffer can't return a slice
                // Use events_vec() instead for ring buffer mode
                &[]
            }
        }
    }

    /// Get all events as a Vec (works for both modes)
    #[must_use]
    pub fn events_vec(&self) -> Vec<Event> {
        match &self.storage {
            TimelineStorage::Vec(vec) => vec.clone(),
            TimelineStorage::Ring(ring) => ring.to_vec(),
        }
    }

    /// Get events for a specific task
    #[must_use]
    pub fn events_for_task(&self, task_id: TaskId) -> Vec<&Event> {
        match &self.storage {
            TimelineStorage::Vec(vec) => vec.iter().filter(|e| e.task_id == task_id).collect(),
            TimelineStorage::Ring(_) => {
                // For ring buffer, we need to allocate anyway
                // This is a limitation of the current design
                Vec::new()
            }
        }
    }

    /// Get events for a specific task (allocating version, works for both modes)
    #[must_use]
    pub fn events_for_task_vec(&self, task_id: TaskId) -> Vec<Event> {
        match &self.storage {
            TimelineStorage::Vec(vec) => vec
                .iter()
                .filter(|e| e.task_id == task_id)
                .cloned()
                .collect(),
            TimelineStorage::Ring(ring) => ring
                .to_vec()
                .into_iter()
                .filter(|e| e.task_id == task_id)
                .collect(),
        }
    }

    /// Get the most recent N events
    #[must_use]
    pub fn recent_events(&self, n: usize) -> Vec<Event> {
        match &self.storage {
            TimelineStorage::Vec(vec) => {
                let start = vec.len().saturating_sub(n);
                vec[start..].to_vec()
            }
            TimelineStorage::Ring(ring) => ring.recent(n),
        }
    }

    /// Get the total duration of the timeline
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.start_time
            .map_or(Duration::ZERO, |start| start.elapsed())
    }

    /// Get number of events
    #[must_use]
    pub fn len(&self) -> usize {
        match &self.storage {
            TimelineStorage::Vec(vec) => vec.len(),
            TimelineStorage::Ring(ring) => ring.len(),
        }
    }

    /// Check if timeline is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the capacity (only meaningful for ring buffer mode)
    #[must_use]
    pub fn capacity(&self) -> Option<usize> {
        match &self.storage {
            TimelineStorage::Vec(_) => None,
            TimelineStorage::Ring(ring) => Some(ring.capacity()),
        }
    }

    /// Get ring buffer statistics (only for ring buffer mode)
    #[must_use]
    pub fn ring_stats(&self) -> Option<crate::ringbuf::RingBufferStats> {
        match &self.storage {
            TimelineStorage::Vec(_) => None,
            TimelineStorage::Ring(ring) => Some(ring.stats()),
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        match &mut self.storage {
            TimelineStorage::Vec(vec) => vec.clear(),
            TimelineStorage::Ring(ring) => ring.clear(),
        }
        self.start_time = None;
    }

    /// Get memory usage estimate in bytes
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        match &self.storage {
            TimelineStorage::Vec(vec) => {
                vec.capacity() * std::mem::size_of::<Event>() + std::mem::size_of::<Self>()
            }
            TimelineStorage::Ring(ring) => ring.memory_usage() + std::mem::size_of::<Self>(),
        }
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
        assert!(!timeline.is_ring_buffer());
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

    #[test]
    fn test_ring_buffer_timeline() {
        let mut timeline = Timeline::with_ring_buffer(3);

        assert!(timeline.is_ring_buffer());
        assert_eq!(timeline.capacity(), Some(3));

        let task_id = TaskId::new();

        // Add 5 events to a buffer of size 3
        for i in 0..5 {
            timeline.add_event(Event::new(i, task_id, EventKind::PollStarted));
        }

        // Should only have 3 events (oldest evicted)
        assert_eq!(timeline.len(), 3);

        // Check ring stats
        let stats = timeline.ring_stats().unwrap();
        assert_eq!(stats.total_writes, 5);
        assert_eq!(stats.overwrites, 2);
    }

    #[test]
    fn test_recent_events() {
        let mut timeline = Timeline::new();
        let task_id = TaskId::new();

        for i in 0..10 {
            timeline.add_event(Event::new(i, task_id, EventKind::PollStarted));
        }

        let recent = timeline.recent_events(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].id.as_u64(), 7);
        assert_eq!(recent[1].id.as_u64(), 8);
        assert_eq!(recent[2].id.as_u64(), 9);
    }

    #[test]
    fn test_timeline_config() {
        let config = TimelineConfig::production();
        assert_eq!(config.mode, StorageMode::RingBuffer);
        assert_eq!(config.max_events, 10_000);

        let config = TimelineConfig::development();
        assert_eq!(config.mode, StorageMode::Unbounded);
    }

    #[test]
    fn test_events_vec_both_modes() {
        // Test Vec mode
        let mut timeline_vec = Timeline::new();
        let task_id = TaskId::new();
        timeline_vec.add_event(Event::new(1, task_id, EventKind::PollStarted));
        assert_eq!(timeline_vec.events_vec().len(), 1);

        // Test Ring mode
        let mut timeline_ring = Timeline::with_ring_buffer(10);
        timeline_ring.add_event(Event::new(1, task_id, EventKind::PollStarted));
        assert_eq!(timeline_ring.events_vec().len(), 1);
    }

    #[test]
    fn test_events_for_task_vec() {
        let mut timeline = Timeline::with_ring_buffer(10);
        let task1 = TaskId::new();
        let task2 = TaskId::new();

        timeline.add_event(Event::new(1, task1, EventKind::PollStarted));
        timeline.add_event(Event::new(2, task2, EventKind::PollStarted));
        timeline.add_event(Event::new(3, task1, EventKind::PollStarted));

        let task1_events = timeline.events_for_task_vec(task1);
        assert_eq!(task1_events.len(), 2);
    }
}
