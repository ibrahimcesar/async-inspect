//! Tracked Semaphore implementation
//!
//! A drop-in replacement for `tokio::sync::Semaphore` that automatically tracks
//! permit acquisition and integrates with async-inspect's deadlock detection.

use crate::deadlock::{DeadlockDetector, ResourceId, ResourceInfo, ResourceKind};
use crate::inspector::Inspector;
use crate::instrument::current_task_id;
use crate::sync::{LockMetrics, MetricsTracker, WaitTimer};

use std::fmt;
use std::sync::Arc;
use tokio::sync::Semaphore as TokioSemaphore;

/// A tracked semaphore that automatically records acquisition metrics
/// and integrates with deadlock detection.
///
/// This is a drop-in replacement for `tokio::sync::Semaphore` with additional
/// observability features.
///
/// # Example
///
/// ```rust,no_run
/// use async_inspect::sync::Semaphore;
///
/// #[tokio::main]
/// async fn main() {
///     // Allow up to 3 concurrent operations
///     let semaphore = Semaphore::new(3, "connection_pool");
///     let semaphore = std::sync::Arc::new(semaphore);
///
///     let mut handles = vec![];
///
///     for i in 0..10 {
///         let sem = semaphore.clone();
///         handles.push(tokio::spawn(async move {
///             let _permit = sem.acquire().await.unwrap();
///             println!("Task {} acquired permit", i);
///             tokio::time::sleep(std::time::Duration::from_millis(100)).await;
///         }));
///     }
///
///     for h in handles {
///         h.await.unwrap();
///     }
///
///     // Check acquisition metrics
///     let metrics = semaphore.metrics();
///     println!("Total acquisitions: {}", metrics.acquisitions);
///     println!("Contentions: {}", metrics.contentions);
/// }
/// ```
pub struct Semaphore {
    /// The underlying Tokio semaphore
    inner: TokioSemaphore,
    /// Name for debugging/display
    name: String,
    /// Resource ID for deadlock detection
    resource_id: ResourceId,
    /// Acquisition metrics
    metrics: Arc<MetricsTracker>,
    /// Initial permit count (for debugging)
    initial_permits: usize,
}

impl Semaphore {
    /// Create a new tracked semaphore with the given number of permits.
    ///
    /// # Arguments
    ///
    /// * `permits` - The number of permits available
    /// * `name` - A descriptive name for debugging and metrics
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Semaphore;
    ///
    /// // Create a semaphore limiting to 5 concurrent operations
    /// let semaphore = Semaphore::new(5, "rate_limiter");
    /// ```
    pub fn new(permits: usize, name: impl Into<String>) -> Self {
        let name = name.into();
        let resource_info = ResourceInfo::new(ResourceKind::Semaphore, name.clone());
        let resource_id = resource_info.id;

        // Register with deadlock detector
        let detector = Inspector::global().deadlock_detector();
        let _ = detector.register_resource(resource_info);

        Self {
            inner: TokioSemaphore::new(permits),
            name,
            resource_id,
            metrics: Arc::new(MetricsTracker::new()),
            initial_permits: permits,
        }
    }

    /// Acquire a permit, blocking until one is available.
    ///
    /// # Returns
    ///
    /// Returns `Ok(SemaphorePermit)` if a permit was acquired, or
    /// `Err(AcquireError)` if the semaphore was closed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Semaphore;
    ///
    /// # async fn example() {
    /// let semaphore = Semaphore::new(3, "pool");
    /// let permit = semaphore.acquire().await.unwrap();
    /// // ... use the resource ...
    /// drop(permit); // Release the permit
    /// # }
    /// ```
    pub async fn acquire(&self) -> Result<SemaphorePermit<'_>, AcquireError> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        // Record that we're waiting for this resource
        if let Some(tid) = task_id {
            detector.wait_for(tid, self.resource_id);
        }

        let timer = WaitTimer::start();

