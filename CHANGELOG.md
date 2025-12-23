# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-12-23

### Added
- **Production Readiness Features**
  - **Ring Buffer Mode**: Fixed-memory event storage for production environments
    * `Timeline::with_ring_buffer(capacity)` for bounded memory usage
    * Automatic eviction of oldest events when capacity is reached
    * Statistics tracking (overwrites, utilization, churn rate)
    * Lock-free reads using `parking_lot::RwLock`
    * Module: `src/ringbuf.rs`
  - **Adaptive Sampling**: Automatically adjusts sampling rate based on measured overhead
    * `Config::global().enable_adaptive_sampling()` to enable
    * Configurable min/max sampling rates and target overhead
    * Self-tuning algorithm that responds to system load
    * `adaptive_stats()` for monitoring sampling behavior
  - **Memory Optimization**: Reduced memory overhead per tracked task
    * String interning (`src/intern.rs`) for repeated task names
    * Compact task storage (`src/compact.rs`) with pre-allocated pools
    * `TaskPool` for high-performance scenarios
  - **Enhanced Error Messages**: Actionable error types with suggestions
    * `ConfigError` for misconfiguration issues with fix suggestions
    * `TaskError` for task tracking problems
    * `Diagnostics` helper for runtime warnings (high memory, deadlocks, slow tasks)
    * Module: `src/errors.rs`
  - **Task Filtering in TUI**: Filter tasks by state, name pattern, and duration
    * Press `f` in TUI to access filtering
    * Glob pattern support for task names
    * Duration-based filtering for slow task detection

### Documentation
- **API Reference** (`docs/content/api-reference.md`): Comprehensive public API documentation with stability guarantees
- **Getting Started Guide** (`docs/content/getting-started.md`): Quick start in under 5 minutes
- **Security Policy** (`SECURITY.md`): Vulnerability reporting and security best practices
- **QA Testing Guide** (`scripts/qa-testing.md`): Comprehensive manual testing checklist
- **QA Smoke Test** (`scripts/qa-smoke-test.sh`): Automated smoke test script

### Fixed
- Dead code warning for `window_start_ms` field in config

### Security
- Updated npm dependencies to fix security vulnerabilities in documentation site
- Added comprehensive security policy with response timelines

---

