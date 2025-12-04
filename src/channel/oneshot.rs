//! Tracked oneshot channel
//!
//! A drop-in replacement for `tokio::sync::oneshot` that tracks message flow.

use crate::channel::{ChannelMetrics, ChannelMetricsTracker};
use std::fmt;
use std::sync::Arc;
use tokio::sync::oneshot as tokio_oneshot;

/// Create a tracked oneshot channel.
///
/// # Arguments
///
/// * `name` - A descriptive name for debugging and metrics
///
/// # Example
///
/// ```rust,no_run
/// use async_inspect::channel::oneshot;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, rx) = oneshot::channel::<String>("result");
///
///     tokio::spawn(async move {
///         tx.send("done".into()).unwrap();
///     });
///
///     let result = rx.await.unwrap();
///     println!("Result: {}", result);
/// }
/// ```
pub fn channel<T>(name: impl Into<String>) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = tokio_oneshot::channel();
    let metrics = Arc::new(ChannelMetricsTracker::new());
    let name = Arc::new(name.into());

    (
        Sender {
            inner: Some(tx),
            metrics: metrics.clone(),
            name: name.clone(),
        },
        Receiver {
            inner: Some(rx),
            metrics,
            name,
        },
    )
}

/// Tracked sender half of a oneshot channel.
pub struct Sender<T> {
    inner: Option<tokio_oneshot::Sender<T>>,
    metrics: Arc<ChannelMetricsTracker>,
    name: Arc<String>,
}

impl<T> Sender<T> {
    /// Send a value.
    ///
    /// # Errors
    ///
    /// Returns the value if the receiver was dropped.
    pub fn send(mut self, value: T) -> Result<(), T> {
        if let Some(tx) = self.inner.take() {
            match tx.send(value) {
                Ok(()) => {
                    self.metrics.record_send(None);
                    Ok(())
                }
                Err(value) => {
                    self.metrics.mark_closed();
                    Err(value)
                }
            }
        } else {
            Err(value)
        }
    }

    /// Check if the receiver has been dropped.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.inner.as_ref().map_or(true, |tx| tx.is_closed())
    }

    /// Get the channel name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("oneshot::Sender")
            .field("name", &self.name)
            .finish()
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.inner.is_some() {
            // Sender dropped without sending
            self.metrics.mark_closed();
        }
    }
}

/// Tracked receiver half of a oneshot channel.
pub struct Receiver<T> {
    inner: Option<tokio_oneshot::Receiver<T>>,
    metrics: Arc<ChannelMetricsTracker>,
    name: Arc<String>,
}

impl<T> Receiver<T> {
    /// Try to receive the value without waiting.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        if let Some(rx) = self.inner.as_mut() {
            match rx.try_recv() {
                Ok(value) => {
                    self.metrics.record_recv(None);
                    self.inner = None;
                    Ok(value)
                }
                Err(tokio_oneshot::error::TryRecvError::Empty) => Err(TryRecvError::Empty),
                Err(tokio_oneshot::error::TryRecvError::Closed) => {
                    self.metrics.mark_closed();
                    self.inner = None;
                    Err(TryRecvError::Closed)
                }
            }
        } else {
            Err(TryRecvError::Closed)
        }
    }

    /// Close the receiver, notifying the sender.
    pub fn close(&mut self) {
        if let Some(rx) = self.inner.as_mut() {
            rx.close();
            self.metrics.mark_closed();
        }
    }

    /// Get the channel name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get current metrics.
    #[must_use]
    pub fn metrics(&self) -> ChannelMetrics {
        self.metrics.get_metrics(0)
    }
}

impl<T> std::future::Future for Receiver<T> {
    type Output = Result<T, RecvError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(ref mut rx) = self.inner {
            // SAFETY: We're not moving the inner receiver
            let rx = unsafe { std::pin::Pin::new_unchecked(rx) };
            match rx.poll(cx) {
                std::task::Poll::Ready(Ok(value)) => {
                    self.metrics.record_recv(None);
                    self.inner = None;
                    std::task::Poll::Ready(Ok(value))
                }
                std::task::Poll::Ready(Err(_)) => {
                    self.metrics.mark_closed();
                    self.inner = None;
                    std::task::Poll::Ready(Err(RecvError(())))
                }
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        } else {
            std::task::Poll::Ready(Err(RecvError(())))
        }
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("oneshot::Receiver")
            .field("name", &self.name)
            .finish()
    }
}

/// Error returned when receiving fails because the sender was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecvError(());

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "channel closed")
    }
}

impl std::error::Error for RecvError {}

/// Error returned when try_recv fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryRecvError {
    /// The channel is empty (sender hasn't sent yet).
    Empty,
    /// The channel is closed.
    Closed,
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryRecvError::Empty => write!(f, "channel empty"),
            TryRecvError::Closed => write!(f, "channel closed"),
        }
    }
}

impl std::error::Error for TryRecvError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oneshot_success() {
        let (tx, rx) = channel::<i32>("test");

        tx.send(42).unwrap();
        let value = rx.await.unwrap();
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_oneshot_sender_dropped() {
        let (tx, rx) = channel::<i32>("test");
        drop(tx);

        assert!(rx.await.is_err());
    }

    #[tokio::test]
    async fn test_oneshot_receiver_dropped() {
        let (tx, rx) = channel::<i32>("test");
        drop(rx);

        assert!(tx.is_closed());
        assert!(tx.send(42).is_err());
    }

    #[tokio::test]
    async fn test_try_recv() {
        let (tx, mut rx) = channel::<i32>("test");

        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

        tx.send(42).unwrap();
        assert_eq!(rx.try_recv().unwrap(), 42);
    }
}
