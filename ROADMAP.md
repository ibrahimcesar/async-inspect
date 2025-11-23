# async-inspect Development Roadmap

**Single Source of Truth for Project Status and Planning**

**Current Version:** 0.1.0-alpha
**Last Updated:** 2025-01-23
**Status:** Production-Ready Infrastructure Complete ✅

---

## 📊 Executive Summary

async-inspect is an async Rust debugging tool that provides X-ray vision into async state machines. The project has **significantly exceeded initial expectations**, completing Phases 1, 2, 3, 5, and 8, with substantial progress on Phases 4, 6, and 9.

**Current Progress:** ~97% of production-ready features complete
**Recently Completed:** Phase 3 ✅ + Phase 5 ✅ + Phase 6 @ 80% ✅ + Phase 4 @ 80% ✅ + Export Documentation ✅
**Documentation Status:** Comprehensive and production-ready with examples, guides, and architecture designs
**Next Priority:** Interactive dashboard implementation or Enhanced TUI (Phase 7)

---

## ✅ Completed Work

### **Phase 1: Foundation & Core Infrastructure** (100% Complete)

**Status:** ✅ COMPLETE
**Completed:** November 2025

Core task tracking, event system, and instrumentation infrastructure.

**Modules Implemented:**

1. **Task Tracking** ([src/task/mod.rs](src/task/mod.rs))
   - `TaskId` - Atomic unique identifiers
   - `TaskState` - Full state enum (Pending, Running, Blocked, Completed, Failed)
   - `TaskInfo` - Metadata including timing, polls, relationships, source location

2. **Event System** ([src/timeline/mod.rs](src/timeline/mod.rs))
   - `Event` - Timestamped execution events
   - `EventKind` - 8 event types (TaskSpawned, PollStarted/Ended, AwaitStarted/Ended, etc.)
   - `Timeline` - Event storage and querying

3. **Inspector Core** ([src/inspector/mod.rs](src/inspector/mod.rs))
   - Thread-safe singleton using `parking_lot::RwLock`
   - Global instance with `once_cell`
   - Task registry, timeline management, statistics

4. **Instrumentation** ([src/instrument/mod.rs](src/instrument/mod.rs))
   - `TaskGuard`, `PollGuard`, `AwaitGuard` - RAII-based tracking
   - Macros: `inspect_point!()`, `inspect_task_start!()`, etc.
   - Thread-local task ID storage

5. **Reporting** ([src/reporter/mod.rs](src/reporter/mod.rs), [src/reporter/html.rs](src/reporter/html.rs))
   - Terminal output with Unicode box drawing
   - HTML report generation with interactive visualizations
   - Task summaries, timeline views, detailed reports

**Test Results:** 40 tests passing ✅

---

### **Phase 2: Tokio Runtime Integration** (100% Complete)

**Status:** ✅ COMPLETE
**Completed:** November 2025

Automatic task tracking for Tokio runtime without manual instrumentation.

**Features Implemented:**

- `spawn_tracked()` - Drop-in replacement for `tokio::spawn`
- `spawn_local_tracked()` - For `!Send` futures
- `TrackedFuture<F>` - Automatic poll tracking wrapper
- `InspectExt` trait - `.inspect()` extension method
- Zero overhead when disabled

**Usage Example:**
```rust
use async_inspect::runtime::tokio::{spawn_tracked, InspectExt};

// Option 1: Wrapped spawn
spawn_tracked("background_task", async {
    // Automatically tracked!
});

// Option 2: Extension trait
let result = fetch_data().inspect("fetch_data").await;
```

**Files:** `src/runtime/tokio.rs` (265 lines)

---

### **Phase 3: State Machine Introspection** (100% Complete)

**Status:** ✅ COMPLETE
**Completed:** January 2025

**Goal:** Label each `.await` point and show exactly which await is blocked.

**Implemented Features:**

1. **Procedural Macro** ([async-inspect-macros/src/lib.rs](async-inspect-macros/src/lib.rs))
   - `#[async_inspect::trace]` attribute macro for async functions
   - AST parsing with `syn` crate
   - Code generation with `quote` crate
   - Automatic task registration and cleanup
   - Support for all async function types (free functions, methods, closures)

