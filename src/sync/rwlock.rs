//! Tracked `RwLock` implementation
//!
//! A drop-in replacement for `tokio::sync::RwLock` that automatically tracks
//! contention and integrates with async-inspect's deadlock detection.

use crate::deadlock::{DeadlockDetector, ResourceId, ResourceInfo, ResourceKind};
use crate::inspector::Inspector;
use crate::instrument::current_task_id;
use crate::sync::{LockMetrics, MetricsTracker, WaitTimer};

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;

/// A tracked read-write lock that automatically records contention metrics
/// and integrates with deadlock detection.
///
/// This is a drop-in replacement for `tokio::sync::RwLock` with additional
/// observability features. It tracks read and write operations separately.
///
/// # Example
///
/// ```rust,no_run
/// use async_inspect::sync::RwLock;
///
/// #[tokio::main]
/// async fn main() {
///     let lock = RwLock::new(vec![1, 2, 3], "shared_data");
///
///     // Multiple readers can access simultaneously
///     let lock = std::sync::Arc::new(lock);
///     let mut handles = vec![];
///
///     for i in 0..5 {
///         let l = lock.clone();
///         handles.push(tokio::spawn(async move {
///             let guard = l.read().await;
///             println!("Reader {}: {:?}", i, *guard);
///         }));
///     }
///
///     // Writer has exclusive access
///     {
///         let mut guard = lock.write().await;
///         guard.push(4);
///     }
///
///     for h in handles {
///         h.await.unwrap();
///     }
///
///     // Check contention metrics
///     let (read_metrics, write_metrics) = lock.metrics();
///     println!("Read acquisitions: {}", read_metrics.acquisitions);
///     println!("Write acquisitions: {}", write_metrics.acquisitions);
/// }
/// ```
pub struct RwLock<T> {
    /// The underlying Tokio `RwLock`
    inner: TokioRwLock<T>,
    /// Name for debugging/display
    name: String,
    /// Resource ID for deadlock detection
    resource_id: ResourceId,
    /// Contention metrics for read operations
    read_metrics: Arc<MetricsTracker>,
    /// Contention metrics for write operations
    write_metrics: Arc<MetricsTracker>,
}

impl<T> RwLock<T> {
    /// Create a new tracked `RwLock` with a name for identification.
    ///
    /// # Arguments
    ///
    /// * `value` - The initial value to protect
    /// * `name` - A descriptive name for debugging and metrics
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::RwLock;
    /// use std::collections::HashMap;
    ///
    /// let lock = RwLock::new(HashMap::<String, i32>::new(), "user_cache");
    /// ```
    pub fn new(value: T, name: impl Into<String>) -> Self {
        let name = name.into();
        let resource_info = ResourceInfo::new(ResourceKind::RwLock, name.clone());
        let resource_id = resource_info.id;

        // Register with deadlock detector
        let detector = Inspector::global().deadlock_detector();
        let _ = detector.register_resource(resource_info);

        Self {
            inner: TokioRwLock::new(value),
            name,
            resource_id,
            read_metrics: Arc::new(MetricsTracker::new()),
            write_metrics: Arc::new(MetricsTracker::new()),
        }
    }

    /// Acquire a read lock, blocking until it's available.
    ///
    /// Multiple readers can hold the lock simultaneously, but writers
    /// must wait for all readers to release.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::RwLock;
    ///
    /// # async fn example() {
    /// let lock = RwLock::new(42, "my_value");
    /// let guard = lock.read().await;
    /// println!("Value: {}", *guard);
    /// # }
    /// ```
    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        // Record that we're waiting for this resource
        if let Some(tid) = task_id {
            detector.wait_for(tid, self.resource_id);
        }

        let timer = WaitTimer::start();

        // Actually acquire the lock
        let guard = self.inner.read().await;

        // Record metrics
        let wait_time = timer.elapsed_if_contended();
        self.read_metrics.record_acquisition(wait_time);

        // Record successful acquisition
        if let Some(tid) = task_id {
            detector.acquire(tid, self.resource_id);
        }

