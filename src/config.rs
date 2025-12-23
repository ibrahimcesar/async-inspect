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

    // --- Adaptive Sampling Fields ---
    /// Whether adaptive sampling is enabled
    adaptive_sampling_enabled: AtomicUsize,

    /// Minimum sampling rate (never go below this)
    adaptive_min_rate: AtomicUsize,

    /// Maximum sampling rate (never go above this)
    adaptive_max_rate: AtomicUsize,

    /// Target overhead in nanoseconds per second
    adaptive_target_overhead_ns: AtomicU64,

    /// Tasks per second counter for load measurement
    tasks_per_window: AtomicU64,

    /// Window start time (epoch millis, approximated)
    /// Reserved for future time-window based adaptive sampling
    #[allow(dead_code)]
    window_start_ms: AtomicU64,
}

impl Config {
    /// Get the global configuration instance
    #[must_use]
    pub fn global() -> &'static Config {
        &CONFIG
    }

    /// Create a new configuration with default settings
    #[must_use]
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
                // Adaptive sampling defaults
                adaptive_sampling_enabled: AtomicUsize::new(0), // Disabled by default
                adaptive_min_rate: AtomicUsize::new(1),         // Track all when load is low
                adaptive_max_rate: AtomicUsize::new(1000),      // Track 1 in 1000 max
                adaptive_target_overhead_ns: AtomicU64::new(1_000_000), // 1ms overhead target per second
                tasks_per_window: AtomicU64::new(0),
                window_start_ms: AtomicU64::new(0),
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
    #[must_use]
    pub fn sampling_rate(&self) -> usize {
        self.inner.sampling_rate.load(Ordering::Relaxed)
    }

    /// Set maximum number of events to retain
    pub fn set_max_events(&self, max: usize) {
        self.inner.max_events.store(max, Ordering::Relaxed);
    }

    /// Get maximum number of events
    #[must_use]
    pub fn max_events(&self) -> usize {
        self.inner.max_events.load(Ordering::Relaxed)
    }

    /// Set maximum number of tasks to track
    pub fn set_max_tasks(&self, max: usize) {
        self.inner.max_tasks.store(max, Ordering::Relaxed);
    }

    /// Get maximum number of tasks
    #[must_use]
    pub fn max_tasks(&self) -> usize {
        self.inner.max_tasks.load(Ordering::Relaxed)
    }

    /// Enable or disable await tracking
    pub fn set_track_awaits(&self, enabled: bool) {
        self.inner
            .track_awaits
            .store(usize::from(enabled), Ordering::Relaxed);
    }

    /// Check if await tracking is enabled
    #[must_use]
    pub fn track_awaits(&self) -> bool {
        self.inner.track_awaits.load(Ordering::Relaxed) != 0
    }

    /// Enable or disable poll tracking
    pub fn set_track_polls(&self, enabled: bool) {
        self.inner
            .track_polls
            .store(usize::from(enabled), Ordering::Relaxed);
    }

    /// Check if poll tracking is enabled
    #[must_use]
    pub fn track_polls(&self) -> bool {
        self.inner.track_polls.load(Ordering::Relaxed) != 0
    }

    /// Enable or disable HTML report generation
    pub fn set_enable_html(&self, enabled: bool) {
        self.inner
            .enable_html
            .store(usize::from(enabled), Ordering::Relaxed);
    }

    /// Check if HTML reports are enabled
    #[must_use]
    pub fn enable_html(&self) -> bool {
        self.inner.enable_html.load(Ordering::Relaxed) != 0
    }

    /// Decide whether to sample this task
    #[must_use]
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
    #[must_use]
    pub fn total_overhead_ns(&self) -> u64 {
        self.inner.overhead_ns.load(Ordering::Relaxed)
    }

    /// Get total instrumentation calls
    #[must_use]
    pub fn instrumentation_calls(&self) -> u64 {
        self.inner.instrumentation_calls.load(Ordering::Relaxed)
    }

    /// Get average overhead per call in nanoseconds
    #[must_use]
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

    // --- Adaptive Sampling Methods ---

    /// Enable adaptive sampling
    ///
    /// When enabled, the sampling rate automatically adjusts based on load
    /// to maintain a target overhead budget.
    ///
    /// # Example
    ///
    /// ```rust
    /// use async_inspect::config::Config;
    ///
    /// let config = Config::new();
    /// config.enable_adaptive_sampling();
    /// config.set_adaptive_target_overhead_ms(5.0); // 5ms overhead budget per second
    /// ```
    pub fn enable_adaptive_sampling(&self) {
        self.inner
            .adaptive_sampling_enabled
            .store(1, Ordering::Relaxed);
    }

    /// Disable adaptive sampling
    pub fn disable_adaptive_sampling(&self) {
        self.inner
            .adaptive_sampling_enabled
            .store(0, Ordering::Relaxed);
    }

    /// Check if adaptive sampling is enabled
    #[must_use]
    pub fn is_adaptive_sampling_enabled(&self) -> bool {
        self.inner.adaptive_sampling_enabled.load(Ordering::Relaxed) != 0
    }

    /// Set adaptive sampling bounds
    ///
    /// - `min_rate`: Minimum sampling rate (e.g., 1 = track all when load is low)
    /// - `max_rate`: Maximum sampling rate (e.g., 1000 = track 1 in 1000 under heavy load)
    pub fn set_adaptive_bounds(&self, min_rate: usize, max_rate: usize) {
        self.inner
            .adaptive_min_rate
            .store(min_rate.max(1), Ordering::Relaxed);
        self.inner
            .adaptive_max_rate
            .store(max_rate.max(min_rate), Ordering::Relaxed);
    }

    /// Get adaptive sampling bounds
    #[must_use]
    pub fn adaptive_bounds(&self) -> (usize, usize) {
        (
            self.inner.adaptive_min_rate.load(Ordering::Relaxed),
            self.inner.adaptive_max_rate.load(Ordering::Relaxed),
        )
    }

    /// Set target overhead in milliseconds per second
    ///
    /// The adaptive sampler will try to keep instrumentation overhead
    /// below this target by adjusting the sampling rate.
    pub fn set_adaptive_target_overhead_ms(&self, ms: f64) {
        let ns = (ms * 1_000_000.0) as u64;
        self.inner
            .adaptive_target_overhead_ns
            .store(ns, Ordering::Relaxed);
    }

    /// Get target overhead in milliseconds
    #[must_use]
    pub fn adaptive_target_overhead_ms(&self) -> f64 {
        self.inner.adaptive_target_overhead_ns.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    /// Record a task for adaptive sampling measurement
    ///
    /// Call this when a task is spawned to help the adaptive sampler
    /// measure load. Returns whether this task should be sampled.
    #[must_use]
    pub fn adaptive_should_sample(&self) -> bool {
        if !self.is_adaptive_sampling_enabled() {
            return self.should_sample();
        }

        // Increment task counter
        self.inner.tasks_per_window.fetch_add(1, Ordering::Relaxed);

        // Check if we should adjust the sampling rate (every ~1000 tasks)
        let tasks = self.inner.tasks_per_window.load(Ordering::Relaxed);
        if tasks % 1000 == 0 {
            self.adjust_sampling_rate();
        }

        // Use the current (possibly adjusted) sampling rate
        self.should_sample()
    }

    /// Adjust sampling rate based on measured overhead
    fn adjust_sampling_rate(&self) {
        let overhead = self.total_overhead_ns();
        let target = self.inner.adaptive_target_overhead_ns.load(Ordering::Relaxed);
        let current_rate = self.sampling_rate();
        let (min_rate, max_rate) = self.adaptive_bounds();

        // Calculate new rate based on overhead ratio
        let new_rate = if overhead == 0 || target == 0 {
            min_rate
        } else if overhead > target {
            // Overhead is too high, increase sampling rate (sample less)
            let ratio = (overhead as f64 / target as f64).sqrt();
            let proposed = (current_rate as f64 * ratio) as usize;
            proposed.clamp(min_rate, max_rate)
        } else {
            // Overhead is under budget, decrease sampling rate (sample more)
            let ratio = (target as f64 / overhead as f64).sqrt();
            let proposed = (current_rate as f64 / ratio) as usize;
            proposed.clamp(min_rate, max_rate)
        };

        if new_rate != current_rate {
            self.set_sampling_rate(new_rate);
        }

        // Reset overhead counters periodically
        if self.instrumentation_calls() > 10_000 {
            self.reset_overhead();
        }
    }

    /// Get adaptive sampling statistics
    #[must_use]
    pub fn adaptive_stats(&self) -> AdaptiveSamplingStats {
        let (min_rate, max_rate) = self.adaptive_bounds();
        AdaptiveSamplingStats {
            enabled: self.is_adaptive_sampling_enabled(),
            current_rate: self.sampling_rate(),
            min_rate,
            max_rate,
            target_overhead_ms: self.adaptive_target_overhead_ms(),
            actual_overhead_ms: self.total_overhead_ns() as f64 / 1_000_000.0,
            tasks_measured: self.inner.tasks_per_window.load(Ordering::Relaxed),
        }
    }

    /// Get overhead statistics
    #[must_use]
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
    #[must_use]
    pub fn total_ms(&self) -> f64 {
        self.total_ns as f64 / 1_000_000.0
    }

    /// Get average overhead in microseconds
    #[must_use]
    pub fn avg_us(&self) -> f64 {
        self.avg_ns / 1_000.0
    }
}

