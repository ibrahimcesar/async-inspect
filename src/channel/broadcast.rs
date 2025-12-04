//! Tracked broadcast channel
//!
//! A drop-in replacement for `tokio::sync::broadcast` that tracks message flow.

use crate::channel::{ChannelMetrics, ChannelMetricsTracker, WaitTimer};
use std::fmt;
use std::sync::Arc;
use tokio::sync::broadcast as tokio_broadcast;

/// Create a tracked broadcast channel.
///
/// # Arguments
///
/// * `capacity` - Maximum number of messages the channel can hold
/// * `name` - A descriptive name for debugging and metrics
///
/// # Example
///
/// ```rust,no_run
/// use async_inspect::channel::broadcast;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, mut rx1) = broadcast::channel::<String>(16, "events");
///     let mut rx2 = tx.subscribe();
///
///     tx.send("hello".into()).unwrap();
///
///     assert_eq!(rx1.recv().await.unwrap(), "hello");
///     assert_eq!(rx2.recv().await.unwrap(), "hello");
/// }
/// ```
pub fn channel<T: Clone>(capacity: usize, name: impl Into<String>) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = tokio_broadcast::channel(capacity);
    let metrics = Arc::new(ChannelMetricsTracker::new());
    let name = Arc::new(name.into());

    (
        Sender {
            inner: tx,
            metrics: metrics.clone(),
            name: name.clone(),
            capacity,
        },
        Receiver {
            inner: rx,
            metrics,
            name,
        },
    )
}

/// Tracked sender half of a broadcast channel.
pub struct Sender<T> {
    inner: tokio_broadcast::Sender<T>,
    metrics: Arc<ChannelMetricsTracker>,
    name: Arc<String>,
    capacity: usize,
}

impl<T: Clone> Sender<T> {
    /// Send a value to all receivers.
    ///
    /// # Errors
    ///
    /// Returns an error if there are no receivers.
    pub fn send(&self, value: T) -> Result<usize, SendError<T>> {
        match self.inner.send(value) {
            Ok(n) => {
                self.metrics.record_send(None);
                Ok(n)
            }
            Err(tokio_broadcast::error::SendError(value)) => {
                self.metrics.mark_closed();
                Err(SendError(value))
            }
        }
    }

    /// Create a new receiver subscribed to this sender.
    #[must_use]
    pub fn subscribe(&self) -> Receiver<T> {
        Receiver {
            inner: self.inner.subscribe(),
            metrics: self.metrics.clone(),
            name: self.name.clone(),
        }
    }

    /// Get the number of active receivers.
    #[must_use]
    pub fn receiver_count(&self) -> usize {
        self.inner.receiver_count()
    }

    /// Get the channel capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the channel name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current metrics for this channel.
    #[must_use]
    pub fn metrics(&self) -> ChannelMetrics {
        self.metrics.get_metrics(0)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            metrics: self.metrics.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
        }
    }
}

impl<T: Clone> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("broadcast::Sender")
            .field("name", &self.name)
            .field("capacity", &self.capacity)
            .field("receivers", &self.receiver_count())
            .finish()
    }
}

/// Tracked receiver half of a broadcast channel.
pub struct Receiver<T> {
    inner: tokio_broadcast::Receiver<T>,
    metrics: Arc<ChannelMetricsTracker>,
    name: Arc<String>,
}

impl<T: Clone> Receiver<T> {
    /// Receive a value, waiting if necessary.
    pub async fn recv(&mut self) -> Result<T, RecvError> {
        let timer = WaitTimer::start();

        match self.inner.recv().await {
            Ok(value) => {
                let wait_time = timer.elapsed_if_waited();
                self.metrics.record_recv(wait_time);
                Ok(value)
            }
            Err(tokio_broadcast::error::RecvError::Closed) => {
                self.metrics.mark_closed();
                Err(RecvError::Closed)
            }
            Err(tokio_broadcast::error::RecvError::Lagged(n)) => Err(RecvError::Lagged(n)),
        }
    }

    /// Try to receive without waiting.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        match self.inner.try_recv() {
            Ok(value) => {
                self.metrics.record_recv(None);
                Ok(value)
            }
            Err(tokio_broadcast::error::TryRecvError::Empty) => Err(TryRecvError::Empty),
            Err(tokio_broadcast::error::TryRecvError::Closed) => {
                self.metrics.mark_closed();
                Err(TryRecvError::Closed)
            }
            Err(tokio_broadcast::error::TryRecvError::Lagged(n)) => Err(TryRecvError::Lagged(n)),
        }
    }

    /// Get the channel name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current metrics for this channel.
    #[must_use]
    pub fn metrics(&self) -> ChannelMetrics {
        self.metrics.get_metrics(0)
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("broadcast::Receiver")
            .field("name", &self.name)
            .finish()
    }
}

/// Error returned when sending fails.
#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "channel closed (no receivers)")
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

/// Error returned when receiving fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecvError {
    /// The channel is closed.
    Closed,
    /// The receiver lagged too far behind (missed messages).
    Lagged(u64),
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecvError::Closed => write!(f, "channel closed"),
            RecvError::Lagged(n) => write!(f, "receiver lagged, missed {n} messages"),
        }
    }
}

impl std::error::Error for RecvError {}

/// Error returned when try_recv fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecvError {
    /// The channel is empty.
    Empty,
    /// The channel is closed.
    Closed,
    /// The receiver lagged too far behind.
    Lagged(u64),
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryRecvError::Empty => write!(f, "channel empty"),
            TryRecvError::Closed => write!(f, "channel closed"),
            TryRecvError::Lagged(n) => write!(f, "receiver lagged, missed {n} messages"),
        }
    }
}

impl std::error::Error for TryRecvError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcast_basic() {
        let (tx, mut rx1) = channel::<i32>(16, "test");
        let mut rx2 = tx.subscribe();

        tx.send(42).unwrap();

        assert_eq!(rx1.recv().await.unwrap(), 42);
        assert_eq!(rx2.recv().await.unwrap(), 42);

        let metrics = tx.metrics();
        assert_eq!(metrics.sent, 1);
    }

    #[tokio::test]
    async fn test_broadcast_multiple_sends() {
        let (tx, mut rx) = channel::<i32>(16, "test");

        tx.send(1).unwrap();
        tx.send(2).unwrap();
        tx.send(3).unwrap();

        assert_eq!(rx.recv().await.unwrap(), 1);
        assert_eq!(rx.recv().await.unwrap(), 2);
        assert_eq!(rx.recv().await.unwrap(), 3);

        let metrics = rx.metrics();
        assert_eq!(metrics.received, 3);
    }

    #[tokio::test]
    async fn test_broadcast_receiver_count() {
        let (tx, _rx1) = channel::<i32>(16, "test");
        assert_eq!(tx.receiver_count(), 1);

        let _rx2 = tx.subscribe();
        assert_eq!(tx.receiver_count(), 2);

        let _rx3 = tx.subscribe();
        assert_eq!(tx.receiver_count(), 3);
    }

    #[tokio::test]
    async fn test_broadcast_no_receivers() {
        let (tx, rx) = channel::<i32>(16, "test");
        drop(rx);

        assert!(tx.send(42).is_err());
    }
}
