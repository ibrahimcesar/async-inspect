//! Runtime integration hooks
//!
//! This module provides integration with async runtimes like Tokio,
//! enabling automatic tracking of spawned tasks.

#[cfg(feature = "tokio")]
pub mod tokio;
