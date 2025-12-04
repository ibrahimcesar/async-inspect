//! # async-inspect 🔍
//!
//! > X-ray vision for async Rust

// Allow some clippy lints that are too pedantic for this crate
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::use_self)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::unused_self)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::float_cmp)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::into_iter_on_ref)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::format_push_string)]
//!
//! **async-inspect** visualizes and inspects async state machines in Rust.
//! See exactly what your futures are doing, where they're stuck, and why.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use async_inspect::prelude::*;
//!
//! #[async_inspect::trace]
//! async fn fetch_user(id: u64) -> User {
//!     let profile = fetch_profile(id).await;
//!     let posts = fetch_posts(id).await;
//!     User { profile, posts }
//! }
//! ```
//!
//! ## Features
//!
//! - 🔍 **State Machine Inspection** - See current state and variables
//! - ⏱️ **Execution Timeline** - Visualize async execution
//! - 💀 **Deadlock Detection** - Find circular dependencies
//! - 📊 **Performance Analysis** - Identify bottlenecks

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

// Re-export proc macros
pub use async_inspect_macros::{inspect, trace};

/// Production configuration
pub mod config;

/// Core inspection types and traits
pub mod inspector;

/// State machine introspection
///
/// State machine analysis and visualization
pub mod state_machine {}

/// Task tracking and monitoring
pub mod task;

/// Timeline and execution history
pub mod timeline;

/// Deadlock detection
pub mod deadlock;

/// Performance profiling
pub mod profile;

/// Runtime integration hooks
pub mod runtime;

/// Instrumentation and tracing
pub mod instrument;

/// Reporting and output
pub mod reporter;

/// Export functionality
pub mod export;

/// Task relationship graph
pub mod graph;

/// Ecosystem integrations
pub mod integrations;

/// Tracked synchronization primitives
///
/// Drop-in replacements for `tokio::sync::Mutex`, `RwLock`, and `Semaphore`
/// with automatic contention tracking and deadlock detection integration.
pub mod sync;

/// Tracked channel primitives
///
/// Drop-in replacements for `tokio::sync::mpsc`, `oneshot`, and `broadcast`
/// channels with automatic message flow tracking.
pub mod channel;

/// Terminal User Interface
#[cfg(feature = "cli")]
pub mod tui;

/// Web Dashboard for real-time monitoring
#[cfg(feature = "dashboard")]
pub mod dashboard;

/// Language Server Protocol (LSP) integration
#[cfg(feature = "lsp")]
pub mod lsp;

/// Usage telemetry
///
/// Anonymous usage analytics to help improve async-inspect.
/// Can be disabled via `ASYNC_INSPECT_NO_TELEMETRY=1` or `DO_NOT_TRACK=1`.
pub mod telemetry;

/// Error types
///
/// Error definitions
pub mod error {

    use thiserror::Error;

    /// Main error type for async-inspect
    #[derive(Error, Debug)]
    pub enum Error {
        /// Inspection error
        #[error("Inspection error: {0}")]
        Inspection(String),

        /// Runtime error
        #[error("Runtime error: {0}")]
        Runtime(String),

        /// Serialization error
        #[error("Serialization error: {0}")]
        Serialization(#[from] serde_json::Error),

        /// IO error
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
    }

    /// Result type alias
    pub type Result<T> = std::result::Result<T, Error>;
}

/// Prelude for convenient imports
///
/// Convenient re-exports
///
/// ```rust
/// use async_inspect::prelude::*;
/// ```
pub mod prelude {

    pub use crate::error::{Error, Result};
    pub use crate::inspector::{Inspector, InspectorStats};
    pub use crate::instrument::{InspectContext, TaskGuard};
    pub use crate::reporter::html::HtmlReporter;
    pub use crate::reporter::Reporter;
    pub use crate::sync::{LockMetrics, Mutex, MutexGuard, RwLock, Semaphore};
    pub use crate::task::{
        sort_tasks, SortDirection, TaskFilter, TaskId, TaskInfo, TaskSortBy, TaskState,
    };
    pub use crate::timeline::{Event, EventKind};

    // Re-export macros
    pub use crate::{
        inspect_point, inspect_task_complete, inspect_task_failed, inspect_task_start,
    };
}

// Re-exports
pub use error::{Error, Result};
pub use inspector::Inspector;

#[cfg(test)]
mod tests {

    #[test]
    fn test_placeholder() {
        // Placeholder test
        assert_eq!(2 + 2, 4);
    }
}
