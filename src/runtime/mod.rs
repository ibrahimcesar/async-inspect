//! Runtime integration hooks
//!
//! This module provides integration with async runtimes like Tokio,
//! async-std, and smol, enabling automatic tracking of spawned tasks.

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "async-std-runtime")]
pub mod async_std;

#[cfg(feature = "smol-runtime")]
pub mod smol;
