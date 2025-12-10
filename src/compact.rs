//! Compact task storage for reduced memory overhead
//!
//! This module provides memory-efficient alternatives to the standard task
//! structures, reducing per-task memory usage by ~50% through:
//!
//! - String interning for task names
//! - Compact timestamp representation (relative to start time)
//! - Packed state representation
//! - Optional fields stored separately

use crate::intern::{intern, InternedString};
use crate::task::{TaskId, TaskState};
use std::time::{Duration, Instant};

/// Compact representation of task state (1 byte instead of enum + String)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompactState {
    /// Task has been spawned but not yet polled
    Pending = 0,
    /// Task is currently being polled
    Running = 1,
    /// Task is waiting on an async operation
    Blocked = 2,
    /// Task has completed successfully
    Completed = 3,
    /// Task was cancelled or panicked
    Failed = 4,
}

impl From<&TaskState> for CompactState {
    fn from(state: &TaskState) -> Self {
        match state {
            TaskState::Pending => Self::Pending,
            TaskState::Running => Self::Running,
            TaskState::Blocked { .. } => Self::Blocked,
            TaskState::Completed => Self::Completed,
            TaskState::Failed => Self::Failed,
        }
    }
}

impl CompactState {
    /// Convert to full TaskState (without await_point info for Blocked)
    #[must_use]
    pub fn to_task_state(self) -> TaskState {
        match self {
            Self::Pending => TaskState::Pending,
            Self::Running => TaskState::Running,
            Self::Blocked => TaskState::Blocked {
                await_point: String::new(),
            },
            Self::Completed => TaskState::Completed,
            Self::Failed => TaskState::Failed,
        }
    }
}

/// Compact timestamp relative to a base time
/// Stores milliseconds as u32, supporting ~49 days of runtime
#[derive(Debug, Clone, Copy)]
pub struct CompactTimestamp(u32);

impl CompactTimestamp {
    /// Create a new compact timestamp relative to a base time
    #[must_use]
    pub fn new(instant: Instant, base: Instant) -> Self {
        let elapsed = instant.saturating_duration_since(base);
        let ms = elapsed.as_millis().min(u32::MAX as u128) as u32;
        Self(ms)
    }

    /// Convert back to duration from base
    #[must_use]
    pub fn as_duration(&self) -> Duration {
        Duration::from_millis(u64::from(self.0))
    }

    /// Get raw milliseconds value
    #[must_use]
    pub fn as_millis(&self) -> u32 {
        self.0
    }
}

/// Memory-efficient task information
///
/// Size comparison (64-bit system, approximate):
/// - Standard TaskInfo: ~200+ bytes (with String allocations)
/// - CompactTaskInfo: ~48 bytes (fixed size, interned strings)
#[derive(Debug, Clone)]
pub struct CompactTaskInfo {
    /// Task ID (8 bytes)
    pub id: TaskId,
    /// Interned task name (4 bytes)
    pub name: InternedString,
    /// Compact state (1 byte)
    pub state: CompactState,
    /// Created timestamp relative to inspector start (4 bytes)
    pub created_at: CompactTimestamp,
    /// Last updated timestamp (4 bytes)
    pub last_updated: CompactTimestamp,
    /// Poll count (4 bytes - u32 is enough for most cases)
    pub poll_count: u32,
    /// Total run time in microseconds (8 bytes)
    pub total_run_time_us: u64,
    /// Parent task ID (9 bytes with Option overhead, but usually optimized)
    pub parent: Option<TaskId>,
}

impl CompactTaskInfo {
    /// Create new compact task info
    #[must_use]
    pub fn new(name: &str, base_time: Instant) -> Self {
        let now = Instant::now();
        Self {
            id: TaskId::new(),
            name: intern(name),
            state: CompactState::Pending,
            created_at: CompactTimestamp::new(now, base_time),
            last_updated: CompactTimestamp::new(now, base_time),
            poll_count: 0,
            total_run_time_us: 0,
            parent: None,
        }
    }

    /// Get the task name as a String
    #[must_use]
    pub fn name_string(&self) -> String {
        self.name.as_str()
    }

    /// Get total run time as Duration
    #[must_use]
    pub fn total_run_time(&self) -> Duration {
        Duration::from_micros(self.total_run_time_us)
    }

    /// Get age (time since creation)
    #[must_use]
    pub fn age(&self, base_time: Instant) -> Duration {
        let created = self.created_at.as_duration();
        let now = base_time.elapsed();
        now.saturating_sub(created)
    }

    /// Update the state
    pub fn update_state(&mut self, new_state: CompactState, base_time: Instant) {
        self.state = new_state;
        self.last_updated = CompactTimestamp::new(Instant::now(), base_time);
    }

    /// Record a poll
    pub fn record_poll(&mut self, duration: Duration, base_time: Instant) {
        self.poll_count = self.poll_count.saturating_add(1);
        self.total_run_time_us = self
            .total_run_time_us
            .saturating_add(duration.as_micros() as u64);
        self.last_updated = CompactTimestamp::new(Instant::now(), base_time);
    }
}

