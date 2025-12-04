//! Tracked MPSC (Multi-Producer Single-Consumer) channel
//!
//! A drop-in replacement for `tokio::sync::mpsc` that automatically tracks
//! message flow and integrates with async-inspect's visualization.

use crate::channel::{ChannelMetrics, ChannelMetricsTracker, WaitTimer};
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc as tokio_mpsc;

/// Create a bounded tracked mpsc channel.
///
/// # Arguments
///
/// * `capacity` - Maximum number of messages the channel can hold
/// * `name` - A descriptive name for debugging and metrics
///
/// # Example
///
/// ```rust,no_run
/// use async_inspect::channel::mpsc;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, mut rx) = mpsc::channel::<i32>(100, "my_channel");
///
///     tx.send(42).await.unwrap();
///     let value = rx.recv().await.unwrap();
///     assert_eq!(value, 42);
/// }
/// ```
pub fn channel<T>(capacity: usize, name: impl Into<String>) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = tokio_mpsc::channel(capacity);
    let metrics = Arc::new(ChannelMetricsTracker::new());
    let name = Arc::new(name.into());
    let capacity = capacity;

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
            capacity,
        },
    )
}

/// Create an unbounded tracked mpsc channel.
///
/// # Arguments
///
/// * `name` - A descriptive name for debugging and metrics
///
/// # Example
///
/// ```rust,no_run
/// use async_inspect::channel::mpsc;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, mut rx) = mpsc::unbounded_channel::<String>("events");
///
///     tx.send("event1".into()).unwrap();
///     let event = rx.recv().await.unwrap();
/// }
/// ```
pub fn unbounded_channel<T>(name: impl Into<String>) -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = tokio_mpsc::unbounded_channel();
    let metrics = Arc::new(ChannelMetricsTracker::new());
    let name = Arc::new(name.into());

    (
        UnboundedSender {
            inner: tx,
            metrics: metrics.clone(),
            name: name.clone(),
        },
        UnboundedReceiver {
            inner: rx,
            metrics,
            name,
        },
    )
}

/// Tracked bounded sender half of an mpsc channel.
pub struct Sender<T> {
    inner: tokio_mpsc::Sender<T>,
    metrics: Arc<ChannelMetricsTracker>,
    name: Arc<String>,
    capacity: usize,
}

impl<T> Sender<T> {
    /// Send a value, waiting if the channel is full.
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver has been dropped.
    pub async fn send(&self, value: T) -> Result<(), SendError<T>> {
        let timer = WaitTimer::start();

        match self.inner.send(value).await {
            Ok(()) => {
                let wait_time = timer.elapsed_if_waited();
                self.metrics.record_send(wait_time);
                Ok(())
            }
            Err(tokio_mpsc::error::SendError(value)) => {
                self.metrics.mark_closed();
                Err(SendError(value))
            }
        }
    }

    /// Try to send a value without waiting.
    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        match self.inner.try_send(value) {
            Ok(()) => {
                self.metrics.record_send(None);
                Ok(())
            }
            Err(tokio_mpsc::error::TrySendError::Full(value)) => Err(TrySendError::Full(value)),
            Err(tokio_mpsc::error::TrySendError::Closed(value)) => {
                self.metrics.mark_closed();
                Err(TrySendError::Closed(value))
            }
        }
    }

    /// Check if the channel is closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Get the channel capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Get the maximum capacity.
    #[must_use]
    pub fn max_capacity(&self) -> usize {
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
        let buffered = (self.capacity - self.inner.capacity()) as u64;
        self.metrics.get_metrics(buffered)
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

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender")
            .field("name", &self.name)
            .field("capacity", &self.capacity)
            .finish()
    }
}

/// Tracked bounded receiver half of an mpsc channel.
pub struct Receiver<T> {
    inner: tokio_mpsc::Receiver<T>,
    metrics: Arc<ChannelMetricsTracker>,
    name: Arc<String>,
    capacity: usize,
}

impl<T> Receiver<T> {
    /// Receive a value, waiting if the channel is empty.
    ///
    /// Returns `None` if the channel is closed and empty.
    pub async fn recv(&mut self) -> Option<T> {
        let timer = WaitTimer::start();

        match self.inner.recv().await {
            Some(value) => {
                let wait_time = timer.elapsed_if_waited();
                self.metrics.record_recv(wait_time);
                Some(value)
            }
            None => {
                self.metrics.mark_closed();
                None
            }
        }
    }

    /// Try to receive a value without waiting.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        match self.inner.try_recv() {
            Ok(value) => {
                self.metrics.record_recv(None);
                Ok(value)
            }
            Err(tokio_mpsc::error::TryRecvError::Empty) => Err(TryRecvError::Empty),
            Err(tokio_mpsc::error::TryRecvError::Disconnected) => {
                self.metrics.mark_closed();
                Err(TryRecvError::Disconnected)
            }
        }
    }

    /// Close the receiver, preventing any new messages.
    pub fn close(&mut self) {
        self.inner.close();
        self.metrics.mark_closed();
    }

    /// Get the channel name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current metrics for this channel.
    #[must_use]
    pub fn metrics(&self) -> ChannelMetrics {
        // Approximate buffered count
        let sent = self.metrics.sent.load(std::sync::atomic::Ordering::Relaxed);
        let received = self
            .metrics
            .received
            .load(std::sync::atomic::Ordering::Relaxed);
        let buffered = sent.saturating_sub(received);
        self.metrics.get_metrics(buffered)
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver")
            .field("name", &self.name)
            .field("capacity", &self.capacity)
            .finish()
    }
}

