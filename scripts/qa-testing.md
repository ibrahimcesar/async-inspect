# async-inspect QA Testing Guide

This document provides end-to-end testing procedures for all async-inspect features.

## Prerequisites

```bash
# Build with all features
cargo build --all-features

# Run all tests first
cargo test --all-features
```

---

## 1. Core Functionality Tests

### 1.1 Basic Task Tracking

**Test:** Verify basic task registration and state tracking.

```bash
cargo run --example basic_inspection
```

**Expected:**
- [ ] Tasks are registered with unique IDs
- [ ] Task states transition correctly (Pending → Running → Completed)
- [ ] Task names are displayed correctly
- [ ] Duration tracking works

**Notes:**
```
_____________________________________________
```

### 1.2 Proc Macro Instrumentation

**Test:** Verify `#[trace]` and `#[inspect]` macros work.

```bash
cargo run --example proc_macro_test
```

**Expected:**
- [ ] `#[async_inspect::trace]` instruments functions
- [ ] `#[async_inspect::inspect]` provides lighter instrumentation
- [ ] Await points are tracked
- [ ] Function entry/exit logged

**Notes:**
```
_____________________________________________
```

---

## 2. Runtime Integration Tests

### 2.1 Tokio Integration

**Test:** Verify Tokio runtime integration.

```bash
cargo run --example tokio_integration --features tokio
```

**Expected:**
- [ ] Tasks spawned with `tokio::spawn` are tracked
- [ ] Multi-threaded runtime works correctly
- [ ] Task hierarchy (parent-child) is maintained

**Notes:**
```
_____________________________________________
```

### 2.2 async-std Integration

**Test:** Verify async-std runtime support.

```bash
cargo run --example async_std_integration --features async-std-runtime
```

**Expected:**
- [ ] async-std tasks are tracked
- [ ] Runtime detection works

**Notes:**
```
_____________________________________________
```

### 2.3 smol Integration

**Test:** Verify smol runtime support.

```bash
cargo run --example smol_integration --features smol-runtime
```

**Expected:**
- [ ] smol tasks are tracked
- [ ] Lightweight runtime overhead

**Notes:**
```
_____________________________________________
```

---

## 3. Deadlock Detection Tests

### 3.1 Basic Deadlock Detection

**Test:** Verify deadlock detection identifies circular dependencies.

```bash
cargo run --example deadlock_detection
```

**Expected:**
- [ ] Circular lock dependencies are detected
- [ ] Deadlock warning is displayed
- [ ] Involved tasks are identified
- [ ] Lock acquisition order is shown

**Notes:**
```
_____________________________________________
```

### 3.2 Tracked Mutex Deadlock Detection

**Test:** Verify `async_inspect::sync::Mutex` auto-detection.

```bash
cargo run --example auto_lock_tracking --features tokio
```

**Expected:**
- [ ] Tracked Mutex records acquisitions
- [ ] Contention is measured
- [ ] Potential deadlocks flagged
- [ ] Metrics available via `.metrics()`

**Notes:**
```
_____________________________________________
```

---

## 4. Production Features Tests

### 4.1 Ring Buffer Mode

**Test:** Verify fixed-memory ring buffer for events.

```bash
cargo test ringbuf --all-features
```

**Manual Test (in Rust):**
```rust
use async_inspect::timeline::{Timeline, TimelineConfig};

let timeline = Timeline::with_ring_buffer(100);
// Add 200 events...
assert_eq!(timeline.len(), 100); // Oldest evicted
let stats = timeline.ring_stats().unwrap();
println!("Overwrites: {}", stats.overwrites);
```

**Expected:**
- [ ] Buffer doesn't grow beyond capacity
- [ ] Oldest events are evicted
- [ ] Statistics track overwrites
- [ ] Memory usage is bounded

**Notes:**
```
_____________________________________________
```

### 4.2 Adaptive Sampling

**Test:** Verify adaptive sampling adjusts based on overhead.

```bash
cargo test adaptive --all-features
```

**Manual Test (in Rust):**
```rust
use async_inspect::config::Config;

let config = Config::global();
config.enable_adaptive_sampling();
config.set_adaptive_bounds(1, 1000);
config.set_adaptive_target_overhead_ms(1.0);

// Simulate high load...
let stats = config.adaptive_stats();
println!("Current rate: {}, Within budget: {}",
    stats.current_rate, stats.is_within_budget());
```

