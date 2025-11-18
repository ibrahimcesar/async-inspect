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

/// Core inspection types and traits
pub mod inspector {
    //! Core inspection functionality
}

/// State machine introspection
pub mod state_machine {
    //! State machine analysis and visualization
}

/// Task tracking and monitoring
pub mod task {
    //! Task lifecycle tracking
}

/// Timeline and execution history
pub mod timeline {
    //! Execution timeline tracking
}

/// Deadlock detection
pub mod deadlock {
    //! Deadlock detection and analysis
}

/// Performance profiling
pub mod profile {
    //! Performance analysis tools
}

/// Runtime integration hooks
pub mod runtime {
    //! Integration with async runtimes
    
    #[cfg(feature = "tokio")]
    pub mod tokio {
        //! Tokio runtime integration
    }
}

/// Instrumentation and tracing
pub mod instrument {
    //! Code instrumentation utilities
}

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
    
    pub use crate::inspector::*;
    pub use crate::state_machine::*;
    pub use crate::task::*;
    pub use crate::error::{Error, Result};
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
