# telemetry-kit Integration Report

**Date**: 2025-11-28
**Project**: async-inspect
**telemetry-kit version**: 0.3

---

## Executive Summary

The integration of telemetry-kit into async-inspect was successful. The library works as advertised with minimal boilerplate. This report documents the integration experience and provides feedback for improving telemetry-kit for other developers.

---

## What Worked Well

### 1. Builder Pattern API
The fluent builder pattern is intuitive and feels idiomatic for Rust:
```rust
TelemetryKit::builder()
    .service_name("async-inspect")?
    .service_version(env!("CARGO_PKG_VERSION"))
    .build()?
```

### 2. Privacy-First Defaults
The automatic respect for `DO_NOT_TRACK` environment variable is excellent. This is industry-standard behavior that developers expect.

### 3. Event Builder Closures
The closure-based event builders are ergonomic:
```rust
tk.track_command("build", |event| {
    event.success(true).duration_ms(1234)
}).await?;
```

### 4. Minimal Dependencies
The library doesn't pull in excessive dependencies, making it suitable for CLI tools where binary size matters.

---

## Feedback & Suggestions for Improvement

### 1. **Add a `blocking` / sync API**

**Issue**: For CLI applications that aren't fully async, or for tracking in destructors/`Drop` implementations, the async-only API requires spawning a runtime.

**Current workaround** (in async-inspect):
```rust
std::thread::spawn(|| {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    if let Ok(rt) = rt {
        rt.block_on(async {
            if let Some(tk) = create_telemetry_kit() {
                let _ = tk.track_feature("...", |e| e).await;
            }
        });
    }
});
```

**Suggested enhancement**:
```rust
// Add sync versions
impl TelemetryKit {
    pub fn track_command_blocking<F>(&self, command: &str, f: F) -> Result<()>;
    pub fn track_feature_blocking<F>(&self, feature: &str, f: F) -> Result<()>;
}
```

Or provide a feature flag like `features = ["blocking"]` that enables these methods.

---

### 2. **Document the `method()` semantic on FeatureEventBuilder**

**Issue**: The `method()` function on `FeatureEventBuilder` lacks clear documentation about its intended semantic purpose.

**Question I had**: What is `method` supposed to represent? Is it:
- The HTTP method?
- The variant/mode of the feature?
- The function/method name that triggered the feature?

**Suggestion**: Add documentation explaining the semantic:
```rust
/// Set the method/variant of the feature being tracked.
///
/// This is used to distinguish different ways a feature can be invoked.
/// For example, a "export" feature might have methods like "json", "csv", "html".
///
/// # Example
/// ```rust
/// tk.track_feature("export", |e| {
///     e.method("json").success(true)
/// }).await?;
/// ```
pub fn method(self, method: impl Into<String>) -> Self
```

---

### 3. **Add `track_error` convenience method**

**Issue**: Error tracking is a common use case but requires using `track_feature` with manual configuration.

**Current approach**:
```rust
tk.track_feature("error", |e| {
    e.method("database_connection")
     .success(false)
     .data("error_code", json!("E001"))
}).await?;
```

**Suggested enhancement**:
```rust
impl TelemetryKit {
    pub async fn track_error<F>(&self, error_type: impl Into<String>, f: F) -> Result<()>
    where
        F: FnOnce(ErrorEventBuilder) -> ErrorEventBuilder;
}

// With builder:
struct ErrorEventBuilder {
    fn error_code(self, code: impl Into<String>) -> Self;
    fn message(self, msg: impl Into<String>) -> Self; // Anonymized
    fn component(self, component: impl Into<String>) -> Self;
    fn recoverable(self, recoverable: bool) -> Self;
}
```

---

### 4. **Consider a `LazyTelemetry` wrapper for optional initialization**

**Issue**: When telemetry is an optional feature (via Cargo features), the conditional compilation becomes verbose.

**Current pattern in async-inspect**:
```rust
pub struct Telemetry {
    #[cfg(feature = "telemetry")]
    inner: Option<telemetry_kit::TelemetryKit>,
    #[cfg(not(feature = "telemetry"))]
    _phantom: std::marker::PhantomData<()>,
}
```

**Suggested enhancement**: Provide a `LazyTelemetry` or `OptionalTelemetry` wrapper:
```rust
use telemetry_kit::OptionalTelemetry;

static TELEMETRY: OptionalTelemetry = OptionalTelemetry::new();

pub fn init() {
    TELEMETRY.init(|| {
        TelemetryKit::builder()
            .service_name("my-app")
            .build()
    });
}

pub async fn track(feature: &str) {
    // NoOp if not initialized
    TELEMETRY.track_feature(feature, |e| e.success(true)).await;
}
```

---

### 5. **Add duration tracking helper**

**Issue**: Measuring duration requires manual timing code.

**Current pattern**:
```rust
let start = Instant::now();
// ... do work ...
let duration = start.elapsed().as_millis() as u64;
tk.track_command("build", |e| e.duration_ms(duration)).await?;
```

**Suggested enhancement**:
```rust
// Scoped duration tracking
let _guard = tk.track_command_scoped("build", |e| e.flag("--release"));
// ... do work ...
// Automatically tracked on drop