/// Tracked unbounded sender half of an mpsc channel.
pub struct UnboundedSender<T> {
    inner: tokio_mpsc::UnboundedSender<T>,
    metrics: Arc<ChannelMetricsTracker>,
    name: Arc<String>,
}

impl<T> UnboundedSender<T> {
    /// Send a value (never blocks for unbounded channels).
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver has been dropped.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        match self.inner.send(value) {
            Ok(()) => {
                self.metrics.record_send(None);
                Ok(())
            }
            Err(tokio_mpsc::error::SendError(value)) => {
                self.metrics.mark_closed();
                Err(SendError(value))
            }
        }
    }

    /// Check if the channel is closed.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Get the channel name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current metrics for this channel.
    #[must_use]
    pub fn metrics(&self) -> ChannelMetrics {
        let sent = self.metrics.sent.load(std::sync::atomic::Ordering::Relaxed);
        let received = self
            .metrics
            .received
            .load(std::sync::atomic::Ordering::Relaxed);
        let buffered = sent.saturating_sub(received);
        self.metrics.get_metrics(buffered)
    }
}

impl<T> Clone for UnboundedSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            metrics: self.metrics.clone(),
            name: self.name.clone(),
        }
    }
}

impl<T> fmt::Debug for UnboundedSender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnboundedSender")
            .field("name", &self.name)
            .finish()
    }
}

/// Tracked unbounded receiver half of an mpsc channel.
pub struct UnboundedReceiver<T> {
    inner: tokio_mpsc::UnboundedReceiver<T>,
    metrics: Arc<ChannelMetricsTracker>,
    name: Arc<String>,
}

impl<T> UnboundedReceiver<T> {
    /// Receive a value, waiting if the channel is empty.
    pub async fn recv(&mut self) -> Option<T> {
        let timer = WaitTimer::start();

        match self.inner.recv().await {
            Some(value) => {
                let wait_time = timer.elapsed_if_waited();
                self.metrics.record_recv(wait_time);
                Some(value)
            }
            None => {
                self.metrics.mark_closed();
                None
            }
        }
    }

    /// Try to receive a value without waiting.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        match self.inner.try_recv() {
            Ok(value) => {
                self.metrics.record_recv(None);
                Ok(value)
            }
            Err(tokio_mpsc::error::TryRecvError::Empty) => Err(TryRecvError::Empty),
            Err(tokio_mpsc::error::TryRecvError::Disconnected) => {
                self.metrics.mark_closed();
                Err(TryRecvError::Disconnected)
            }
        }
    }

    /// Close the receiver.
    pub fn close(&mut self) {
        self.inner.close();
        self.metrics.mark_closed();
    }

    /// Get the channel name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current metrics for this channel.
    #[must_use]
    pub fn metrics(&self) -> ChannelMetrics {
        let sent = self.metrics.sent.load(std::sync::atomic::Ordering::Relaxed);
        let received = self
            .metrics
            .received
            .load(std::sync::atomic::Ordering::Relaxed);
        let buffered = sent.saturating_sub(received);
        self.metrics.get_metrics(buffered)
    }
}

impl<T> fmt::Debug for UnboundedReceiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnboundedReceiver")
            .field("name", &self.name)
            .finish()
    }
}

/// Error returned when sending fails because the receiver was dropped.
#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "channel closed")
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

/// Error returned when try_send fails.
#[derive(Debug)]
pub enum TrySendError<T> {
    /// Channel is full.
    Full(T),
    /// Channel is closed.
    Closed(T),
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrySendError::Full(_) => write!(f, "channel full"),
            TrySendError::Closed(_) => write!(f, "channel closed"),
        }
    }
}

impl<T: fmt::Debug> std::error::Error for TrySendError<T> {}

/// Error returned when try_recv fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecvError {
    /// Channel is empty.
    Empty,
    /// Channel is disconnected.
    Disconnected,
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryRecvError::Empty => write!(f, "channel empty"),
            TryRecvError::Disconnected => write!(f, "channel disconnected"),
        }
    }
}

impl std::error::Error for TryRecvError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bounded_channel() {
        let (tx, mut rx) = channel::<i32>(10, "test");

        tx.send(42).await.unwrap();
        tx.send(43).await.unwrap();

        assert_eq!(rx.recv().await, Some(42));
        assert_eq!(rx.recv().await, Some(43));

        let metrics = rx.metrics();
        assert_eq!(metrics.sent, 2);
        assert_eq!(metrics.received, 2);
    }

    #[tokio::test]
    async fn test_unbounded_channel() {
        let (tx, mut rx) = unbounded_channel::<String>("events");

        tx.send("hello".into()).unwrap();
        tx.send("world".into()).unwrap();

        assert_eq!(rx.recv().await, Some("hello".into()));
        assert_eq!(rx.recv().await, Some("world".into()));

        let metrics = rx.metrics();
        assert_eq!(metrics.sent, 2);
        assert_eq!(metrics.received, 2);
    }

    #[tokio::test]
    async fn test_channel_close() {
        let (tx, mut rx) = channel::<i32>(10, "test");

        tx.send(1).await.unwrap();
        drop(tx);

        assert_eq!(rx.recv().await, Some(1));
        assert_eq!(rx.recv().await, None);

        let metrics = rx.metrics();
        assert!(metrics.closed);
    }

    #[tokio::test]
    async fn test_try_send_recv() {
        let (tx, mut rx) = channel::<i32>(2, "test");

        tx.try_send(1).unwrap();
        tx.try_send(2).unwrap();

        // Channel full
        assert!(matches!(tx.try_send(3), Err(TrySendError::Full(3))));

        assert_eq!(rx.try_recv().unwrap(), 1);
        assert_eq!(rx.try_recv().unwrap(), 2);
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));
    }
}
