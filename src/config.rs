//! Production configuration and settings
//!
//! This module provides configuration options for using async-inspect
//! in production environments with minimal overhead.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Global configuration instance
static CONFIG: once_cell::sync::Lazy<Config> = once_cell::sync::Lazy::new(Config::default);

/// Production configuration for async-inspect
#[derive(Clone)]
pub struct Config {
    inner: Arc<ConfigInner>,
}

struct ConfigInner {
    /// Sampling rate: track 1 in N tasks (1 = track all)
    sampling_rate: AtomicUsize,

    /// Maximum number of events to retain (0 = unlimited)
    max_events: AtomicUsize,

    /// Maximum number of tasks to track (0 = unlimited)
    max_tasks: AtomicUsize,

    /// Counter for sampling decisions
    sample_counter: AtomicU64,

    /// Whether to track await points
    track_awaits: AtomicUsize,

    /// Whether to track poll counts
    track_polls: AtomicUsize,

    /// Whether to generate HTML reports
    enable_html: AtomicUsize,

    /// Overhead tracking: total time spent in instrumentation (nanoseconds)
    overhead_ns: AtomicU64,

    /// Number of instrumentation calls
    instrumentation_calls: AtomicU64,
}

impl Config {
    /// Get the global configuration instance
    pub fn global() -> &'static Config {
        &CONFIG
    }

    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ConfigInner {
                sampling_rate: AtomicUsize::new(1),   // Track all tasks by default
                max_events: AtomicUsize::new(10_000), // Default: keep last 10k events
                max_tasks: AtomicUsize::new(1_000),   // Default: track up to 1k tasks
                sample_counter: AtomicU64::new(0),
                track_awaits: AtomicUsize::new(1), // Enabled by default
                track_polls: AtomicUsize::new(1),  // Enabled by default
                enable_html: AtomicUsize::new(1),  // Enabled by default
                overhead_ns: AtomicU64::new(0),
                instrumentation_calls: AtomicU64::new(0),
            }),
        }
    }

    /// Set sampling rate (1 = track all, 10 = track 1 in 10, etc.)
    pub fn set_sampling_rate(&self, rate: usize) {
        self.inner
            .sampling_rate
            .store(rate.max(1), Ordering::Relaxed);
    }

    /// Get current sampling rate
    pub fn sampling_rate(&self) -> usize {
        self.inner.sampling_rate.load(Ordering::Relaxed)
    }

    /// Set maximum number of events to retain
    pub fn set_max_events(&self, max: usize) {
        self.inner.max_events.store(max, Ordering::Relaxed);
    }

    /// Get maximum number of events
    pub fn max_events(&self) -> usize {
        self.inner.max_events.load(Ordering::Relaxed)
    }

    /// Set maximum number of tasks to track
    pub fn set_max_tasks(&self, max: usize) {
        self.inner.max_tasks.store(max, Ordering::Relaxed);
    }

    /// Get maximum number of tasks
    pub fn max_tasks(&self) -> usize {
        self.inner.max_tasks.load(Ordering::Relaxed)
    }

    /// Enable or disable await tracking
    pub fn set_track_awaits(&self, enabled: bool) {
        self.inner
            .track_awaits
            .store(enabled as usize, Ordering::Relaxed);
    }

    /// Check if await tracking is enabled
    pub fn track_awaits(&self) -> bool {
        self.inner.track_awaits.load(Ordering::Relaxed) != 0
    }

    /// Enable or disable poll tracking
    pub fn set_track_polls(&self, enabled: bool) {
        self.inner
            .track_polls
            .store(enabled as usize, Ordering::Relaxed);
    }

    /// Check if poll tracking is enabled
    pub fn track_polls(&self) -> bool {
        self.inner.track_polls.load(Ordering::Relaxed) != 0
    }

    /// Enable or disable HTML report generation
    pub fn set_enable_html(&self, enabled: bool) {
        self.inner
            .enable_html
            .store(enabled as usize, Ordering::Relaxed);
    }

    /// Check if HTML reports are enabled
    pub fn enable_html(&self) -> bool {
        self.inner.enable_html.load(Ordering::Relaxed) != 0
    }

    /// Decide whether to sample this task
    pub fn should_sample(&self) -> bool {
        let rate = self.sampling_rate();
        if rate <= 1 {
            return true;
        }

        let count = self.inner.sample_counter.fetch_add(1, Ordering::Relaxed);
        count % rate as u64 == 0
    }

    /// Record instrumentation overhead
    pub fn record_overhead(&self, nanos: u64) {
        self.inner.overhead_ns.fetch_add(nanos, Ordering::Relaxed);
        self.inner
            .instrumentation_calls
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Get total overhead in nanoseconds
    pub fn total_overhead_ns(&self) -> u64 {
        self.inner.overhead_ns.load(Ordering::Relaxed)
    }

    /// Get total instrumentation calls
    pub fn instrumentation_calls(&self) -> u64 {
        self.inner.instrumentation_calls.load(Ordering::Relaxed)
    }

    /// Get average overhead per call in nanoseconds
    pub fn avg_overhead_ns(&self) -> f64 {
        let calls = self.instrumentation_calls();
        if calls == 0 {
            return 0.0;
        }
        self.total_overhead_ns() as f64 / calls as f64
    }

    /// Configure for production use (minimal overhead)
    pub fn production_mode(&self) {
        self.set_sampling_rate(100); // Track 1% of tasks
        self.set_max_events(1_000); // Keep only 1k events
        self.set_max_tasks(500); // Track up to 500 tasks
        self.set_track_awaits(false); // Disable detailed await tracking
        self.set_enable_html(false); // Disable HTML generation
    }

    /// Configure for development use (full tracking)
    pub fn development_mode(&self) {
        self.set_sampling_rate(1); // Track all tasks
        self.set_max_events(10_000); // Keep 10k events
        self.set_max_tasks(1_000); // Track up to 1k tasks
        self.set_track_awaits(true); // Enable await tracking
        self.set_enable_html(true); // Enable HTML generation
    }

    /// Configure for debugging (maximum detail)
    pub fn debug_mode(&self) {
        self.set_sampling_rate(1); // Track all tasks
        self.set_max_events(0); // Unlimited events
        self.set_max_tasks(0); // Unlimited tasks
        self.set_track_awaits(true); // Enable await tracking
        self.set_enable_html(true); // Enable HTML generation
    }

    /// Get overhead statistics
    pub fn overhead_stats(&self) -> OverheadStats {
        OverheadStats {
            total_ns: self.total_overhead_ns(),
            calls: self.instrumentation_calls(),
            avg_ns: self.avg_overhead_ns(),
        }
    }

    /// Reset overhead counters
    pub fn reset_overhead(&self) {
        self.inner.overhead_ns.store(0, Ordering::Relaxed);
        self.inner.instrumentation_calls.store(0, Ordering::Relaxed);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Overhead statistics
#[derive(Debug, Clone, Copy)]
pub struct OverheadStats {
    /// Total overhead in nanoseconds
    pub total_ns: u64,

    /// Number of instrumentation calls
    pub calls: u64,

    /// Average overhead per call in nanoseconds
    pub avg_ns: f64,
}

impl OverheadStats {
    /// Get total overhead in milliseconds
    pub fn total_ms(&self) -> f64 {
        self.total_ns as f64 / 1_000_000.0
    }

    /// Get average overhead in microseconds
    pub fn avg_us(&self) -> f64 {
        self.avg_ns / 1_000.0
    }
}

/// Helper macro to measure and record overhead
#[macro_export]
macro_rules! measure_overhead {
    ($expr:expr) => {{
        let start = std::time::Instant::now();
        let result = $expr;
        let elapsed = start.elapsed().as_nanos() as u64;
        $crate::config::Config::global().record_overhead(elapsed);
        result
    }};
}

/// Helper to conditionally execute code only when sampling
#[macro_export]
macro_rules! if_sampled {
    ($body:block) => {
        if $crate::config::Config::global().should_sample() $body
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_rate() {
        let config = Config::new();
        config.set_sampling_rate(10);
        assert_eq!(config.sampling_rate(), 10);

        // Check that sampling works
        let mut sampled = 0;
        for _ in 0..100 {
            if config.should_sample() {
                sampled += 1;
            }
        }
        // Should sample approximately 10 times (1 in 10)
        assert!(sampled >= 8 && sampled <= 12);
    }

    #[test]
    fn test_overhead_tracking() {
        let config = Config::new();
        config.reset_overhead();

        config.record_overhead(1000);
        config.record_overhead(2000);

        let stats = config.overhead_stats();
        assert_eq!(stats.total_ns, 3000);
        assert_eq!(stats.calls, 2);
        assert_eq!(stats.avg_ns, 1500.0);
    }

    #[test]
    fn test_production_mode() {
        let config = Config::new();
        config.production_mode();

        assert_eq!(config.sampling_rate(), 100);
        assert!(!config.track_awaits());
        assert!(!config.enable_html());
    }
}
