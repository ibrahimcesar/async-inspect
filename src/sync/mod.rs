//! Tracked synchronization primitives
//!
//! This module provides drop-in replacements for Tokio's synchronization primitives
//! (`Mutex`, `RwLock`, `Semaphore`) that automatically track contention and integrate
//! with async-inspect's deadlock detection.
//!
//! # Example
//!
//! ```rust,no_run
//! use async_inspect::sync::Mutex;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a tracked mutex (drop-in replacement for tokio::sync::Mutex)
//!     let mutex = Mutex::new(42, "my_counter");
//!
//!     // Use it like normal - tracking is automatic!
//!     {
//!         let mut guard = mutex.lock().await;
//!         *guard += 1;
//!     }
//!
//!     // Check contention metrics
//!     let metrics = mutex.metrics();
//!     println!("Acquisitions: {}", metrics.acquisitions);
//!     println!("Contentions: {}", metrics.contentions);
//! }
//! ```

mod mutex;
mod rwlock;
mod semaphore;

pub use mutex::{Mutex, MutexGuard};
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub use semaphore::{AcquireError, Semaphore, SemaphorePermit, TryAcquireError};

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Metrics for a tracked synchronization primitive
#[derive(Debug, Clone, Default)]
pub struct LockMetrics {
    /// Total number of successful acquisitions
    pub acquisitions: u64,
    /// Number of times a task had to wait (contention)
    pub contentions: u64,
    /// Total time spent waiting for the lock
    pub total_wait_time: Duration,
    /// Maximum wait time observed
    pub max_wait_time: Duration,
    /// Average wait time (when contended)
    pub avg_wait_time: Duration,
}

impl LockMetrics {
    /// Calculate contention rate (0.0 to 1.0)
    #[must_use]
    pub fn contention_rate(&self) -> f64 {
        if self.acquisitions == 0 {
            0.0
        } else {
            self.contentions as f64 / self.acquisitions as f64
        }
    }
}

/// Internal metrics tracker for locks
#[derive(Debug)]
pub(crate) struct MetricsTracker {
    acquisitions: AtomicU64,
    contentions: AtomicU64,
    total_wait_nanos: AtomicU64,
    max_wait_nanos: AtomicU64,
}

impl MetricsTracker {
    pub fn new() -> Self {
        Self {
            acquisitions: AtomicU64::new(0),
            contentions: AtomicU64::new(0),
            total_wait_nanos: AtomicU64::new(0),
            max_wait_nanos: AtomicU64::new(0),
        }
    }

    pub fn record_acquisition(&self, wait_time: Option<Duration>) {
        self.acquisitions.fetch_add(1, Ordering::Relaxed);

        if let Some(wait) = wait_time {
            self.contentions.fetch_add(1, Ordering::Relaxed);
            let nanos = wait.as_nanos() as u64;
            self.total_wait_nanos.fetch_add(nanos, Ordering::Relaxed);

            // Update max wait time (compare-and-swap loop)
            let mut current_max = self.max_wait_nanos.load(Ordering::Relaxed);
            while nanos > current_max {
                match self.max_wait_nanos.compare_exchange_weak(
                    current_max,
                    nanos,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(actual) => current_max = actual,
                }
            }
        }
    }

    pub fn get_metrics(&self) -> LockMetrics {
        let acquisitions = self.acquisitions.load(Ordering::Relaxed);
        let contentions = self.contentions.load(Ordering::Relaxed);
        let total_wait_nanos = self.total_wait_nanos.load(Ordering::Relaxed);
        let max_wait_nanos = self.max_wait_nanos.load(Ordering::Relaxed);

        let total_wait_time = Duration::from_nanos(total_wait_nanos);
        let max_wait_time = Duration::from_nanos(max_wait_nanos);
        let avg_wait_time = if contentions > 0 {
            Duration::from_nanos(total_wait_nanos / contentions)
        } else {
            Duration::ZERO
        };

        LockMetrics {
            acquisitions,
            contentions,
            total_wait_time,
            max_wait_time,
            avg_wait_time,
        }
    }

    pub fn reset(&self) {
        self.acquisitions.store(0, Ordering::Relaxed);
        self.contentions.store(0, Ordering::Relaxed);
        self.total_wait_nanos.store(0, Ordering::Relaxed);
        self.max_wait_nanos.store(0, Ordering::Relaxed);
    }
}

impl Default for MetricsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to measure wait time
pub(crate) struct WaitTimer {
    start: Instant,
    /// Threshold below which we don't count as contention (immediate acquisition)
    threshold: Duration,
}

impl WaitTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
            threshold: Duration::from_micros(10), // 10µs threshold
        }
    }

    /// Returns Some(duration) if there was actual contention, None if immediate
    pub fn elapsed_if_contended(&self) -> Option<Duration> {
        let elapsed = self.start.elapsed();
        if elapsed > self.threshold {
            Some(elapsed)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_tracker() {
        let tracker = MetricsTracker::new();

        // Record some acquisitions
        tracker.record_acquisition(None); // Immediate
        tracker.record_acquisition(Some(Duration::from_millis(10)));
        tracker.record_acquisition(Some(Duration::from_millis(20)));

        let metrics = tracker.get_metrics();
        assert_eq!(metrics.acquisitions, 3);
        assert_eq!(metrics.contentions, 2);
        assert_eq!(metrics.max_wait_time, Duration::from_millis(20));
    }

    #[test]
    fn test_contention_rate() {
        let metrics = LockMetrics {
            acquisitions: 100,
            contentions: 25,
            ..Default::default()
        };
        assert!((metrics.contention_rate() - 0.25).abs() < 0.001);
    }
}
