//! Usage telemetry for async-inspect
//!
//! This module provides anonymous usage telemetry to help improve async-inspect.
//! Telemetry is privacy-focused and can be disabled via:
//! - Environment variable: `ASYNC_INSPECT_NO_TELEMETRY=1`
//! - Environment variable: `DO_NOT_TRACK=1`
//! - Compile-time: `--no-default-features` or exclude `telemetry` feature
//!
//! We collect anonymous usage data to understand:
//! - Which features are most used
//! - Common usage patterns
//! - Performance characteristics in real-world scenarios
//!
//! No personal information, code, or sensitive data is ever collected.

use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};

static TELEMETRY: OnceCell<Telemetry> = OnceCell::new();
static TELEMETRY_DISABLED: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "telemetry")]
static OPT_OUT_TRACKED: AtomicBool = AtomicBool::new(false);

/// Telemetry wrapper
pub struct Telemetry {
    #[cfg(feature = "telemetry")]
    inner: Option<telemetry_kit::TelemetryKit>,
    #[cfg(not(feature = "telemetry"))]
    _phantom: std::marker::PhantomData<()>,
}

impl Telemetry {
    /// Check if telemetry is enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        #[cfg(feature = "telemetry")]
        {
            self.inner.is_some() && !TELEMETRY_DISABLED.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "telemetry"))]
        {
            false
        }
    }
}

#[cfg(feature = "telemetry")]
fn create_telemetry_kit() -> Option<telemetry_kit::TelemetryKit> {
    telemetry_kit::TelemetryKit::builder()
        .service_name("async-inspect")
        .ok()
        .and_then(|b| b.service_version(env!("CARGO_PKG_VERSION")).build().ok())
}

/// Initialize telemetry (call once at startup)
///
/// Respects `ASYNC_INSPECT_NO_TELEMETRY=1` and `DO_NOT_TRACK=1` environment variables.
pub fn init() {
    let _ = TELEMETRY.get_or_init(|| {
        // Check for opt-out environment variables
        let no_telemetry = std::env::var("ASYNC_INSPECT_NO_TELEMETRY")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let do_not_track = std::env::var("DO_NOT_TRACK")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        if no_telemetry || do_not_track {
            TELEMETRY_DISABLED.store(true, Ordering::Relaxed);

            // Track that someone opted out (this helps us understand opt-out rates)
            // This is a one-time anonymous ping with no identifying information
            #[cfg(feature = "telemetry")]
            if !OPT_OUT_TRACKED.swap(true, Ordering::Relaxed) {
                // Spawn a background task to send opt-out signal
                std::thread::spawn(|| {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build();
                    if let Ok(rt) = rt {
                        rt.block_on(async {
                            if let Some(tk) = create_telemetry_kit() {
                                let _ = tk
                                    .track_feature("telemetry_opt_out", |event| event.success(true))
                                    .await;
                            }
                        });
                    }
                });
            }

            return Telemetry {
                #[cfg(feature = "telemetry")]
                inner: None,
                #[cfg(not(feature = "telemetry"))]
                _phantom: std::marker::PhantomData,
            };
        }

        #[cfg(feature = "telemetry")]
        {
            Telemetry {
                inner: create_telemetry_kit(),
            }
        }

        #[cfg(not(feature = "telemetry"))]
        {
            Telemetry {
                _phantom: std::marker::PhantomData,
            }
        }
    });
}

/// Get the global telemetry instance
#[cfg(feature = "telemetry")]
fn get() -> Option<&'static Telemetry> {
    TELEMETRY.get()
}

/// Track a command/CLI invocation
#[allow(unused_variables)]
pub async fn track_command(command: &str, success: bool, duration_ms: Option<u64>) {
    #[cfg(feature = "telemetry")]
    {
        if let Some(telemetry) = get() {
            if let Some(ref tk) = telemetry.inner {
                let _ = tk
                    .track_command(command, |event| {
                        let event = event.success(success);
                        if let Some(ms) = duration_ms {
                            event.duration_ms(ms)
                        } else {
                            event
                        }
                    })
                    .await;
            }
        }
    }
}

/// Track a feature usage
#[allow(unused_variables)]
pub async fn track_feature(feature: &str, metadata: Option<&str>) {
    #[cfg(feature = "telemetry")]
    {
        if let Some(telemetry) = get() {
            if let Some(ref tk) = telemetry.inner {
                let _ = tk
                    .track_feature(feature, |event| {
                        let event = event.success(true);
                        if let Some(meta) = metadata {
                            event.method(meta)
                        } else {
                            event
                        }
                    })
                    .await;
            }
        }
    }
}

/// Track feature usage synchronously (spawns background task)
#[allow(unused_variables)]
pub fn track_feature_sync(feature: &'static str, metadata: Option<&'static str>) {
    #[cfg(feature = "telemetry")]
    {
        if TELEMETRY_DISABLED.load(Ordering::Relaxed) {
            return;
        }
        tokio::spawn(async move {
            track_feature(feature, metadata).await;
        });
    }
}

/// Track command usage synchronously (spawns background task)
#[allow(unused_variables)]
pub fn track_command_sync(command: &'static str, success: bool, duration_ms: Option<u64>) {
    #[cfg(feature = "telemetry")]
    {
        if TELEMETRY_DISABLED.load(Ordering::Relaxed) {
            return;
        }
        tokio::spawn(async move {
            track_command(command, success, duration_ms).await;
        });
    }
}

/// Programmatically disable telemetry at runtime
pub fn disable() {
    TELEMETRY_DISABLED.store(true, Ordering::Relaxed);
}

/// Check if telemetry is currently enabled
#[must_use]
pub fn is_enabled() -> bool {
    #[cfg(feature = "telemetry")]
    {
        !TELEMETRY_DISABLED.load(Ordering::Relaxed) && get().is_some_and(Telemetry::is_enabled)
    }
    #[cfg(not(feature = "telemetry"))]
    {
        false
    }
}
