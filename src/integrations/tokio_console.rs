//! Tokio Console integration guide
//!
//! This module provides guidance on using async-inspect alongside tokio-console.
//! Since tokio-console requires compile-time instrumentation, both tools can
//! work side-by-side to provide complementary insights.

//! # Using async-inspect with tokio-console
//!
//! ## Setup
//!
//! 1. Add tokio-console to your dependencies:
//! ```toml
//! [dependencies]
//! console-subscriber = "0.2"
//! tokio = { version = "1", features = ["tracing"] }
//! ```
//!
//! 2. Initialize both in your application:
//! ```rust,ignore
//! use async_inspect::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize tokio-console
//!     console_subscriber::init();
//!
//!     // async-inspect works alongside automatically
//!     // Your traced tasks will be visible in both tools
//!
//!     my_async_function().await;
//! }
//! ```
//!
//! ## Complementary Features
//!
//! ### Tokio Console provides:
//! - Real-time task monitoring
//! - Live task tree visualization
//! - Poll times and wait times
//! - Resource tracking
//!
//! ### async-inspect provides:
//! - Historical trace export (JSON/CSV)
//! - Relationship graph analysis
//! - Deadlock detection
//! - Performance profiling
//! - Custom inspection points
//!
//! ## Best Practices
//!
//! 1. **Development**: Use tokio-console for real-time debugging
//! 2. **Production**: Use async-inspect with sampling for minimal overhead
//! 3. **Post-mortem**: Export async-inspect traces for offline analysis
//! 4. **CI/CD**: Use async-inspect exports to detect performance regressions
//!
//! ## Environment Variables
//!
//! Both tools respect standard Rust tracing environment variables:
//!
//! ```bash
//! # Enable tokio-console
//! export RUSTFLAGS="--cfg tokio_unstable"
//!
//! # Set trace level
//! export RUST_LOG=debug
//! ```
//!
//! ## Example Configuration
//!
//! ```rust,ignore
//! use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
//!
//! // Create a layered subscriber with both tokio-console and async-inspect
//! tracing_subscriber::registry()
//!     .with(console_subscriber::spawn())  // tokio-console
//!     .with(async_inspect::integrations::tracing_layer::AsyncInspectLayer::new())
//!     .init();
//! ```

use std::fmt;

/// Configuration for using async-inspect with tokio-console
#[derive(Debug, Clone)]
pub struct ConsoleIntegrationConfig {
    /// Enable tokio-console compatibility mode
    pub console_compatible: bool,
    /// Filter tasks to reduce noise in console
    pub filter_short_tasks: bool,
    /// Minimum task duration to show (ms)
    pub min_duration_ms: u64,
}

impl Default for ConsoleIntegrationConfig {
    fn default() -> Self {
        Self {
            console_compatible: true,
            filter_short_tasks: true,
            min_duration_ms: 10,
        }
    }
}

impl fmt::Display for ConsoleIntegrationConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConsoleIntegration {{ compatible: {}, filter: {}, min_ms: {} }}",
            self.console_compatible, self.filter_short_tasks, self.min_duration_ms
        )
    }
}

/// Check if tokio-console is likely active
pub fn is_console_active() -> bool {
    std::env::var("RUSTFLAGS")
        .map(|flags| flags.contains("tokio_unstable"))
        .unwrap_or(false)
}

/// Print integration status and recommendations
pub fn print_integration_info() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  async-inspect + tokio-console Integration                ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    if is_console_active() {
        println!("✅ tokio-console appears to be active");
        println!("   Both tools will work together seamlessly!\n");
    } else {
        println!("ℹ️  tokio-console not detected");
        println!("   To enable: RUSTFLAGS=\"--cfg tokio_unstable\" cargo run\n");
    }

    println!("📊 Tool Comparison:");
    println!("┌─────────────────────┬──────────────┬──────────────┐");
    println!("│ Feature             │ tokio-console│ async-inspect│");
    println!("├─────────────────────┼──────────────┼──────────────┤");
    println!("│ Real-time monitoring│      ✅      │      ✅      │");
    println!("│ Historical export   │      ❌      │      ✅      │");
    println!("│ Graph analysis      │      ❌      │      ✅      │");
    println!("│ Deadlock detection  │      ❌      │      ✅      │");
    println!("│ Production ready    │      ❌      │      ✅      │");
    println!("│ Zero overhead       │      ❌      │   ✅ (opt)   │");
    println!("└─────────────────────┴──────────────┴──────────────┘\n");

    println!("💡 Recommendations:");
    println!("   • Development: Use tokio-console for live debugging");
    println!("   • Production:  Use async-inspect with sampling");
    println!("   • CI/CD:       Compare async-inspect exports");
    println!("   • Post-mortem: Analyze exported JSON/CSV traces\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = ConsoleIntegrationConfig::default();
        assert!(config.console_compatible);
    }
}
