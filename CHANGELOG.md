# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GNU Terry Pratchett `X-Clacks-Overhead` header to documentation site
- Comprehensive CI/CD workflows with multi-platform testing
- GitHub Actions workflow for automated releases
- Manual workflow dispatch capability for testing releases
- Code coverage reporting with Codecov integration

### Fixed
- Windows test failures due to hardcoded Unix temp directory paths
- Clippy warnings across all lint categories
- Example feature requirements for conditional compilation
- Cross-platform compatibility in HTML reporter tests
- CI success job to properly handle platform-specific test failures

### Changed
- CI workflow to use focused clippy lints (correctness, suspicious, perf)
- Test matrix to allow Windows failures without blocking CI success
- Cache configuration to be non-blocking on missing Cargo.lock

### Infrastructure
- Multi-platform CI testing (Linux, macOS, Windows) on stable, beta, and nightly
- Automated binary builds for 5 platform targets
- Feature combination testing with cargo-hack
- Security audit integration with cargo-audit
- Documentation build verification

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