/// Statistics about adaptive sampling behavior
#[derive(Debug, Clone, Copy)]
pub struct AdaptiveSamplingStats {
    /// Whether adaptive sampling is enabled
    pub enabled: bool,

    /// Current sampling rate
    pub current_rate: usize,

    /// Minimum sampling rate
    pub min_rate: usize,

    /// Maximum sampling rate
    pub max_rate: usize,

    /// Target overhead in milliseconds
    pub target_overhead_ms: f64,

    /// Actual measured overhead in milliseconds
    pub actual_overhead_ms: f64,

    /// Number of tasks measured
    pub tasks_measured: u64,
}

impl AdaptiveSamplingStats {
    /// Check if overhead is within budget
    #[must_use]
    pub fn is_within_budget(&self) -> bool {
        self.actual_overhead_ms <= self.target_overhead_ms
    }

    /// Get the overhead ratio (actual / target)
    #[must_use]
    pub fn overhead_ratio(&self) -> f64 {
        if self.target_overhead_ms == 0.0 {
            0.0
        } else {
            self.actual_overhead_ms / self.target_overhead_ms
        }
    }

    /// Check if the sampler is at minimum rate (tracking all)
    #[must_use]
    pub fn is_at_min_rate(&self) -> bool {
        self.current_rate == self.min_rate
    }