**Expected:**
- [ ] Sampling rate increases under high load
- [ ] Sampling rate decreases when overhead is low
- [ ] Target overhead is respected
- [ ] Statistics accurately reflect behavior

**Notes:**
```
_____________________________________________
```

### 4.3 Production Mode

**Test:** Verify production configuration preset.

```rust
use async_inspect::config::Config;

Config::global().production_mode();
assert_eq!(Config::global().sampling_rate(), 100);
assert!(!Config::global().track_awaits());
```

**Expected:**
- [ ] Sampling rate set to 100
- [ ] Await tracking disabled
- [ ] HTML generation disabled
- [ ] Low overhead confirmed

**Notes:**
```
_____________________________________________
```

---

## 5. TUI Tests

### 5.1 TUI Monitor

**Test:** Verify terminal UI functionality.

```bash
cargo run --example tui_monitor --features cli
```

**Expected:**
- [ ] TUI launches correctly
- [ ] Task list displays
- [ ] Real-time updates work
- [ ] Keyboard navigation works (↑/↓, j/k)

**Keyboard Tests:**
| Key | Action | Works? |
|-----|--------|--------|
| `↑/↓` or `j/k` | Navigate tasks | [ ] |
| `/` | Search tasks | [ ] |
| `f` | Filter by state | [ ] |
| `s` | Sort tasks | [ ] |
| `d` | Task details | [ ] |
| `?` | Show help | [ ] |
| `q` | Quit | [ ] |

**Notes:**
```
_____________________________________________
```

### 5.2 Task Filtering

**Test:** Verify task filtering in TUI.

```bash
cargo run --example tui_monitor --features cli
# Press 'f' to filter, test each filter type
```

**Expected:**
- [ ] Filter by state (Running, Blocked, Completed)
- [ ] Filter by name pattern (glob: `fetch_*`)
- [ ] Filter by duration (> 100ms)
- [ ] Multiple filters combine correctly

**Notes:**
```
_____________________________________________
```

---

## 6. Dashboard Tests

### 6.1 Web Dashboard

**Test:** Verify web dashboard starts and displays data.

```bash
cargo run --example dashboard_demo --features dashboard
# Open http://localhost:3000 in browser
```

**Expected:**
- [ ] Dashboard starts on port 3000
- [ ] Task list displays in browser
- [ ] Real-time updates via WebSocket
- [ ] Task details expandable
- [ ] Timeline visualization works

**Notes:**
```
_____________________________________________
```

---

## 7. Channel & Sync Primitive Tests

### 7.1 Tracked Channels

**Test:** Verify channel tracking (mpsc, oneshot, broadcast).

```bash
cargo run --example channel_visualization --features tokio
```

**Expected:**
- [ ] Channel creation tracked
- [ ] Send/receive operations logged
- [ ] Backpressure detected
- [ ] Channel stats available

**Notes:**
```
_____________________________________________
```

### 7.2 Lock Contention Metrics

**Test:** Verify lock contention tracking.

```rust
use async_inspect::sync::Mutex;

let mutex = Mutex::new(42);
// Concurrent access...
let metrics = mutex.metrics();
println!("Contentions: {}", metrics.contentions);
println!("Avg wait: {:?}", metrics.avg_wait_time);
```

**Expected:**
- [ ] Acquisition count accurate
- [ ] Contention count accurate
- [ ] Wait time measured
- [ ] Contention rate calculated

**Notes:**
```
_____________________________________________
```

---

## 8. Export & Reporting Tests

### 8.1 JSON Export

**Test:** Verify JSON export functionality.

```bash
cargo run --example export_formats --features tokio
```

**Expected:**
- [ ] Tasks export to valid JSON
- [ ] Timeline exports correctly
- [ ] JSON can be re-imported
- [ ] All fields present

**Notes:**
```
_____________________________________________
```

### 8.2 HTML Reports

**Test:** Verify HTML report generation.

```bash
cargo run --example visualization --features tokio
# Check generated HTML file
```

**Expected:**
- [ ] HTML file generated
- [ ] Opens in browser
- [ ] Timeline visualization works
- [ ] Task details displayed

**Notes:**
```
_____________________________________________
```

---

## 9. LSP Integration Tests