// Or with async:
tk.track_command_timed("build", |e| e.flag("--release"), async {
    // ... async work ...
}).await?;
```

---

### 6. **Add example for "feature flag" pattern**

**Issue**: Many libraries want telemetry as an optional Cargo feature. The docs don't show how to structure this.

**Suggested documentation addition**:

```markdown
## Making Telemetry Optional

Add telemetry as an optional feature:

```toml
[features]
default = ["telemetry"]
telemetry = ["telemetry-kit"]

[dependencies]
telemetry-kit = { version = "0.3", optional = true }
```

Create a wrapper module:

```rust
// src/telemetry.rs
#[cfg(feature = "telemetry")]
mod inner {
    use telemetry_kit::TelemetryKit;
    // ... full implementation
}

#[cfg(not(feature = "telemetry"))]
mod inner {
    // NoOp stubs
    pub fn init() {}
    pub async fn track_feature(_: &str) {}
}

pub use inner::*;
```
```

---

### 7. **Document error handling best practices**

**Issue**: The API returns `Result<()>` but it's unclear how errors should be handled.

**Questions I had**:
- Should telemetry errors be logged? Silently ignored?
- What kinds of errors can occur (network, storage, validation)?
- Should the main application continue if telemetry fails?

**Suggested documentation**:
```markdown
## Error Handling

Telemetry should never crash your application. We recommend:

```rust
// Ignore errors - telemetry is non-critical
let _ = tk.track_command("build", |e| e).await;

// Or log and continue
if let Err(e) = tk.track_command("build", |e| e).await {
    tracing::debug!("Telemetry error (non-fatal): {}", e);
}
```

Common errors:
- `StorageError`: SQLite issues (disk full, permissions)
- `SyncError`: Network issues (offline, server down)
- `ValidationError`: Invalid event data
```

---

### 8. **Add `#[must_use]` to builders**

**Issue**: If a developer forgets to call `.build()` or the tracking method, there's no warning.

**Suggested enhancement**:
```rust
#[must_use = "builders do nothing until tracked"]
pub struct CommandEventBuilder { ... }

#[must_use = "builder does nothing until .build() is called"]
pub struct TelemetryBuilder { ... }
```

---

### 9. **Consider pre-built feature constants**

**Issue**: Magic strings for common features lead to typos and inconsistency.

**Suggested enhancement**:
```rust
pub mod features {
    pub const STARTUP: &str = "startup";
    pub const SHUTDOWN: &str = "shutdown";
    pub const ERROR: &str = "error";
    pub const CONFIG_CHANGE: &str = "config_change";
}

// Usage
tk.track_feature(telemetry_kit::features::STARTUP, |e| e).await?;
```

---

### 10. **Add integration example with tracing**

**Issue**: Many Rust applications use `tracing`. Showing how telemetry-kit complements (not replaces) tracing would be helpful.

**Suggested documentation**:
```markdown
## Integration with tracing

telemetry-kit is for **usage analytics**, not **debugging/observability**.

| Use Case | Tool |
|----------|------|
| Debug logs | `tracing` |
| Performance spans | `tracing` + OpenTelemetry |
| Usage analytics | `telemetry-kit` |
| Error monitoring | Sentry, etc. |

Example combining both:
```rust
#[tracing::instrument]
async fn export_data(format: &str) -> Result<()> {
    // tracing handles the debug spans
    let data = fetch_data().await?;
    write_output(format, &data)?;

    // telemetry-kit tracks the usage
    telemetry.track_feature("export", |e| {
        e.method(format).success(true)
    }).await?;

    Ok(())
}
```
```

---

## Documentation Gaps Found

1. **No changelog** visible in docs.rs - makes it hard to know what changed between versions
2. **No migration guide** from older versions
3. **Privacy policy template** - Would be helpful to provide a sample privacy policy text that users can adapt
4. **Data retention information** - How long is data kept? Is there auto-cleanup?
5. **Self-hosting documentation** - The README mentions it's self-hostable but I couldn't find detailed setup instructions

---

## Minor Issues Encountered

### 1. Crates.io version mismatch
The README shows `version = "0.0.1"` but the actual version is `0.3`. This should be updated.

### 2. docs.rs rendering
Some code examples in docs.rs don't render properly (likely markdown formatting issues).

---

## Conclusion

telemetry-kit is a solid library that delivers on its promise of "privacy-first, batteries-included telemetry." The integration into async-inspect took approximately 30 minutes, which is excellent for adding usage analytics.

The main areas for improvement are:
1. **Sync API support** (highest impact for CLI tools)
2. **Better documentation** for optional feature patterns
3. **Helper methods** for common patterns (duration, errors)

Overall rating: **4/5** - Would recommend for Rust CLI projects.

---

*This report was generated during the integration of telemetry-kit v0.3 into async-inspect.*
