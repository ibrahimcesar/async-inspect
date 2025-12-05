//! Tracked Mutex implementation
//!
//! A drop-in replacement for `tokio::sync::Mutex` that automatically tracks
//! contention and integrates with async-inspect's deadlock detection.

use crate::deadlock::{DeadlockDetector, ResourceId, ResourceInfo, ResourceKind};
use crate::inspector::Inspector;
use crate::instrument::current_task_id;
use crate::sync::{LockMetrics, MetricsTracker, WaitTimer};

use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// A tracked mutex that automatically records contention metrics
/// and integrates with deadlock detection.
///
/// This is a drop-in replacement for `tokio::sync::Mutex` with additional
/// observability features.
///
/// # Example
///
/// ```rust,no_run
/// use async_inspect::sync::Mutex;
///
/// #[tokio::main]
/// async fn main() {
///     let mutex = Mutex::new(0, "counter");
///
///     // Spawn multiple tasks that contend for the lock
///     let mutex = std::sync::Arc::new(mutex);
///     let mut handles = vec![];
///
///     for i in 0..10 {
///         let m = mutex.clone();
///         handles.push(tokio::spawn(async move {
///             let mut guard = m.lock().await;
///             *guard += 1;
///         }));
///     }
///
///     for h in handles {
///         h.await.unwrap();
///     }
///
///     // Check contention metrics
///     let metrics = mutex.metrics();
///     println!("Total acquisitions: {}", metrics.acquisitions);
///     println!("Contentions: {}", metrics.contentions);
///     println!("Contention rate: {:.1}%", metrics.contention_rate() * 100.0);
/// }
/// ```
pub struct Mutex<T> {
    /// The underlying Tokio mutex
    inner: TokioMutex<T>,
    /// Name for debugging/display
    name: String,
    /// Resource ID for deadlock detection
    resource_id: ResourceId,
    /// Contention metrics
    metrics: Arc<MetricsTracker>,
}

impl<T> Mutex<T> {
    /// Create a new tracked mutex with a name for identification.
    ///
    /// # Arguments
    ///
    /// * `value` - The initial value to protect
    /// * `name` - A descriptive name for debugging and metrics
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Mutex;
    ///
    /// let mutex = Mutex::new(vec![1, 2, 3], "shared_vector");
    /// ```
    pub fn new(value: T, name: impl Into<String>) -> Self {
        let name = name.into();
        let resource_info = ResourceInfo::new(ResourceKind::Mutex, name.clone());
        let resource_id = resource_info.id;

        // Register with deadlock detector
        let detector = Inspector::global().deadlock_detector();
        let _ = detector.register_resource(resource_info);

        Self {
            inner: TokioMutex::new(value),
            name,
            resource_id,
            metrics: Arc::new(MetricsTracker::new()),
        }
    }

    /// Acquire the lock, blocking until it's available.
    ///
    /// This method automatically:
    /// - Records wait time if there's contention
    /// - Notifies the deadlock detector
    /// - Tracks acquisition metrics
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Mutex;
    ///
    /// # async fn example() {
    /// let mutex = Mutex::new(42, "my_value");
    /// let guard = mutex.lock().await;
    /// println!("Value: {}", *guard);
    /// # }
    /// ```
    pub async fn lock(&self) -> MutexGuard<'_, T> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        // Record that we're waiting for this resource
        if let Some(tid) = task_id {
            detector.wait_for(tid, self.resource_id);
        }

        let timer = WaitTimer::start();

        // Actually acquire the lock
        let guard = self.inner.lock().await;

        // Record metrics
        let wait_time = timer.elapsed_if_contended();
        self.metrics.record_acquisition(wait_time);

        // Record successful acquisition
        if let Some(tid) = task_id {
            detector.acquire(tid, self.resource_id);
        }