/// Pool allocator for compact tasks
/// Pre-allocates task slots to reduce allocation overhead
pub struct TaskPool {
    /// Pre-allocated task slots
    tasks: Vec<Option<CompactTaskInfo>>,
    /// Free slot indices
    free_slots: Vec<usize>,
    /// Base time for relative timestamps
    base_time: Instant,
    /// Maximum capacity
    capacity: usize,
}

impl TaskPool {
    /// Create a new task pool with given capacity
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            tasks: vec![None; capacity],
            free_slots: (0..capacity).rev().collect(),
            base_time: Instant::now(),
            capacity,
        }
    }

    /// Get the base time for this pool
    #[must_use]
    pub fn base_time(&self) -> Instant {
        self.base_time
    }

    /// Allocate a new task slot
    pub fn allocate(&mut self, name: &str) -> Option<(usize, &mut CompactTaskInfo)> {
        let slot = self.free_slots.pop()?;
        let task = CompactTaskInfo::new(name, self.base_time);
        self.tasks[slot] = Some(task);
        self.tasks[slot].as_mut().map(|t| (slot, t))
    }

    /// Free a task slot
    pub fn free(&mut self, slot: usize) {
        if slot < self.capacity && self.tasks[slot].is_some() {
            self.tasks[slot] = None;
            self.free_slots.push(slot);
        }
    }

    /// Get a task by slot index
    #[must_use]
    pub fn get(&self, slot: usize) -> Option<&CompactTaskInfo> {
        self.tasks.get(slot).and_then(|t| t.as_ref())
    }

    /// Get a mutable task by slot index
    pub fn get_mut(&mut self, slot: usize) -> Option<&mut CompactTaskInfo> {
        self.tasks.get_mut(slot).and_then(|t| t.as_mut())
    }

    /// Get the number of allocated tasks
    #[must_use]
    pub fn len(&self) -> usize {
        self.capacity - self.free_slots.len()
    }

    /// Check if the pool is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the capacity
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Iterate over all allocated tasks
    pub fn iter(&self) -> impl Iterator<Item = &CompactTaskInfo> {
        self.tasks.iter().filter_map(|t| t.as_ref())
    }

    /// Get memory usage estimate in bytes
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        // Vec<Option<CompactTaskInfo>> + Vec<usize>
        self.capacity * std::mem::size_of::<Option<CompactTaskInfo>>()
            + self.free_slots.capacity() * std::mem::size_of::<usize>()
    }
}

impl Default for TaskPool {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Memory statistics for the compact storage system
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Number of interned strings
    pub interned_strings: usize,
    /// Approximate memory used by string interner
    pub interner_bytes: usize,
    /// Number of tasks in pool
    pub pooled_tasks: usize,
    /// Memory used by task pool
    pub pool_bytes: usize,
    /// Total memory usage
    pub total_bytes: usize,
    /// Estimated savings vs standard storage
    pub estimated_savings_percent: f64,
}

impl MemoryStats {
    /// Calculate memory stats for a task pool
    #[must_use]
    pub fn calculate(pool: &TaskPool) -> Self {
        use crate::intern::StringInterner;

        let interner = StringInterner::global();
        let interned_strings = interner.len();
        let interner_bytes = interner.memory_usage();
        let pooled_tasks = pool.len();
        let pool_bytes = pool.memory_usage();
        let total_bytes = interner_bytes + pool_bytes;

        // Estimate standard storage: ~200 bytes per task + string allocations
        let standard_estimate = pooled_tasks * 200;
        let estimated_savings_percent = if standard_estimate > 0 {
            (1.0 - (total_bytes as f64 / standard_estimate as f64)) * 100.0
        } else {
            0.0
        };

        Self {
            interned_strings,
            interner_bytes,
            pooled_tasks,
            pool_bytes,
            total_bytes,
            estimated_savings_percent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_task_info_size() {
        // Verify the compact task info is reasonably small
        let size = std::mem::size_of::<CompactTaskInfo>();
        assert!(
            size <= 64,
            "CompactTaskInfo should be <= 64 bytes, got {size}"
        );
    }

    #[test]
    fn test_task_pool() {
        let mut pool = TaskPool::new(10);

        // Allocate first task
        let (slot1, _) = pool.allocate("task1").unwrap();
        assert_eq!(pool.get(slot1).unwrap().name_string(), "task1");

        // Allocate second task
        let (slot2, _) = pool.allocate("task2").unwrap();
        assert_eq!(pool.len(), 2);

        pool.free(slot1);
        assert_eq!(pool.len(), 1);

        // Slot should be reusable
        let (new_slot, _) = pool.allocate("task3").unwrap();
        assert_eq!(new_slot, slot1);

        pool.free(slot2);
        pool.free(new_slot);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_compact_timestamp() {
        let base = Instant::now();
        std::thread::sleep(Duration::from_millis(10));
        let now = Instant::now();

        let ts = CompactTimestamp::new(now, base);
        let duration = ts.as_duration();

        // Should be approximately 10ms (with some tolerance)
        assert!(duration.as_millis() >= 10);
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_string_interning_dedup() {
        let mut pool = TaskPool::new(100);

        // Create many tasks with same name
        for _ in 0..50 {
            pool.allocate("fetch_user");
        }

        // String should only be interned once
        use crate::intern::StringInterner;
        let interner = StringInterner::global();
        // Note: global interner may have other strings, so we just check it's < 50
        assert!(interner.len() < 50);
    }
}