### 9.1 LSP Server

**Test:** Verify LSP server starts.

```bash
cargo run --bin async-inspect-lsp --features lsp
```

**Expected:**
- [ ] LSP server starts without errors
- [ ] Accepts connections
- [ ] Responds to initialize request

**Notes:**
```
_____________________________________________
```

### 9.2 VS Code Extension

**Test:** Verify VS Code extension integration.

```bash
cd vscode-extension
npm install
npm run compile
# Install extension in VS Code
```

**Expected:**
- [ ] Extension installs
- [ ] Commands available in command palette
- [ ] Task tree view displays
- [ ] Hover info works

**Notes:**
```
_____________________________________________
```

---

## 10. Performance Tests

### 10.1 Overhead Benchmark

**Test:** Measure instrumentation overhead.

```bash
cargo bench overhead
```

**Expected:**
- [ ] Overhead < 1μs per task in production mode
- [ ] Overhead < 10μs per task in debug mode
- [ ] No significant memory growth

**Results:**
```
Production mode overhead: ______ ns/task
Debug mode overhead: ______ ns/task
Memory per 1000 tasks: ______ KB
```

### 10.2 High Task Count

**Test:** Verify performance with many tasks.

```bash
cargo bench throughput
```

**Expected:**
- [ ] 10,000+ tasks tracked without issue
- [ ] TUI remains responsive
- [ ] Memory usage bounded with ring buffer

**Notes:**
```
_____________________________________________
```

---

## 11. Error Handling Tests

### 11.1 Enhanced Error Messages

**Test:** Verify actionable error messages.

```rust
use async_inspect::errors::{ConfigError, Diagnostics};

// Test config error
let err = ConfigError::invalid_sampling_rate(0);
println!("{}", err);
// Should include suggestion

// Test diagnostics
let msg = Diagnostics::high_memory_usage(100_000_000, 50_000_000);
println!("{}", msg);
// Should include actionable suggestions
```

**Expected:**
- [ ] Errors include clear description
- [ ] Suggestions are actionable
- [ ] Feature-not-enabled errors show Cargo.toml fix

**Notes:**
```
_____________________________________________
```

---

## 12. Documentation Tests

### 12.1 Doc Tests Pass

```bash
cargo test --doc --all-features
```

**Expected:**
- [ ] All doc tests pass
- [ ] Examples in docs compile

### 12.2 Docusaurus Build

```bash
cd docs
npm install
npm run build
```

**Expected:**
- [ ] Docs build without errors
- [ ] All pages render correctly
- [ ] Links work

**Notes:**
```
_____________________________________________
```

---

## Test Summary

| Category | Tests | Passed | Failed | Notes |
|----------|-------|--------|--------|-------|
| Core Functionality | 2 | | | |
| Runtime Integration | 3 | | | |
| Deadlock Detection | 2 | | | |
| Production Features | 3 | | | |
| TUI | 2 | | | |
| Dashboard | 1 | | | |
| Channels & Sync | 2 | | | |
| Export & Reporting | 2 | | | |
| LSP Integration | 2 | | | |
| Performance | 2 | | | |
| Error Handling | 1 | | | |
| Documentation | 2 | | | |
| **Total** | **24** | | | |

---

## Quick Smoke Test Commands

Run these for a quick validation:

```bash
# 1. Build everything
cargo build --all-features

# 2. Run all unit tests
cargo test --all-features

# 3. Run doc tests
cargo test --doc --all-features

# 4. Check clippy
cargo clippy --all-features

# 5. Quick examples
cargo run --example basic_inspection
cargo run --example tokio_integration --features tokio
cargo run --example deadlock_detection

# 6. TUI (if interactive testing)
cargo run --example tui_monitor --features cli

# 7. Dashboard (if web testing)
cargo run --example dashboard_demo --features dashboard
```

---

## Regression Checklist

Before release, ensure:

- [ ] All unit tests pass
- [ ] All doc tests pass
- [ ] All examples run without errors
- [ ] TUI keyboard shortcuts work
- [ ] Dashboard loads in browser
- [ ] Production mode has low overhead
- [ ] Ring buffer correctly bounds memory
- [ ] Adaptive sampling adjusts correctly
- [ ] Error messages are helpful
- [ ] Documentation builds

---

*Last updated: 2025-12-09*