        MutexGuard {
            guard,
            resource_id: self.resource_id,
            task_id,
            detector: detector.clone(),
        }
    }

    /// Try to acquire the lock immediately.
    ///
    /// Returns `None` if the lock is already held.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Mutex;
    ///
    /// # async fn example() {
    /// let mutex = Mutex::new(42, "my_value");
    /// let result = mutex.try_lock();
    /// if let Some(guard) = result {
    ///     println!("Got the lock: {}", *guard);
    /// } else {
    ///     println!("Lock is held by another task");
    /// }
    /// # }
    /// ```
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        match self.inner.try_lock() {
            Ok(guard) => {
                // Immediate acquisition - no contention
                self.metrics.record_acquisition(None);

                if let Some(tid) = task_id {
                    detector.acquire(tid, self.resource_id);
                }

                Some(MutexGuard {
                    guard,
                    resource_id: self.resource_id,
                    task_id,
                    detector: detector.clone(),
                })
            }
            Err(_) => None,
        }
    }

    /// Get the current contention metrics for this mutex.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Mutex;
    ///
    /// # async fn example() {
    /// let mutex = Mutex::new(42, "my_value");
    /// // ... some operations ...
    /// let metrics = mutex.metrics();
    /// println!("Acquisitions: {}", metrics.acquisitions);
    /// println!("Contention rate: {:.1}%", metrics.contention_rate() * 100.0);
    /// # }
    /// ```
    #[must_use]
    pub fn metrics(&self) -> LockMetrics {
        self.metrics.get_metrics()
    }

    /// Reset the contention metrics.
    pub fn reset_metrics(&self) {
        self.metrics.reset();
    }

    /// Get the name of this mutex.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the resource ID for deadlock detection.
    #[must_use]
    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Consume the mutex and return the inner value.
    ///
    /// # Panics
    ///
    /// This method will panic if the mutex is poisoned.
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

impl<T: fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metrics = self.metrics();
        f.debug_struct("Mutex")
            .field("name", &self.name)
            .field("resource_id", &self.resource_id)
            .field("acquisitions", &metrics.acquisitions)
            .field("contentions", &metrics.contentions)
            .finish()
    }
}

/// RAII guard for a tracked mutex.
///
/// When this guard is dropped, the lock is released and the deadlock
/// detector is notified.
pub struct MutexGuard<'a, T> {
    guard: tokio::sync::MutexGuard<'a, T>,
    resource_id: ResourceId,
    task_id: Option<crate::task::TaskId>,
    detector: DeadlockDetector,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // Notify deadlock detector that we're releasing the lock
        if let Some(tid) = self.task_id {
            self.detector.release(tid, self.resource_id);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MutexGuard")
            .field("value", &*self.guard)
            .field("resource_id", &self.resource_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_lock_unlock() {
        let mutex = Mutex::new(42, "test_mutex");

        {
            let mut guard = mutex.lock().await;
            assert_eq!(*guard, 42);
            *guard = 100;
        }

        let guard = mutex.lock().await;
        assert_eq!(*guard, 100);

        let metrics = mutex.metrics();
        assert_eq!(metrics.acquisitions, 2);
    }

    #[tokio::test]
    async fn test_try_lock() {
        let mutex = Mutex::new(42, "test_mutex");

        // Should succeed when unlocked
        let guard = mutex.try_lock();
        assert!(guard.is_some());

        // Should fail when already locked
        let guard2 = mutex.try_lock();
        assert!(guard2.is_none());

        // Drop the first guard
        drop(guard);

        // Should succeed again
        let guard3 = mutex.try_lock();
        assert!(guard3.is_some());
    }

    #[tokio::test]
    async fn test_contention_metrics() {
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};

        let mutex = Arc::new(Mutex::new(0, "contended_mutex"));
        let mut handles = vec![];

        // Spawn tasks that will contend for the lock
        for _ in 0..5 {
            let m = mutex.clone();
            handles.push(tokio::spawn(async move {
                let mut guard = m.lock().await;
                // Hold the lock briefly to cause contention
                sleep(Duration::from_millis(10)).await;
                *guard += 1;
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        let metrics = mutex.metrics();
        assert_eq!(metrics.acquisitions, 5);
        // At least some contention should have occurred
        assert!(metrics.contentions > 0);
    }

    #[tokio::test]
    async fn test_into_inner() {
        let mutex = Mutex::new(vec![1, 2, 3], "vec_mutex");
        let inner = mutex.into_inner();
        assert_eq!(inner, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_get_mut() {
        let mut mutex = Mutex::new(42, "mut_mutex");
        *mutex.get_mut() = 100;
        let guard = mutex.lock().await;
        assert_eq!(*guard, 100);
    }
}
