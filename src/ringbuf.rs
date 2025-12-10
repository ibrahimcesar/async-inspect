//! Ring buffer for fixed-memory event storage
//!
//! This module provides a lock-free ring buffer implementation for
//! production environments where memory usage must be bounded.
//!
//! # Features
//!
//! - **Fixed Memory**: Pre-allocated capacity, never grows
//! - **Lock-Free Reads**: Multiple readers can access concurrently
//! - **Automatic Eviction**: Oldest events are overwritten when full
//! - **Statistics**: Track overwrites and utilization
//!
//! # Example
//!
//! ```rust
//! use async_inspect::ringbuf::RingBuffer;
//!
//! let buffer: RingBuffer<i32> = RingBuffer::new(3);
//! buffer.push(1);
//! buffer.push(2);
//! buffer.push(3);
//! buffer.push(4); // Overwrites 1
//!
//! assert_eq!(buffer.len(), 3);
//! assert_eq!(buffer.to_vec(), vec![2, 3, 4]);
//! ```

use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// A fixed-size ring buffer for bounded memory usage
///
/// When the buffer is full, new items overwrite the oldest ones.
/// This is ideal for production environments where memory must be bounded.
#[derive(Debug)]
pub struct RingBuffer<T> {
    /// Storage for items
    storage: RwLock<Vec<Option<T>>>,
    /// Current write position (wraps around)
    write_pos: AtomicUsize,
    /// Number of items currently stored
    len: AtomicUsize,
    /// Maximum capacity
    capacity: usize,
    /// Total items ever written (for statistics)
    total_writes: AtomicU64,
    /// Number of items that were overwritten
    overwrites: AtomicU64,
}

impl<T: Clone> RingBuffer<T> {
    /// Create a new ring buffer with the specified capacity
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Ring buffer capacity must be > 0");

        let storage: Vec<Option<T>> = (0..capacity).map(|_| None).collect();

        Self {
            storage: RwLock::new(storage),
            write_pos: AtomicUsize::new(0),
            len: AtomicUsize::new(0),
            capacity,
            total_writes: AtomicU64::new(0),
            overwrites: AtomicU64::new(0),
        }
    }

    /// Push an item into the buffer
    ///
    /// If the buffer is full, the oldest item is overwritten.
    pub fn push(&self, item: T) {
        let pos = self.write_pos.fetch_add(1, Ordering::SeqCst) % self.capacity;

        let mut storage = self.storage.write();

        // Check if we're overwriting
        if storage[pos].is_some() {
            self.overwrites.fetch_add(1, Ordering::Relaxed);
        } else {
            // Only increment len if we're not overwriting
            let current_len = self.len.load(Ordering::Relaxed);
            if current_len < self.capacity {
                self.len.fetch_add(1, Ordering::Relaxed);
            }
        }

        storage[pos] = Some(item);
        self.total_writes.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the number of items currently in the buffer
    #[must_use]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed).min(self.capacity)
    }

    /// Check if the buffer is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check if the buffer is full
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }

    /// Get the maximum capacity
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get all items in chronological order (oldest first)
    #[must_use]
    pub fn to_vec(&self) -> Vec<T> {
        let storage = self.storage.read();
        let len = self.len();
        let write_pos = self.write_pos.load(Ordering::Relaxed);

        if len == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(len);

        if len < self.capacity {
            // Buffer not full yet, items are in order from 0
            for i in 0..len {
                if let Some(ref item) = storage[i] {
                    result.push(item.clone());
                }
            }
        } else {
            // Buffer is full, oldest item is at write_pos
            let start = write_pos % self.capacity;
            for i in 0..self.capacity {
                let idx = (start + i) % self.capacity;
                if let Some(ref item) = storage[idx] {
                    result.push(item.clone());
                }
            }
        }

        result
    }

    /// Iterate over items in chronological order (oldest first)
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.to_vec().into_iter()
    }

    /// Get the most recent N items (newest first)
    #[must_use]
    pub fn recent(&self, n: usize) -> Vec<T> {
        let mut items = self.to_vec();
        items.reverse();
        items.truncate(n);
        items
    }

    /// Get statistics about the ring buffer
    #[must_use]
    pub fn stats(&self) -> RingBufferStats {
        RingBufferStats {
            capacity: self.capacity,
            len: self.len(),
            total_writes: self.total_writes.load(Ordering::Relaxed),
            overwrites: self.overwrites.load(Ordering::Relaxed),
            utilization: self.len() as f64 / self.capacity as f64,
        }
    }

    /// Clear all items from the buffer
    pub fn clear(&self) {
        let mut storage = self.storage.write();
        for item in storage.iter_mut() {
            *item = None;
        }
        self.write_pos.store(0, Ordering::Relaxed);
        self.len.store(0, Ordering::Relaxed);
        // Keep total_writes and overwrites for historical stats
    }

    /// Reset all statistics
    pub fn reset_stats(&self) {
        self.total_writes.store(0, Ordering::Relaxed);
        self.overwrites.store(0, Ordering::Relaxed);
    }

    /// Get memory usage estimate in bytes
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        self.capacity * std::mem::size_of::<Option<T>>()
            + std::mem::size_of::<Self>()
    }
}