        // Actually acquire the permit
        if let Ok(permit) = self.inner.acquire().await {
            // Record metrics
            let wait_time = timer.elapsed_if_contended();
            self.metrics.record_acquisition(wait_time);

            // Record successful acquisition
            if let Some(tid) = task_id {
                detector.acquire(tid, self.resource_id);
            }

            Ok(SemaphorePermit {
                permit,
                resource_id: self.resource_id,
                task_id,
                detector: detector.clone(),
            })
        } else {
            // Clear wait state on error
            if let Some(tid) = task_id {
                detector.release(tid, self.resource_id);
            }
            Err(AcquireError(()))
        }
    }

    /// Acquire multiple permits at once.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of permits to acquire
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Semaphore;
    ///
    /// # async fn example() {
    /// let semaphore = Semaphore::new(10, "batch_pool");
    /// let permit = semaphore.acquire_many(5).await.unwrap();
    /// // ... use 5 resources at once ...
    /// # }
    /// ```
    pub async fn acquire_many(&self, n: u32) -> Result<SemaphorePermit<'_>, AcquireError> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        if let Some(tid) = task_id {
            detector.wait_for(tid, self.resource_id);
        }

        let timer = WaitTimer::start();

        if let Ok(permit) = self.inner.acquire_many(n).await {
            let wait_time = timer.elapsed_if_contended();
            self.metrics.record_acquisition(wait_time);

            if let Some(tid) = task_id {
                detector.acquire(tid, self.resource_id);
            }

            Ok(SemaphorePermit {
                permit,
                resource_id: self.resource_id,
                task_id,
                detector: detector.clone(),
            })
        } else {
            if let Some(tid) = task_id {
                detector.release(tid, self.resource_id);
            }
            Err(AcquireError(()))
        }
    }

    /// Try to acquire a permit immediately.
    ///
    /// Returns `None` if no permits are available.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Semaphore;
    ///
    /// let semaphore = Semaphore::new(1, "exclusive");
    /// let result = semaphore.try_acquire();
    /// if let Ok(permit) = result {
    ///     println!("Got the permit!");
    ///     drop(permit);
    /// } else {
    ///     println!("No permits available");
    /// }
    /// ```
    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>, TryAcquireError> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        match self.inner.try_acquire() {
            Ok(permit) => {
                self.metrics.record_acquisition(None);

                if let Some(tid) = task_id {
                    detector.acquire(tid, self.resource_id);
                }

                Ok(SemaphorePermit {
                    permit,
                    resource_id: self.resource_id,
                    task_id,
                    detector: detector.clone(),
                })
            }
            Err(tokio::sync::TryAcquireError::NoPermits) => Err(TryAcquireError::NoPermits),
            Err(tokio::sync::TryAcquireError::Closed) => Err(TryAcquireError::Closed),
        }
    }

    /// Try to acquire multiple permits immediately.
    pub fn try_acquire_many(&self, n: u32) -> Result<SemaphorePermit<'_>, TryAcquireError> {
        let detector = Inspector::global().deadlock_detector();
        let task_id = current_task_id();

        match self.inner.try_acquire_many(n) {
            Ok(permit) => {
                self.metrics.record_acquisition(None);

                if let Some(tid) = task_id {
                    detector.acquire(tid, self.resource_id);
                }

                Ok(SemaphorePermit {
                    permit,
                    resource_id: self.resource_id,
                    task_id,
                    detector: detector.clone(),
                })
            }
            Err(tokio::sync::TryAcquireError::NoPermits) => Err(TryAcquireError::NoPermits),
            Err(tokio::sync::TryAcquireError::Closed) => Err(TryAcquireError::Closed),
        }
    }

    /// Get the current number of available permits.
    #[must_use]
    pub fn available_permits(&self) -> usize {
        self.inner.available_permits()
    }

    /// Add permits to the semaphore.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of permits to add
    pub fn add_permits(&self, n: usize) {
        self.inner.add_permits(n);
    }

    /// Close the semaphore.
    ///
    /// All pending acquire operations will fail with an error.
    pub fn close(&self) {
        self.inner.close();
    }

    /// Check if the semaphore is closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Get the current acquisition metrics for this semaphore.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use async_inspect::sync::Semaphore;
    ///
    /// let semaphore = Semaphore::new(5, "pool");
    /// // ... some operations ...
    /// let metrics = semaphore.metrics();
    /// println!("Acquisitions: {}", metrics.acquisitions);
    /// println!("Contention rate: {:.1}%", metrics.contention_rate() * 100.0);
    /// ```
    #[must_use]
    pub fn metrics(&self) -> LockMetrics {
        self.metrics.get_metrics()
    }

    /// Reset the acquisition metrics.
    pub fn reset_metrics(&self) {
        self.metrics.reset();
    }

    /// Get the name of this semaphore.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the resource ID for deadlock detection.
    #[must_use]
    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }

    /// Get the initial number of permits.
    #[must_use]
    pub fn initial_permits(&self) -> usize {
        self.initial_permits
    }
}

impl fmt::Debug for Semaphore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metrics = self.metrics();
        f.debug_struct("Semaphore")
            .field("name", &self.name)
            .field("resource_id", &self.resource_id)
            .field("initial_permits", &self.initial_permits)
            .field("available_permits", &self.available_permits())
            .field("acquisitions", &metrics.acquisitions)
            .field("contentions", &metrics.contentions)
            .finish()
    }
}