    /// Check if the sampler is at maximum rate (tracking minimum)
    #[must_use]
    pub fn is_at_max_rate(&self) -> bool {
        self.current_rate == self.max_rate
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
        assert!((8..=12).contains(&sampled));
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

    #[test]
    fn test_adaptive_sampling_enable_disable() {
        let config = Config::new();
        assert!(!config.is_adaptive_sampling_enabled());

        config.enable_adaptive_sampling();
        assert!(config.is_adaptive_sampling_enabled());

        config.disable_adaptive_sampling();
        assert!(!config.is_adaptive_sampling_enabled());
    }

    #[test]
    fn test_adaptive_sampling_bounds() {
        let config = Config::new();
        config.set_adaptive_bounds(5, 500);

        let (min, max) = config.adaptive_bounds();
        assert_eq!(min, 5);
        assert_eq!(max, 500);
    }

    #[test]
    fn test_adaptive_target_overhead() {
        let config = Config::new();
        config.set_adaptive_target_overhead_ms(2.5);

        let target = config.adaptive_target_overhead_ms();
        assert!((target - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_adaptive_stats() {
        let config = Config::new();
        config.enable_adaptive_sampling();
        config.set_adaptive_bounds(1, 100);
        config.set_adaptive_target_overhead_ms(5.0);

        let stats = config.adaptive_stats();
        assert!(stats.enabled);
        assert_eq!(stats.min_rate, 1);
        assert_eq!(stats.max_rate, 100);
        assert!((stats.target_overhead_ms - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_adaptive_should_sample_disabled() {
        let config = Config::new();
        config.disable_adaptive_sampling();
        config.set_sampling_rate(1);

        // When disabled, should use normal sampling
        assert!(config.adaptive_should_sample());
    }

    #[test]
    fn test_adaptive_stats_methods() {
        let stats = AdaptiveSamplingStats {
            enabled: true,
            current_rate: 10,
            min_rate: 1,
            max_rate: 100,
            target_overhead_ms: 5.0,
            actual_overhead_ms: 3.0,
            tasks_measured: 1000,
        };

        assert!(stats.is_within_budget());
        assert!((stats.overhead_ratio() - 0.6).abs() < 0.001);
        assert!(!stats.is_at_min_rate());
        assert!(!stats.is_at_max_rate());
    }
}
