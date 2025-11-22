//! # async-inspect 🔍
//!
//! > X-ray vision for async Rust
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
#![allow(clippy::module_name_repetitions)]

// Re-export proc macros
pub use async_inspect_macros::{inspect, trace};

/// Production configuration
pub mod config;

/// Core inspection types and traits
pub mod inspector;

/// State machine introspection
pub mod state_machine {
    //! State machine analysis and visualization
}

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

/// Terminal User Interface
#[cfg(feature = "cli")]
pub mod tui;

/// Error types
pub mod error {
    //! Error definitions

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
pub mod prelude {
    //! Convenient re-exports
    //!
    //! ```rust
    //! use async_inspect::prelude::*;
    //! ```

    pub use crate::error::{Error, Result};
    pub use crate::inspector::{Inspector, InspectorStats};
    pub use crate::instrument::{InspectContext, TaskGuard};
    pub use crate::reporter::html::HtmlReporter;
    pub use crate::reporter::Reporter;
    pub use crate::task::{TaskId, TaskInfo, TaskState};
    pub use crate::timeline::{Event, EventKind};

    // Re-export macros
    pub use crate::{
        inspect_point, inspect_task_complete, inspect_task_failed, inspect_task_start,
    };
}

// Re-exports
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        // Placeholder test
        assert_eq!(2 + 2, 4);
    }
}
