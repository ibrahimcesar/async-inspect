//! Ecosystem integrations for async-inspect
//!
//! This module provides integrations with popular Rust async ecosystem tools
//! including tracing, Prometheus, OpenTelemetry, and more.

/// Tracing subscriber integration
#[cfg(feature = "tracing-sub")]
pub mod tracing_layer;

/// Prometheus metrics exporter
#[cfg(feature = "prometheus-export")]
pub mod prometheus;

/// OpenTelemetry exporter
#[cfg(feature = "opentelemetry-export")]
pub mod opentelemetry;

/// Tokio-console integration guide
pub mod tokio_console;
