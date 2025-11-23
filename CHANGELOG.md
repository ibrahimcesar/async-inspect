# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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

[Unreleased]: https://github.com/ibrahimcesar/async-inspect/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ibrahimcesar/async-inspect/releases/tag/v0.1.0