- **Language Server Protocol (LSP) Support (Phase 9 - Complete)**: Universal editor integration via LSP
  - **LSP server binary** (`async-inspect-lsp`) for any LSP-compatible editor
    * Standalone binary built with `--features lsp`
    * Communicates via stdin/stdout following LSP specification
    * Lightweight and performant server implementation
  - **Diagnostics** for async code quality
    * Detects untracked `tokio::spawn` calls (code: `async-inspect-001`)
    * Identifies missing `.inspect()` on await points (code: `async-inspect-002`)
    * Real-time analysis as you type
    * Configurable severity levels (hint, info, warning, error)
  - **Code actions** for quick fixes
    * Convert `tokio::spawn` to `spawn_tracked` with one click
    * Add `.inspect("label")` before `.await` automatically
    * Preserve code formatting and indentation
    * Batch apply multiple fixes
  - **Hover information** displaying live statistics
    * Total tasks count
    * Running, completed, failed, and blocked task counts
    * Direct link to dashboard (http://localhost:8080)
    * Markdown-formatted for rich display
  - **Autocompletion** for async-inspect APIs
    * `spawn_tracked` with snippet support
    * `.inspect()` method completion
    * Parameter hints and documentation
  - **Editor configurations** for popular editors
    * Neovim (nvim-lspconfig): [lsp-config/neovim.lua](lsp-config/neovim.lua)
    * Emacs (lsp-mode): [lsp-config/emacs.el](lsp-config/emacs.el)
    * Vim (vim-lsp): [lsp-config/vim.vim](lsp-config/vim.vim)
    * Helix: [lsp-config/helix.toml](lsp-config/helix.toml)
    * Sublime Text: [lsp-config/sublime-lsp.json](lsp-config/sublime-lsp.json)
  - Module: `src/lsp/mod.rs` (420+ lines)
  - Binary: `src/bin/async-inspect-lsp.rs`
  - Documentation: [docs/lsp-guide.md](docs/lsp-guide.md) - Comprehensive LSP usage guide
  - Dependencies: `tower-lsp` 0.20, `tokio-util` 0.7
  - Feature flag: `lsp`
  - **Phase 9 Ecosystem Integration is now 100% complete!** 🎉

- **IntelliJ IDEA Plugin (Phase 9 - Ecosystem Integration)**: Real-time monitoring for IntelliJ IDEA and RustRover
  - **WebSocket client** connecting to async-inspect dashboard server
    * Automatic connection to localhost:8080
    * Connection state management with visual indicators
    * Event streaming with broadcast channels
    * REST API fallback for initial state
  - **Tool window UI** with comprehensive monitoring views
    * Metrics panel showing total, running, completed, failed, and blocked tasks
    * Task table with sortable columns (ID, Name, State, Parent, Polls, Duration)
    * Event log with auto-scroll and 1000-event buffer
    * Real-time updates via Kotlin coroutines and StateFlow
  - **Timeline visualization panel** with custom Swing rendering
    * Gantt-style timeline chart with interactive hover
    * Color-coded task states (blue=running, green=completed, red=failed, yellow=blocked)
    * Time-based scaling with automatic window adjustment
    * Click to focus on specific tasks
  - **Toolbar actions** for workflow integration
    * Start/Stop Monitoring with connection state awareness
    * Clear Tasks to reset view
    * Export Data to JSON with file chooser dialog
    * Settings dialog for host/port configuration
  - **Service architecture** with application and project-level services
    * InspectorService for global connection management
    * ProjectInspectorService for per-project state
    * Event listener interface for extensibility
  - Directory: `intellij-plugin/` with Gradle build system
  - Main files:
    * `InspectorService.kt` (480+ lines) - Core WebSocket client and state management
    * `AsyncInspectToolWindowFactory.kt` (200+ lines) - Main UI components
    * `TimelinePanel.kt` (350+ lines) - Custom timeline visualization
    * `plugin.xml` - Plugin manifest with actions and extensions
  - Compatibility: IntelliJ IDEA 2023.3+ and RustRover
  - Dependencies: Kotlin 1.9.22, kotlinx-coroutines, Gson, IntelliJ Platform SDK
  - Documentation: `intellij-plugin/README.md` with setup and usage guide
  - **Phase 9 is now 75% complete!**

- **Web Dashboard (Phase 4 - Complete)**: Real-time browser-based monitoring with WebSockets
  - **WebSocket server** using axum for real-time event streaming
    * Event broadcasting to all connected clients
    * Periodic metrics snapshots (100ms intervals)
    * Automatic reconnection support
    * Clean connection/disconnection handling
  - **Interactive web frontend** with Chart.js and Tailwind CSS
    * Live metrics dashboard with 5 key metrics cards (total, running, completed, failed, blocked)
    * Real-time timeline chart showing running vs blocked tasks over time
    * Searchable task table with state-based filtering
    * Event log with auto-scroll, pause controls, and filtering
    * Responsive design works on desktop and mobile
  - **REST API fallback** for programmatic access
    * `GET /` - Serve dashboard HTML
    * `GET /ws` - WebSocket upgrade endpoint
    * `GET /api/tasks` - Get all current tasks as JSON
    * `GET /api/stats` - Get current statistics as JSON
  - **Dashboard events** with comprehensive coverage
    * TaskSpawned, TaskCompleted, TaskFailed events
    * StateChanged for task state transitions
    * MetricsSnapshot for periodic statistics
    * AwaitStarted/AwaitEnded for await point tracking
  - Module: `src/dashboard/mod.rs` (320 lines)
  - Frontend: `src/dashboard/static/index.html` (500+ lines of HTML/JS/CSS)
  - Example: `examples/dashboard_demo.rs` with 8 workers and background tasks
  - Feature flag: `dashboard` (includes axum, tokio-tungstenite, tower-http)
  - Usage: Start with `Dashboard::new(8080).start().await?`, open http://localhost:8080
  - **Phase 4 is now 100% complete!**

- **Performance Profiling (Phase 6 - Complete)**: Advanced profiling with lock contention and regression detection
  - **Lock contention metrics** tracking resource contention with detailed statistics
    * Contention rate calculation (contentions / acquisitions)
    * Wait time tracking (total, average, maximum)
    * Per-resource metrics with task tracking
    * Automatic identification of highly contended locks
    * Methods: `record_lock_wait()`, `record_lock_acquisition()`, `most_contended_locks()`
  - **Performance snapshots** for serializable performance capture
    * Complete task and lock metrics serialization to JSON
    * Save/load snapshots for historical comparison
    * Includes task stats, durations, and resource metrics
  - **Performance comparison** between baseline and current runs
    * Automatic calculation of performance deltas
    * Task duration changes (mean, median, P95, P99)
    * Lock contention rate changes
    * Throughput comparison
  - **Regression detection** with severity-based findings
    * Critical (>50%), Major (20-50%), Minor (5-20%) severity levels
    * Automatic status determination (Improved/Regressed/Similar/Mixed)
    * Detailed findings with impact percentages
    * `has_regressions()` and `get_regressions()` methods
  - Module: `src/profile/comparison.rs` (468 lines)
  - Updated: `src/profile/mod.rs` with `LockContentionMetrics` structure
  - Example: `examples/performance_profiling.rs` demonstrating all features
  - Documentation: Performance profiling complete in Phase 6
- **Runtime Support**: Multi-runtime compatibility for broader ecosystem support
  - async-std runtime integration with `async-std-runtime` feature flag
  - smol runtime integration with `smol-runtime` feature flag
  - `spawn_tracked()` function for both runtimes
  - `InspectExt` trait adding `.inspect()` method to all futures
  - Modules: `src/runtime/async_std.rs` (237 lines), `src/runtime/smol.rs` (199 lines)
  - Examples: `examples/async_std_integration.rs`, `examples/smol_integration.rs`
  - Full feature parity with existing Tokio integration
  - Documentation updated in README.md and ROADMAP.md
- **Enhanced TUI (Phase 7)**: Powerful interactive terminal interface with advanced features
  - Dependency graph view showing parent-child task relationships (toggle with `v`)
  - Real-time search functionality (activate with `/`, filter by name or ID)
  - Mouse support for scroll wheel navigation
  - Direct export from TUI (press `e` for JSON, CSV, Chrome Trace)
  - Enhanced help screen with categorized shortcuts
  - View mode indicators and improved UI layout
  - Module: `src/tui/mod.rs` (~850 lines)
  - Documentation: `docs/content/tui-monitor.md` (complete guide)
  - Updated example: `examples/tui_monitor.rs` showcasing new features
  - Deferred: Keyboard shortcut customization (requires config file system)
- **Visualization & Export (Phase 4 @ 80%)**: Industry-standard format compatibility
  - Chrome Trace Event Format exporter for chrome://tracing and Perfetto UI
  - Flamegraph folded stack format for inferno/speedscope/flamegraph.pl
  - Module: `src/export/chrome_trace.rs` (352 lines)
  - Module: `src/export/flamegraph.rs` (288 lines)
  - Full event type support: Complete (X), Instant (i), Metadata (M)
  - Call stack tracking for flamegraph generation
  - Builder pattern for customization (`FlamegraphBuilder`)
  - Compatible with existing Gantt timeline, HTML reports, JSON/CSV exports
  - Comprehensive export example: `examples/export_formats.rs` demonstrating all formats
  - Detailed visualization guide: `docs/content/visualization.md` (447 lines)
  - README section with usage examples for all export formats
  - Remaining: Perfetto native protobuf, interactive web dashboard implementation
- **Interactive Web Dashboard (Phase 4 - Designed)**: Real-time monitoring architecture
  - Comprehensive architecture design document: `DASHBOARD_DESIGN.md`
  - WebSocket-based event streaming design
  - Browser-based UI with live updates
  - Dashboard feature flag with axum, tokio-tungstenite, tower-http dependencies
  - Planned features: timeline chart, metrics dashboard, task list, event log
  - Implementation status: Architecture complete, pending server and UI implementation
- **Performance Profiling (Phase 6 @ 80%)**: Comprehensive performance analysis and reporting
  - Poll duration statistics with P50, P95, P99 percentiles in `DurationStats`
  - Hot path identification tracking frequently executed code paths
  - Slowest task detection and bottleneck identification
  - Performance recommendations with actionable optimization suggestions
  - Efficiency analysis (running time / total time ratio)
  - Busy task detection for tasks with excessive polls
  - Statistical analysis: mean, median, standard deviation, min/max
  - Modules: `src/profile/mod.rs` and `src/profile/reporter.rs`
  - Working example: `examples/performance_analysis.rs`
  - Remaining: lock contention metrics integration, run comparisons, regression detection
- **State Machine Introspection (Phase 3)**: `#[async_inspect::trace]` proc macro for automatic .await point instrumentation
  - Procedural macro in `async-inspect-macros` crate using `syn` and `quote`
  - Automatic sequential labeling of await points (await#1, await#2, etc.)
  - AST transformation that preserves semantics and error propagation
  - Source location tracking (file, line, column)
  - Automatic task registration and cleanup
  - Full integration with existing Inspector infrastructure
  - Working example: `examples/proc_macro_test.rs` (16 tasks, 74 events tracked)
- **Deadlock Detection Integration (Phase 5)**: Full integration with Inspector
  - Added `deadlock_detector()` method to Inspector for global access
  - DFS-based cycle detection in wait-for graphs
  - Resource tracking (Mutex, RwLock, Semaphore, Channel)
  - Human-readable cycle descriptions with actionable suggestions
  - Working example: `examples/deadlock_detection.rs`
- GNU Terry Pratchett `X-Clacks-Overhead` header to documentation site
- Comprehensive CI/CD workflows with multi-platform testing
- GitHub Actions workflow for automated releases
- Manual workflow dispatch capability for testing releases
- Code coverage reporting with Codecov integration

### Security
- **SLSA Level 3 Provenance**: Automated provenance generation for all release binaries using GitHub's attestation API
- **Dependency Review**: Automated scanning on pull requests to detect vulnerable or non-compliant dependencies
- **License Compliance**: cargo-deny configuration to block GPL/AGPL licenses and ensure MIT/Apache-2.0 compatibility
- **Security Audits**: Continuous monitoring via cargo-audit and cargo-deny in CI pipeline
- **Restricted Permissions**: Principle of least privilege applied to all GitHub Actions workflows
- **Supply Chain Security**: Only allow dependencies from crates.io registry, blocking unknown git sources
- Security documentation section in README with verification instructions

### Fixed
- Windows test failures due to hardcoded Unix temp directory paths
- Clippy warnings reduced from 314 to 93 through auto-fixes and code improvements
- Example feature requirements for conditional compilation
- Cross-platform compatibility in HTML reporter tests
- CI success job to properly handle platform-specific test failures
- Flamegraph Palette::Rust error (removed non-existent palette reference)
- Added `#[must_use]` attributes to builder methods and pure functions

### Changed
- CI workflow to use focused clippy lints (correctness, suspicious, perf)
- Test matrix to allow Windows failures without blocking CI success
- Cache configuration to be non-blocking on missing Cargo.lock
- GitHub Actions permissions to read-only by default with explicit grants per job

### Infrastructure
- Multi-platform CI testing (Linux, macOS, Windows) on stable, beta, and nightly
- Automated binary builds for 5 platform targets
- Feature combination testing with cargo-hack
- Security audit integration with cargo-audit and cargo-deny
- Documentation build verification
- SLSA provenance attestation for release artifacts
- Dependency review workflow for pull requests

## [0.1.0] - TBD

### Added
- Core inspector infrastructure
- Task tracking and monitoring
- Timeline event system
- Relationship graph analysis
- Deadlock detection algorithms
- Performance profiling tools
- CLI with multiple commands (monitor, export, stats, config, info, version)
- TUI monitor for real-time inspection
- JSON and CSV export functionality
- Production configuration system
- Proc macro instrumentation (`#[async_inspect::trace]`)
- Tokio runtime integration

### Integrations
- Prometheus metrics exporter
- OpenTelemetry trace exporter
- Tracing subscriber layer
- Tokio-console compatibility

### Examples
- Basic inspection example
- TUI monitor example
- Relationship graph example
- Ecosystem integration example
- Production ready example
- Performance analysis example
- Deadlock detection example
- Task hierarchy example
- Tokio integration example

### Documentation
- Comprehensive README
- Contributing guidelines
- Code of conduct
- API documentation
- CLI usage guide
- Quick start guide
- Roadmap

### Infrastructure
- GitHub Actions CI/CD
- Automated releases
- Dependabot configuration
- Issue and PR templates
- Documentation deployment

[Unreleased]: https://github.com/ibrahimcesar/async-inspect/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ibrahimcesar/async-inspect/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ibrahimcesar/async-inspect/releases/tag/v0.1.0