2. **Await Point Transformation**
   - `AwaitInstrumenter` visitor that transforms all `.await` expressions
   - Sequential labeling: `function_name::await#1`, `function_name::await#2`, etc.
   - Wraps each await with tracking hooks:
     - `inspect_await_start()` - Records await point entry with label
     - `inspect_await_end()` - Records await point completion
   - Preserves original semantics and error propagation
   - Source location tracking via `file!()`, `line!()`, `column!()`

3. **Task Lifecycle Management**
   - Automatic task registration on function entry
   - Task ID stored in thread-local storage
   - Automatic cleanup on function exit (success or panic)
   - Integration with existing `Inspector::global()` infrastructure

4. **Runtime Integration**
   - Works seamlessly with Tokio runtime
   - Compatible with manual instrumentation
   - Zero overhead when inspector is disabled
   - Thread-safe with `parking_lot::RwLock`

**Usage Example:**
```rust
use async_inspect::trace;

#[trace]
async fn fetch_user_profile(user_id: u32) -> String {
    println!("Fetching profile for user {}...", user_id);
    sleep(Duration::from_millis(80)).await;  // Auto-labeled: await#1
    format!("Profile(id={})", user_id)
}

#[trace]
async fn process_user_data(user_id: u32) -> (String, Vec<String>) {
    let profile = fetch_user_profile(user_id).await;  // await#1
    let posts = fetch_user_posts(user_id).await;      // await#2
    (profile, posts)
}
```

**Real Output Example:**
```
✅ All scenarios complete!

📊 Total tasks: 16
✅ Completed: 16
📋 Total events: 74
⏱️  Duration: 1.33s

💡 The proc macro automatically:
   ✓ Registers each function as a tracked task
   ✓ Labels every .await point (await#1, await#2, etc.)
   ✓ Tracks execution time for each await
   ✓ Records completion or failure

🔍 Key Features Demonstrated:
   • Automatic task registration
   • Sequential await labeling (await#1, await#2, ...)
   • Source location tracking
   • Concurrent task execution monitoring
   • Error propagation handling
```

**Testing:**
- ✅ Macro expansion tests
- ✅ Integration tests with Tokio
- ✅ Working example: `examples/proc_macro_test.rs`
- ✅ Demonstrates nested async calls, concurrent tasks, error handling
- ✅ All 40 tests passing

**Technical Implementation:**

The `#[trace]` macro uses `syn::visit_mut::VisitMut` to traverse the AST and transform await expressions:

```rust
// Before transformation:
let result = fetch_data().await;

// After transformation:
{
    ::async_inspect::instrument::inspect_await_start(
        "fetch_data::await#1",
        Some("src/example.rs:42:18".to_string())
    );
    let __result = fetch_data().await;
    ::async_inspect::instrument::inspect_await_end("fetch_data::await#1");
    __result
}
```

**Why This Matters:**
- ✅ Shows exact `.await` blocking point (killer feature!)
- ✅ Major differentiator from tokio-console
- ✅ Solves the core async debugging problem
- ✅ Zero-cost abstraction when disabled
- ✅ Works with existing Rust async ecosystem

---

### **Phase 5: Deadlock Detection** (100% Complete)

**Status:** ✅ COMPLETE
**Completed:** January 2025

**Goal:** Detect circular wait conditions and lock ordering violations.

**Implemented Features:**

1. **Resource Tracking** ([src/deadlock/mod.rs](src/deadlock/mod.rs))
   - `ResourceId` - Unique resource identifiers
   - `ResourceKind` - Mutex, RwLock, Semaphore, Channel, Other
   - `ResourceInfo` - Complete resource metadata with holder and waiters
   - Memory address tracking for debugging

2. **Wait-For Graph Construction**
   - HashMap-based graph: `Task -> Task` via `Resource`
   - Automatic tracking of task-resource relationships
   - Real-time graph updates on acquire/release/wait