        RwLockReadGuard {
            guard,
            resource_id: self.resource_id,
            task_id,
            detector: detector.clone(),
        }
    }

    /// Acquire a write lock, blocking until it's available.
    ///
    /// Writers have exclusive access - no other readers or writers
    /// can hold the lock simultaneously.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::RwLock;
    ///
    /// # async fn example() {
    /// let lock = RwLock::new(42, "my_value");
    /// let mut guard = lock.write().await;
    /// *guard = 100;
    /// # }
    /// ```
    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        // Record that we're waiting for this resource
        if let Some(tid) = task_id {
            detector.wait_for(tid, self.resource_id);
        }

        let timer = WaitTimer::start();

        // Actually acquire the lock
        let guard = self.inner.write().await;

        // Record metrics
        let wait_time = timer.elapsed_if_contended();
        self.write_metrics.record_acquisition(wait_time);

        // Record successful acquisition
        if let Some(tid) = task_id {
            detector.acquire(tid, self.resource_id);
        }

        RwLockWriteGuard {
            guard,
            resource_id: self.resource_id,
            task_id,
            detector: detector.clone(),
        }
    }

    /// Try to acquire a read lock immediately.
    ///
    /// Returns `None` if a writer is holding the lock.
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        match self.inner.try_read() {
            Ok(guard) => {
                self.read_metrics.record_acquisition(None);

                if let Some(tid) = task_id {
                    detector.acquire(tid, self.resource_id);
                }

                Some(RwLockReadGuard {
                    guard,
                    resource_id: self.resource_id,
                    task_id,
                    detector: detector.clone(),
                })
            }
            Err(_) => None,
        }
    }

    /// Try to acquire a write lock immediately.
    ///
    /// Returns `None` if any readers or writers are holding the lock.
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        match self.inner.try_write() {
            Ok(guard) => {
                self.write_metrics.record_acquisition(None);

                if let Some(tid) = task_id {
                    detector.acquire(tid, self.resource_id);
                }

                Some(RwLockWriteGuard {
                    guard,
                    resource_id: self.resource_id,
                    task_id,
                    detector: detector.clone(),
                })
            }
            Err(_) => None,
        }
    }

    /// Get the current contention metrics for this `RwLock`.
    ///
    /// Returns a tuple of (`read_metrics`, `write_metrics`).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::RwLock;
    ///
    /// # async fn example() {
    /// let lock = RwLock::new(42, "my_value");
    /// // ... some operations ...
    /// let (read_metrics, write_metrics) = lock.metrics();
    /// println!("Read acquisitions: {}", read_metrics.acquisitions);
    /// println!("Write acquisitions: {}", write_metrics.acquisitions);
    /// # }
    /// ```
    #[must_use]
    pub fn metrics(&self) -> (LockMetrics, LockMetrics) {
        (
            self.read_metrics.get_metrics(),
            self.write_metrics.get_metrics(),
        )
    }

    /// Get only the read metrics.
    #[must_use]
    pub fn read_metrics(&self) -> LockMetrics {
        self.read_metrics.get_metrics()
    }

    /// Get only the write metrics.
    #[must_use]
    pub fn write_metrics(&self) -> LockMetrics {
        self.write_metrics.get_metrics()
    }

    /// Reset all contention metrics.
    pub fn reset_metrics(&self) {
        self.read_metrics.reset();
        self.write_metrics.reset();
    }

    /// Get the name of this `RwLock`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the resource ID for deadlock detection.
    #[must_use]
    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Consume the `RwLock` and return the inner value.
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    /// Get a mutable reference to the inner value without locking.
    ///
    /// This is safe because we have exclusive access via `&mut self`.
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for RwLock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (read_metrics, write_metrics) = self.metrics();
        f.debug_struct("RwLock")
            .field("name", &self.name)
            .field("resource_id", &self.resource_id)
            .field("read_acquisitions", &read_metrics.acquisitions)
            .field("write_acquisitions", &write_metrics.acquisitions)
            .finish()
    }
}

/// RAII guard for a tracked `RwLock` read lock.
///
/// When this guard is dropped, the lock is released and the deadlock
/// detector is notified.
pub struct RwLockReadGuard<'a, T> {
    guard: tokio::sync::RwLockReadGuard<'a, T>,
    resource_id: ResourceId,
    task_id: Option<crate::task::TaskId>,
    detector: DeadlockDetector,
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(tid) = self.task_id {
            self.detector.release(tid, self.resource_id);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RwLockReadGuard")
            .field("value", &*self.guard)
            .field("resource_id", &self.resource_id)
            .finish()
    }
}

/// RAII guard for a tracked `RwLock` write lock.
///
/// When this guard is dropped, the lock is released and the deadlock
/// detector is notified.
pub struct RwLockWriteGuard<'a, T> {
    guard: tokio::sync::RwLockWriteGuard<'a, T>,
    resource_id: ResourceId,
    task_id: Option<crate::task::TaskId>,
    detector: DeadlockDetector,
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(tid) = self.task_id {
            self.detector.release(tid, self.resource_id);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RwLockWriteGuard")
            .field("value", &*self.guard)
            .field("resource_id", &self.resource_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_read_write() {
        let lock = RwLock::new(42, "test_lock");

        // Read
        {
            let guard = lock.read().await;
            assert_eq!(*guard, 42);
        }

        // Write
        {
            let mut guard = lock.write().await;
            *guard = 100;
        }

        // Read again
        let guard = lock.read().await;
        assert_eq!(*guard, 100);

        let (read_metrics, write_metrics) = lock.metrics();
        assert_eq!(read_metrics.acquisitions, 2);
        assert_eq!(write_metrics.acquisitions, 1);
    }

    #[tokio::test]
    async fn test_concurrent_readers() {
        use std::sync::Arc;

        let lock = Arc::new(RwLock::new(vec![1, 2, 3], "shared_vec"));
        let mut handles = vec![];

        // Spawn multiple readers
        for _ in 0..5 {
            let l = lock.clone();
            handles.push(tokio::spawn(async move {
                let guard = l.read().await;
                assert_eq!(guard.len(), 3);
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        let read_metrics = lock.read_metrics();
        assert_eq!(read_metrics.acquisitions, 5);
    }

    #[tokio::test]
    async fn test_try_read_write() {
        let lock = RwLock::new(42, "test_lock");

        // Should succeed when no locks held
        let guard = lock.try_read();
        assert!(guard.is_some());
        drop(guard);

        // Try write should succeed when no locks held
        let guard = lock.try_write();
        assert!(guard.is_some());

        // Try read should fail when write lock is held
        let guard2 = lock.try_read();
        assert!(guard2.is_none());

        drop(guard);

        // Should succeed now
        let guard3 = lock.try_read();
        assert!(guard3.is_some());
    }

    #[tokio::test]
    async fn test_into_inner() {
        let lock = RwLock::new(vec![1, 2, 3], "vec_lock");
        let inner = lock.into_inner();
        assert_eq!(inner, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_get_mut() {
        let mut lock = RwLock::new(42, "mut_lock");
        *lock.get_mut() = 100;
        let guard = lock.read().await;
        assert_eq!(*guard, 100);
    }
}
