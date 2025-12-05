//! Tracked channel primitives
//!
//! This module provides drop-in replacements for Tokio's channel types
//! (`mpsc`, `oneshot`, `broadcast`) that automatically track message flow
//! and integrate with async-inspect's visualization.
//!
//! # Example
//!
//! ```rust,no_run
//! use async_inspect::channel::mpsc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (tx, mut rx) = mpsc::channel(32, "commands");
//!
//!     tokio::spawn(async move {
//!         tx.send("hello").await.unwrap();
//!     });
//!
//!     let msg = rx.recv().await.unwrap();
//!     println!("Received: {}", msg);
//!
//!     // Check channel metrics
//!     let metrics = rx.metrics();
//!     println!("Messages received: {}", metrics.received);
//! }
//! ```

pub mod broadcast;
pub mod mpsc;
pub mod oneshot;

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Metrics for a tracked channel
#[derive(Debug, Clone, Default)]
pub struct ChannelMetrics {
    /// Total messages sent
    pub sent: u64,
    /// Total messages received
    pub received: u64,
    /// Messages currently buffered (approximate)
    pub buffered: u64,
    /// Number of times send blocked waiting for capacity
    pub send_waits: u64,
    /// Number of times recv blocked waiting for messages
    pub recv_waits: u64,
    /// Total time spent waiting to send
    pub total_send_wait_time: Duration,
    /// Total time spent waiting to receive
    pub total_recv_wait_time: Duration,
    /// Channel closed
    pub closed: bool,
}

impl ChannelMetrics {
    /// Calculate the send utilization (how often sends block)
    #[must_use]
    pub fn send_block_rate(&self) -> f64 {
        if self.sent == 0 {
            0.0
        } else {
            self.send_waits as f64 / self.sent as f64
        }
    }

    /// Calculate the receive utilization (how often recvs block)
    #[must_use]
    pub fn recv_block_rate(&self) -> f64 {
        if self.received == 0 {
            0.0
        } else {
            self.recv_waits as f64 / self.received as f64
        }
    }
}

/// Internal metrics tracker for channels
#[derive(Debug)]
pub(crate) struct ChannelMetricsTracker {
    sent: AtomicU64,
    received: AtomicU64,
    send_waits: AtomicU64,
    recv_waits: AtomicU64,
    total_send_wait_nanos: AtomicU64,
    total_recv_wait_nanos: AtomicU64,
    closed: std::sync::atomic::AtomicBool,
}

impl ChannelMetricsTracker {
    pub fn new() -> Self {
        Self {
            sent: AtomicU64::new(0),
            received: AtomicU64::new(0),
            send_waits: AtomicU64::new(0),
            recv_waits: AtomicU64::new(0),
            total_send_wait_nanos: AtomicU64::new(0),
            total_recv_wait_nanos: AtomicU64::new(0),
            closed: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn record_send(&self, wait_time: Option<Duration>) {
        self.sent.fetch_add(1, Ordering::Relaxed);
        if let Some(wait) = wait_time {
            self.send_waits.fetch_add(1, Ordering::Relaxed);
            self.total_send_wait_nanos
                .fetch_add(wait.as_nanos() as u64, Ordering::Relaxed);
        }
    }

    pub fn record_recv(&self, wait_time: Option<Duration>) {
        self.received.fetch_add(1, Ordering::Relaxed);
        if let Some(wait) = wait_time {
            self.recv_waits.fetch_add(1, Ordering::Relaxed);
            self.total_recv_wait_nanos
                .fetch_add(wait.as_nanos() as u64, Ordering::Relaxed);
        }
    }

    pub fn mark_closed(&self) {
        self.closed.store(true, Ordering::Relaxed);
    }

    pub fn get_metrics(&self, buffered: u64) -> ChannelMetrics {
        ChannelMetrics {
            sent: self.sent.load(Ordering::Relaxed),
            received: self.received.load(Ordering::Relaxed),
            buffered,
            send_waits: self.send_waits.load(Ordering::Relaxed),
            recv_waits: self.recv_waits.load(Ordering::Relaxed),
            total_send_wait_time: Duration::from_nanos(
                self.total_send_wait_nanos.load(Ordering::Relaxed),
            ),
            total_recv_wait_time: Duration::from_nanos(
                self.total_recv_wait_nanos.load(Ordering::Relaxed),
            ),
            closed: self.closed.load(Ordering::Relaxed),
        }
    }
}

impl Default for ChannelMetricsTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to measure wait time for channel operations
pub(crate) struct WaitTimer {
    start: Instant,
    threshold: Duration,
}

impl WaitTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
            threshold: Duration::from_micros(10),
        }
    }

    pub fn elapsed_if_waited(&self) -> Option<Duration> {
        let elapsed = self.start.elapsed();
        if elapsed > self.threshold {
            Some(elapsed)
        } else {
            None
        }
    }
}