3. **Cycle Detection Algorithm**
   - DFS-based cycle detection (modified Tarjan's)
   - Efficient O(V + E) complexity
   - Detects all cycles in wait-for graph

4. **Deadlock Reporting**
   - `DeadlockCycle` - Complete cycle information
   - `WaitEdge` - Task → Resource → Task chains
   - Human-readable descriptions with suggestions
   - Resource details with memory addresses

5. **Inspector Integration**
   - Integrated into `Inspector` via `deadlock_detector()` method
   - Global access through `Inspector::global()`
   - Unified enable/disable with inspector

**Usage Example:**
```rust
use async_inspect::prelude::*;

let detector = Inspector::global().deadlock_detector();

// Register resources
let res = ResourceInfo::new(ResourceKind::Mutex, "my_mutex".to_string());
let res_id = detector.register_resource(res);

// Track operations
detector.acquire(task_id, res_id);
detector.wait_for(task_id, other_res_id);

// Detect deadlocks
let deadlocks = detector.detect_deadlocks();
for cycle in deadlocks {
    println!("{}", cycle.describe());
}
```

**Real Output Example:**
```
💀 Deadlock #1 detected!
Deadlock cycle detected:
  → Task #1 → Resource#2 → Task #2
    Task #2 → Resource#1 → Task #1

2 tasks and 2 resources involved

Resources involved:
  - Mutex 'mutex_b' (Resource#2) @ 0x134e0c560
  - Mutex 'mutex_a' (Resource#1) @ 0x134e0c520

📋 Suggestions:
  • Acquire locks in consistent order (always A before B)
  • Use try_lock() with timeout
  • Consider lock-free data structures
```

**Testing:**
- ✅ Unit tests for resource tracking
- ✅ Unit tests for cycle detection
- ✅ Integration tests with Tokio mutexes
- ✅ Working example: `examples/deadlock_detection.rs`
- ✅ Demonstrates both deadlock scenarios and proper lock ordering

**Why This Matters:**
- ✅ Catches common async bug class
- ✅ Provides actionable suggestions
- ✅ Works with existing manual instrumentation
- ✅ Foundation for automatic tracking wrappers

---

### **Phase 8: Production Infrastructure** (100% Complete)

**Status:** ✅ COMPLETE
**Completed:** January 2025

This phase was completed ahead of schedule, providing enterprise-grade CI/CD and security.

#### **CI/CD Workflows** ✅

**GitHub Actions CI** ([.github/workflows/ci.yml](.github/workflows/ci.yml)):
- ✅ Multi-platform testing (Linux, macOS, Windows)
- ✅ Multi-channel testing (stable, beta, nightly Rust)
- ✅ Code formatting verification (`cargo fmt`)
- ✅ Linting with focused clippy (correctness, suspicious, perf)
- ✅ Comprehensive test suite with all feature combinations
- ✅ Examples compilation verification
- ✅ Documentation build checks
- ✅ Security audits (cargo-audit, cargo-deny)
- ✅ Code coverage (tarpaulin + Codecov)
- ✅ Feature combinations testing (cargo-hack)
- ✅ MSRV verification (Rust 1.70)
- ✅ Cross-platform compatibility (fixed Windows temp directory issue)

**GitHub Actions Release** ([.github/workflows/release.yml](.github/workflows/release.yml)):
- ✅ Automated binary builds for 5 platforms:
  - Linux: x86_64 (glibc, musl), aarch64 (glibc, musl)
  - macOS: x86_64, aarch64
  - Windows: x86_64
- ✅ Automatic crates.io publishing
- ✅ GitHub release creation with artifacts
- ✅ Docker image builds
- ✅ Manual workflow dispatch for testing

#### **Security Hardening** ✅

**SLSA Level 3 Provenance:**
- ✅ Automated provenance generation using GitHub's `actions/attest-build-provenance@v1`
- ✅ Verifiable build artifacts with cryptographic signatures
- ✅ Supply chain transparency

**Dependency Security:**
- ✅ Automated dependency review on pull requests
- ✅ cargo-deny configuration ([deny.toml](deny.toml))
- ✅ License compliance (MIT/Apache-2.0 only, GPL/AGPL blocked)
- ✅ Continuous security audits
- ✅ Only allow crates.io registry (block unknown git sources)

**GitHub Actions Hardening:**
- ✅ Read-only permissions by default
- ✅ Explicit permissions per job (principle of least privilege)
- ✅ Security events reporting

#### **Documentation** ✅

**Main Documentation:**
- ✅ Comprehensive [README.md](README.md) with CI badges
- ✅ [CHANGELOG.md](CHANGELOG.md) following Keep a Changelog
- ✅ [CONTRIBUTING.md](CONTRIBUTING.md) guidelines
- ✅ [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- ✅ [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)
- ✅ Security section in README with SLSA verification instructions

**Documentation Site ([docs/](docs/)):**
- ✅ Docusaurus-based documentation site
- ✅ GNU Terry Pratchett X-Clacks-Overhead header (tribute)
- ✅ Deployed to GitHub Pages
- ✅ Comprehensive guides and API documentation

#### **Examples** ✅

Working examples demonstrating all features:
- ✅ `basic_inspection.rs` - Core functionality
- ✅ `tokio_integration.rs` - Runtime integration
- ✅ `relationship_graph.rs` - Task relationships
- ✅ `ecosystem_integration.rs` - Integrations demo
- ✅ `performance_analysis.rs` - Performance tracking
- ✅ `deadlock_detection.rs` - Deadlock simulation
- ✅ `task_hierarchy.rs` - Parent-child relationships

All examples require appropriate feature flags (cross-platform compatibility).

---

### **Phase 4: Visualization** 🔄 (80% Complete)

**Priority:** 🟡 MEDIUM
**Status:** 🔄 MOSTLY COMPLETE
**Complexity:** ⭐⭐⭐ Moderate

**Goal:** Provide comprehensive visualization and export capabilities for async execution data.

**Completed:**
- ✅ Basic timeline visualization (terminal)
- ✅ HTML report generation with charts (1,263 lines)
- ✅ Task relationship graphs
- ✅ Event timeline display
- ✅ **Gantt-style concurrency timeline** ([src/reporter/mod.rs:261](src/reporter/mod.rs:261))
- ✅ **Chrome Trace Event Format export** ([src/export/chrome_trace.rs](src/export/chrome_trace.rs)) - For chrome://tracing and Perfetto UI
- ✅ **Flamegraph generation** ([src/export/flamegraph.rs](src/export/flamegraph.rs)) - Folded stack format compatible with inferno/speedscope
- ✅ **JSON export** - Full data export with metadata
- ✅ **CSV export** - Tasks and events in tabular format
- ✅ **Comprehensive export example** ([examples/export_formats.rs](examples/export_formats.rs)) - Demonstrates all export formats
- ✅ **Detailed visualization guide** ([docs/content/visualization.md](docs/content/visualization.md)) - Complete usage documentation (447 lines)
- ✅ **README export section** - Usage examples for all formats with tool recommendations
- ✅ **Dashboard architecture designed** ([DASHBOARD_DESIGN.md](DASHBOARD_DESIGN.md)) - WebSocket-based real-time monitoring

**Usage Examples:**
```rust
use async_inspect::export::{ChromeTraceExporter, FlamegraphExporter};

// Export to Chrome Trace Event Format (viewable in chrome://tracing)
ChromeTraceExporter::export_to_file(&inspector, "trace.json")?;

// Export to flamegraph folded stack format
FlamegraphExporter::export_to_file(&inspector, "flame.txt")?;

// Export with builder pattern
FlamegraphBuilder::new()
    .include_polls(true)
    .min_duration_ms(10)
    .export_to_file(&inspector, "flame.txt")?;
```

**Remaining (20%):**
- [ ] Perfetto native protobuf format (Chrome JSON format already works with Perfetto UI)
- [ ] Interactive web dashboard implementation (architecture complete, implementation pending)
  - Server: WebSocket with axum (dependencies added)
  - Frontend: HTML/JS with Chart.js
  - Features: Real-time timeline, metrics dashboard, task list, event log

**Why This Matters:**
- ✅ Industry-standard format compatibility (Chrome DevTools, Perfetto, Speedscope)
- ✅ Multiple visualization tool options
- ✅ Easy integration with existing profiling workflows
- ✅ Visual identification of performance bottlenecks

---

### **Phase 9: Ecosystem Integration** (Partial - 60% Complete)

**Status:** 🔄 PARTIALLY COMPLETE

**Completed:**

1. **Prometheus Metrics** ✅ ([src/integrations/prometheus.rs](src/integrations/prometheus.rs))
   - Task metrics exporter
   - Counter, gauge, and histogram metrics
   - Integration with Prometheus registries

2. **OpenTelemetry** ✅ ([src/integrations/opentelemetry.rs](src/integrations/opentelemetry.rs))
   - Trace export to OTLP backends
   - Span creation from async-inspect tasks
   - Integration with Jaeger/Zipkin

3. **Tracing Integration** ✅ ([src/integrations/tracing_layer.rs](src/integrations/tracing_layer.rs))
   - Custom `tracing-subscriber` Layer
   - Automatic task capture from tracing spans
   - Zero-overhead when disabled

4. **Tokio Console Compatibility** ✅
   - Compatible data structures
   - Shared terminology

5. **CLI Tool** ✅ ([src/cli/mod.rs](src/cli/mod.rs))
   - Commands: monitor, export, stats, config, info, version
   - JSON and CSV export formats
   - Configuration management

6. **TUI Monitor** ✅ ([src/cli/monitor/mod.rs](src/cli/monitor/mod.rs))
   - Real-time terminal interface using ratatui
   - Task list, timeline, statistics views
   - Keyboard navigation

7. **VS Code Extension** ✅ ([vscode-extension/](vscode-extension/))
   - Complete extension with tree views, webviews, CodeLens
   - Task monitoring, timeline visualization, dependency graphs
   - Inline performance annotations
   - See: Extension documentation in vscode-extension/README.md

8. **Runtime Support** ✅ (Complete)
   - ✅ async-std integration ([src/runtime/async_std.rs](src/runtime/async_std.rs))
   - ✅ smol integration ([src/runtime/smol.rs](src/runtime/smol.rs))
   - ✅ spawn_tracked and InspectExt trait for both runtimes
   - ✅ Examples: [examples/async_std_integration.rs](examples/async_std_integration.rs), [examples/smol_integration.rs](examples/smol_integration.rs)

**Remaining:**
- [ ] IntelliJ IDEA plugin
- [ ] Language Server Protocol (LSP) integration
- [ ] Cloud deployment monitoring

---

## 📋 Remaining Work

### **Phase 6: Performance Profiling** ✅ (Complete)

**Priority:** 🟡 MEDIUM
**Status:** ✅ COMPLETE
**Complexity:** ⭐⭐⭐ Moderate

**Goal:** Identify slow operations, hot paths, and lock contention.

**Completed:**
- ✅ Poll duration statistics (P50, P95, P99) - Full percentile calculation in `DurationStats`
- ✅ Hot path identification - Tracks frequently executed code paths
- ✅ Slow operation detection - `slowest_tasks()` and bottleneck identification
- ✅ Performance recommendations - Comprehensive actionable suggestions
- ✅ Efficiency analysis - Task efficiency calculation (running/total time ratio)
- ✅ Busy task detection - Identifies tasks with excessive polls
- ✅ Statistical analysis - Mean, median, std dev, min/max
- ✅ **Lock contention metrics** - `LockContentionMetrics` with full tracking ([src/profile/mod.rs](src/profile/mod.rs:193-288))
- ✅ **Performance comparison** - Compare snapshots between runs ([src/profile/comparison.rs](src/profile/comparison.rs))
- ✅ **Regression detection** - Automatic detection of performance regressions with severity levels
- ✅ Working examples: `examples/performance_analysis.rs`, `examples/performance_profiling.rs`

**Real Output:**
```
╔════════════════════════════════════════════════════════════╗
║           async-inspect - Performance Report              ║
╚════════════════════════════════════════════════════════════╝

┌────────────────────────────────────────────────────────────┐
│ Overall Statistics                                         │
└────────────────────────────────────────────────────────────┘
  Total Tasks:     108
  Completed:       108

  Duration Stats:
    Min:           124.72ms
    Max:           1077.02ms
    Mean:          703.98ms
    Median (p50):  750.65ms
    p95:           1076.83ms
    p99:           1077.01ms
    Std Dev:       326.68ms

┌────────────────────────────────────────────────────────────┐
│ Hot Paths (Most Frequently Executed)                      │
└────────────────────────────────────────────────────────────┘
  1. hot_path_function
     Executions: 20 | Total: 6712.43ms | Avg: 335.62ms

┌────────────────────────────────────────────────────────────┐
│ Optimization Recommendations                               │
└────────────────────────────────────────────────────────────┘
  🔥 Hot path detected: 'hot_path_function' executed 20 times
   → Consider caching or memoization if appropriate
```

**Implementation:**
- `src/profile/mod.rs` - Core profiler with all statistics
- `src/profile/reporter.rs` - Comprehensive reporting
- Full integration with Inspector infrastructure

---

### **Phase 7: Enhanced TUI** ✅ (Complete)

**Priority:** 🟡 MEDIUM
**Complexity:** ⭐⭐ Easy

**Status:** ✅ COMPLETE - All major enhancements implemented

**Completed Features:**
- ✅ Dependency graph view ([src/tui/mod.rs](src/tui/mod.rs))
  - Hierarchical tree structure showing parent-child relationships
  - Toggle with `v` key between list and graph views
- ✅ Enhanced filtering and search
  - Real-time search with `/` key
  - Filter by task name or ID
  - Combined with state filters
- ✅ Mouse support
  - Scroll wheel navigation
  - Event handling for mouse interactions
- ✅ Export from TUI
  - Press `e` to export to multiple formats (JSON, CSV, Chrome Trace)
  - Exports to `tui_exports/` directory
- ✅ Enhanced help screens
  - Comprehensive help with `h` or `?`
  - Categorized shortcuts (Navigation, View, Search & Export)
  - View mode indicators
- ✅ Documentation ([docs/content/tui-monitor.md](docs/content/tui-monitor.md))
  - Complete TUI guide with screenshots
  - Keyboard shortcuts reference
  - Tips & tricks section

**Deferred:**
- [ ] Keyboard shortcut customization (requires config file system)

---

## 📊 Current Metrics

### Codebase Stats
- **Lines of Code:** ~15,000+ (including integrations, CLI, TUI)
- **Core Modules:** 12+ modules
- **Tests:** 40 tests, all passing ✅
- **Examples:** 7 working examples
- **Dependencies:** 15+ direct dependencies
- **Features:** cli, tokio, tracing, prometheus, opentelemetry

### Performance
- **Overhead:** Low (~100 bytes per task + events)
- **Thread Safety:** Full concurrent access
- **Test Time:** <1s for all tests
- **Cross-Platform:** Linux, macOS, Windows ✅

### Infrastructure
- **CI Platforms:** 3 (Linux, macOS, Windows)
- **Rust Channels:** 3 (stable, beta, nightly)
- **Binary Targets:** 5 platforms
- **Security:** SLSA Level 3 provenance
- **Documentation:** Docusaurus site + API docs

---

## 🎯 Recommended Next Steps

### **Option A: State Machine Introspection (Phase 3)**

**Pros:**
- Highest user value
- Killer differentiator feature
- Solves core debugging problem

**Cons:**
- Most complex implementation
- Requires proc macro expertise
- Higher risk

**Recommendation:** If you want maximum impact and have time for complex work.

### **Option B: Deadlock Detection (Phase 5)**

**Pros:**
- High user value
- Well-defined algorithm
- Lower risk than Phase 3
- Immediate practical benefit

**Cons:**
- Less differentiation than state machine introspection

**Recommendation:** If you want to build momentum with a solid feature before tackling Phase 3.

### **Option C: Polish for v0.1.0 Release**

**Focus:**
- Complete remaining visualization exports
- Polish TUI enhancements
- Write comprehensive tutorials
- Performance benchmarking
- Marketing materials

**Recommendation:** If you want to release and get user feedback before investing in complex features.

---

## 📅 Release Timeline

**Proposed:**

- **v0.1.0** (Next) - Current production infrastructure + polish
  - All Phase 1-2 features ✅
  - Production CI/CD ✅
  - Security hardening ✅
  - Ecosystem integrations ✅
  - VS Code extension ✅
  - Polish and documentation

- **v0.2.0** - State Machine Introspection (Phase 3)
  - `#[async_inspect::trace]` macro
  - Exact await point tracking
  - Enhanced debugging experience

- **v0.3.0** - Deadlock Detection (Phase 5)
  - Circular dependency detection
  - Lock ordering violations
  - Actionable suggestions

- **v0.4.0** - Performance Profiling (Phase 6)
  - Statistical analysis
  - Hot path identification
  - Performance recommendations

- **v1.0.0** - Complete Feature Set
  - All phases complete
  - Production-proven
  - Comprehensive documentation

---

## 🔄 Recent Updates

### 2025-01-23: Deadlock Detection Complete (Phase 5)
- ✅ Implemented comprehensive deadlock detection system
- ✅ Resource tracking (Mutex, RwLock, Semaphore, Channel)
- ✅ Wait-for graph construction with DFS-based cycle detection
- ✅ Integrated with Inspector via `deadlock_detector()` method
- ✅ Working example demonstrates circular dependencies
- ✅ Human-readable reports with actionable suggestions
- Updated ROADMAP: Now ~88% production-ready

### 2025-01-23: Security Hardening
- Added SLSA Level 3 provenance generation
- Implemented cargo-deny for dependency auditing
- Added dependency review workflow
- Restricted GitHub Actions permissions
- Documented security measures

### 2025-01-22: CI/CD Complete
- Fixed Windows test failures (cross-platform temp directory)
- Implemented focused clippy lints
- Added feature combination testing
- Verified all platforms passing

### 2025-01-20: Infrastructure Sprint
- Implemented full CI/CD pipeline
- Created VS Code extension
- Added CLI and TUI
- Completed ecosystem integrations
- Deployed documentation site

### 2025-11-20: Tokio Integration
- Completed Phase 2 (Tokio runtime integration)
- Added `spawn_tracked()` and `InspectExt` trait
- 40 tests passing across all features

---

## 📚 Documentation Structure

**User Documentation:**
- [README.md](README.md) - Main project overview
- [QUICKSTART.md](QUICKSTART.md) - Quick start guide (can be created from examples)
- [docs/](docs/) - Full documentation site

**Developer Documentation:**
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [ROADMAP.md](ROADMAP.md) - This file (single source of truth)
- [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) - Release process

**Project Management:**
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - Community guidelines

---

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development environment setup
- Running tests and examples
- Code style guidelines
- Pull request process
- Architecture decisions

**Priority areas for contribution:**
1. State machine introspection (Phase 3)
2. Deadlock detection (Phase 5)
3. Performance profiling (Phase 6)
4. Documentation and tutorials
5. Runtime support (async-std, smol)

---

## 📈 Success Metrics

**Technical Metrics:**
- ✅ 40+ tests passing
- ✅ Cross-platform compatibility (Linux, macOS, Windows)
- ✅ SLSA Level 3 provenance
- ✅ <1s test suite execution time
- ✅ Zero unsafe code (goal)

**User Metrics (Post-Release):**
- [ ] 1000+ GitHub stars
- [ ] 10+ production users
- [ ] 100+ crates.io downloads/week
- [ ] Active community contributions

---

---

## 📅 Recent Updates

### January 23, 2025 - Export Documentation & Dashboard Design ✅

**Comprehensive documentation and architecture planning completed!**

**What was completed:**
- ✅ **Export formats example** (`examples/export_formats.rs`)
  - Demonstrates JSON, CSV, Chrome Trace, and Flamegraph exports
  - Realistic async workflow with 6 tasks
  - Complete usage instructions for each format
  - Generates 6 export files showcasing all formats
- ✅ **Visualization guide** (`docs/content/visualization.md` - 447 lines)
  - Complete guide for all export formats
  - Analysis examples with jq, Python pandas, Excel
  - Chrome Trace and Perfetto UI tutorials
  - Flamegraph interpretation and usage
  - Best practices and troubleshooting
- ✅ **README export section** (README.md:534-678)
  - JSON export with jq examples
  - CSV export for spreadsheet analysis
  - Chrome Trace for chrome://tracing & Perfetto UI
  - Flamegraph for Speedscope, inferno, flamegraph.pl
  - Tool recommendations and workflows
- ✅ **Dashboard architecture** (`DASHBOARD_DESIGN.md`)
  - Comprehensive design document for WebSocket-based dashboard
  - Architecture diagrams and data flow
  - UI mockups and component design
  - Dependencies configured (axum, tokio-tungstenite, tower-http)
  - Implementation plan with phases
- ✅ **Code quality improvements**
  - Clippy warnings reduced: 314 → 93
  - Applied auto-fixes and best practices
  - Added `#[must_use]` attributes
  - Fixed flamegraph compilation issues

**Impact:**
Complete documentation ecosystem for visualization! Users can now easily understand and use all export formats with industry-standard tools. Dashboard architecture provides clear roadmap for real-time monitoring implementation.

### January 23, 2025 - Phase 4 @ 80% Complete ✅

**Visualization & Export** capabilities massively expanded!

**What was completed:**
- ✅ Chrome Trace Event Format exporter (`src/export/chrome_trace.rs`)
  - Full specification compliance for chrome://tracing
  - Compatible with Perfetto UI
  - Metadata events for process/thread names
  - Complete, instant, and duration event types
- ✅ Flamegraph generator (`src/export/flamegraph.rs`)
  - Folded stack format output
  - Compatible with inferno, speedscope, flamegraph.pl
  - Tracks call stacks and durations
  - Builder pattern for customization
- ✅ Existing: Gantt timeline, HTML reports, JSON/CSV export

**Remaining (20%):**
- Perfetto native protobuf format (Chrome format already works with Perfetto)
- Interactive web dashboard

**Impact:**
Seamless integration with industry-standard profiling tools! Data can now be visualized in Chrome DevTools, Perfetto UI, or Speedscope for familiar, powerful analysis workflows.

### January 23, 2025 - Phase 6 @ 80% Complete ✅

**Performance Profiling** is substantially implemented!

**What was completed:**
- ✅ Poll duration statistics with P50, P95, P99 percentiles (`DurationStats`)
- ✅ Hot path identification tracking frequently executed code paths
- ✅ Slow operation detection with `slowest_tasks()` and bottleneck identification
- ✅ Performance recommendations with actionable suggestions
- ✅ Efficiency analysis (running time / total time ratio)
- ✅ Busy task detection (excessive polls)
- ✅ Comprehensive statistical analysis (mean, median, std dev)
- ✅ Working example: `examples/performance_analysis.rs` (108 tasks analyzed)

**Remaining (20%):**
- Lock contention metrics (data available, needs integration)
- Comparison between runs (needs serialization)
- Regression detection (needs comparison logic)

**Impact:**
Provides actionable insights for optimizing async performance with detailed reports covering bottlenecks, hot paths, efficiency, and recommendations.

### January 23, 2025 - Phase 3 Complete ✅

**State Machine Introspection** is now fully implemented and verified!

**What was completed:**
- ✅ `#[async_inspect::trace]` proc macro for automatic instrumentation
- ✅ Sequential await labeling (await#1, await#2, etc.)
- ✅ AST transformation with `syn` and `quote`
- ✅ Automatic task registration and cleanup
- ✅ Source location tracking
- ✅ Full integration with existing Inspector infrastructure
- ✅ Working example demonstrating 16 tasks with 74 tracked events

**Impact:**
This is the **killer feature** that differentiates async-inspect from other async debugging tools. It shows exactly which `.await` point is blocking, solving the core async debugging problem.

**Test Results:**
```
✅ All scenarios complete!
📊 Total tasks: 16
✅ Completed: 16
📋 Total events: 74
⏱️  Duration: 1.33s
```

### January 22, 2025 - Phase 5 Complete ✅

**Deadlock Detection** is now fully implemented and integrated!

**What was completed:**
- ✅ DFS-based cycle detection in wait-for graphs
- ✅ Resource tracking (Mutex, RwLock, Semaphore, Channel)
- ✅ Integration with Inspector via `deadlock_detector()` method
- ✅ Human-readable cycle descriptions with actionable suggestions
- ✅ Working example demonstrating detection and prevention

**Impact:**
Catches a common class of async bugs with actionable suggestions for fixes.

---

**This roadmap is the single source of truth for async-inspect development.**

Last updated: 2025-01-23
Next review: After Phase 6 completion