/// Error returned when acquiring a permit fails because the semaphore is closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquireError(());

impl fmt::Display for AcquireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "semaphore closed")
    }
}

impl std::error::Error for AcquireError {}

/// Error returned when trying to acquire a permit fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TryAcquireError {
    /// No permits available.
    NoPermits,
    /// The semaphore is closed.
    Closed,
}

impl fmt::Display for TryAcquireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryAcquireError::NoPermits => write!(f, "no permits available"),
            TryAcquireError::Closed => write!(f, "semaphore closed"),
        }
    }
}

impl std::error::Error for TryAcquireError {}

/// RAII guard for a tracked semaphore permit.
///
/// When this guard is dropped, the permit is released and the deadlock
/// detector is notified.
pub struct SemaphorePermit<'a> {
    permit: tokio::sync::SemaphorePermit<'a>,
    resource_id: ResourceId,
    task_id: Option<crate::task::TaskId>,
    detector: DeadlockDetector,
}

impl SemaphorePermit<'_> {
    /// Forget this permit, preventing it from being released.
    ///
    /// This is useful for implementing manual permit management.
    pub fn forget(self) {
        // Use ManuallyDrop to prevent the Drop impl from running
        let mut this = std::mem::ManuallyDrop::new(self);
        // Clear task_id so if Drop somehow ran, it wouldn't notify
        this.task_id = None;
        // SAFETY: We're taking ownership and forgetting, so this is the last access
        let permit = unsafe { std::ptr::read(&this.permit) };
        permit.forget();
    }
}

impl Drop for SemaphorePermit<'_> {
    fn drop(&mut self) {
        if let Some(tid) = self.task_id {
            self.detector.release(tid, self.resource_id);
        }
    }
}

impl fmt::Debug for SemaphorePermit<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SemaphorePermit")
            .field("resource_id", &self.resource_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_acquire_release() {
        let semaphore = Semaphore::new(2, "test_sem");

        let permit1 = semaphore.acquire().await.unwrap();
        assert_eq!(semaphore.available_permits(), 1);

        let permit2 = semaphore.acquire().await.unwrap();
        assert_eq!(semaphore.available_permits(), 0);

        drop(permit1);
        assert_eq!(semaphore.available_permits(), 1);

        drop(permit2);
        assert_eq!(semaphore.available_permits(), 2);

        let metrics = semaphore.metrics();
        assert_eq!(metrics.acquisitions, 2);
    }

    #[tokio::test]
    async fn test_try_acquire() {
        let semaphore = Semaphore::new(1, "test_sem");

        let permit = semaphore.try_acquire();
        assert!(permit.is_ok());

        // Should fail - no permits available
        let permit2 = semaphore.try_acquire();
        assert!(matches!(permit2, Err(TryAcquireError::NoPermits)));

        drop(permit);

        // Should succeed now
        let permit3 = semaphore.try_acquire();
        assert!(permit3.is_ok());
    }

    #[tokio::test]
    async fn test_acquire_many() {
        let semaphore = Semaphore::new(5, "test_sem");

        let permit = semaphore.acquire_many(3).await.unwrap();
        assert_eq!(semaphore.available_permits(), 2);

        drop(permit);
        assert_eq!(semaphore.available_permits(), 5);
    }

    #[tokio::test]
    async fn test_contention() {
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};

        let semaphore = Arc::new(Semaphore::new(1, "contended_sem"));
        let mut handles = vec![];

        for _ in 0..5 {
            let sem = semaphore.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                sleep(Duration::from_millis(10)).await;
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        let metrics = semaphore.metrics();
        assert_eq!(metrics.acquisitions, 5);
        // At least some contention should have occurred
        assert!(metrics.contentions > 0);
    }

    #[tokio::test]
    async fn test_close() {
        let semaphore = Semaphore::new(1, "closeable");

        // Take the only permit
        let _permit = semaphore.acquire().await.unwrap();

        // Close the semaphore
        semaphore.close();
        assert!(semaphore.is_closed());

        // This should fail
        let result = semaphore.try_acquire();
        assert!(matches!(result, Err(TryAcquireError::Closed)));
    }

    #[tokio::test]
    async fn test_add_permits() {
        let semaphore = Semaphore::new(1, "expandable");

        let _permit = semaphore.acquire().await.unwrap();
        assert_eq!(semaphore.available_permits(), 0);

        semaphore.add_permits(2);
        assert_eq!(semaphore.available_permits(), 2);
    }
}