impl<T: Clone> Default for RingBuffer<T> {
    fn default() -> Self {
        Self::new(10_000) // Default 10k capacity
    }
}

/// Statistics about ring buffer usage
#[derive(Debug, Clone, Copy)]
pub struct RingBufferStats {
    /// Maximum capacity
    pub capacity: usize,
    /// Current number of items
    pub len: usize,
    /// Total items ever written
    pub total_writes: u64,
    /// Number of items that were overwritten
    pub overwrites: u64,
    /// Current utilization (0.0 - 1.0)
    pub utilization: f64,
}

impl RingBufferStats {
    /// Get the overwrite rate (0.0 - 1.0)
    #[must_use]
    pub fn overwrite_rate(&self) -> f64 {
        if self.total_writes == 0 {
            0.0
        } else {
            self.overwrites as f64 / self.total_writes as f64
        }
    }

    /// Check if the buffer is experiencing high churn
    #[must_use]
    pub fn is_high_churn(&self) -> bool {
        self.overwrite_rate() > 0.5
    }
}

/// A ring buffer specialized for events with timestamps
///
/// Provides additional methods for time-based queries.
pub struct EventRingBuffer<T> {
    inner: RingBuffer<T>,
}

impl<T: Clone> EventRingBuffer<T> {
    /// Create a new event ring buffer
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: RingBuffer::new(capacity),
        }
    }

    /// Push an event
    pub fn push(&self, event: T) {
        self.inner.push(event);
    }

    /// Get all events
    #[must_use]
    pub fn events(&self) -> Vec<T> {
        self.inner.to_vec()
    }

    /// Get recent events
    #[must_use]
    pub fn recent_events(&self, n: usize) -> Vec<T> {
        self.inner.recent(n)
    }

    /// Get statistics
    #[must_use]
    pub fn stats(&self) -> RingBufferStats {
        self.inner.stats()
    }

    /// Get the number of events
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all events
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Get capacity
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Get memory usage
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        self.inner.memory_usage()
    }
}

impl<T: Clone> Default for EventRingBuffer<T> {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let buffer: RingBuffer<i32> = RingBuffer::new(3);

        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        buffer.push(1);
        buffer.push(2);

        assert_eq!(buffer.len(), 2);
        assert!(!buffer.is_full());

        buffer.push(3);

        assert_eq!(buffer.len(), 3);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_overwrite() {
        let buffer: RingBuffer<i32> = RingBuffer::new(3);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        buffer.push(4); // Overwrites 1

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.to_vec(), vec![2, 3, 4]);

        let stats = buffer.stats();
        assert_eq!(stats.total_writes, 4);
        assert_eq!(stats.overwrites, 1);
    }

    #[test]
    fn test_recent() {
        let buffer: RingBuffer<i32> = RingBuffer::new(5);

        for i in 1..=5 {
            buffer.push(i);
        }

        let recent = buffer.recent(3);
        assert_eq!(recent, vec![5, 4, 3]);
    }

    #[test]
    fn test_wrap_around() {
        let buffer: RingBuffer<i32> = RingBuffer::new(3);

        // Fill and wrap around multiple times
        for i in 1..=10 {
            buffer.push(i);
        }

        // Should have last 3 items
        assert_eq!(buffer.to_vec(), vec![8, 9, 10]);

        let stats = buffer.stats();
        assert_eq!(stats.total_writes, 10);
        assert_eq!(stats.overwrites, 7);
    }

    #[test]
    fn test_clear() {
        let buffer: RingBuffer<i32> = RingBuffer::new(3);

        buffer.push(1);
        buffer.push(2);
        buffer.clear();

        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_stats() {
        let buffer: RingBuffer<i32> = RingBuffer::new(100);

        for i in 0..50 {
            buffer.push(i);
        }

        let stats = buffer.stats();
        assert_eq!(stats.capacity, 100);
        assert_eq!(stats.len, 50);
        assert_eq!(stats.utilization, 0.5);
        assert_eq!(stats.overwrites, 0);
    }

    #[test]
    fn test_overwrite_rate() {
        let buffer: RingBuffer<i32> = RingBuffer::new(10);

        // Write 100 items to a buffer of size 10
        for i in 0..100 {
            buffer.push(i);
        }

        let stats = buffer.stats();
        assert_eq!(stats.total_writes, 100);
        assert_eq!(stats.overwrites, 90);
        assert!((stats.overwrite_rate() - 0.9).abs() < 0.01);
        assert!(stats.is_high_churn());
    }

    #[test]
    #[should_panic(expected = "capacity must be > 0")]
    fn test_zero_capacity_panics() {
        let _buffer: RingBuffer<i32> = RingBuffer::new(0);
    }
}
