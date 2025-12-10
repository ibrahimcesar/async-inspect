# async-inspect Roadmap

**Vision**: The best debugging experience for async Rust.

This roadmap outlines planned features and improvements. We welcome community input—open an issue or discussion to share your ideas!

---

## v0.2.0 - Enhanced Debugging

**Focus**: Deeper insights into async behavior

### Planned Features

- [ ] **Await Point Source Maps** - Click-to-navigate from blocked await to source code location

### Improvements

- [x] Reduce memory overhead per tracked task (string interning, compact storage)
- [ ] Improve TUI responsiveness with 10,000+ tasks
- [x] Add task filtering by duration, state, or name pattern

---

## v0.3.0 - Production Observability

**Focus**: Safe for production use with minimal overhead
3
### Planned Features

- [x] **Adaptive Sampling** - Automatically adjust sampling rate based on load
- [x] **Ring Buffer Mode** - Fixed memory usage, keep only recent N events
- [ ] **Remote Dashboard** - Connect dashboard to remote running processes
- [ ] **Alerting** - Configurable alerts for deadlocks, slow tasks, or high contention

### Integrations

- [ ] AWS CloudWatch Metrics export
- [ ] Datadog APM integration
- [ ] Grafana Tempo trace export

---

## v0.4.0 - Distributed Tracing

**Focus**: Multi-service async debugging

### Planned Features

- [ ] **Trace Context Propagation** - Track async operations across service boundaries
- [ ] **Distributed Deadlock Detection** - Detect cross-service circular waits
- [ ] **Service Topology Map** - Visualize async call patterns between services

### Integrations

- [ ] gRPC/tonic automatic instrumentation
- [ ] HTTP client tracing (reqwest, hyper)
- [ ] Message queue tracing (Kafka, RabbitMQ, NATS)

---

## v1.0.0 - Stable Release

**Focus**: API stability and ecosystem maturity

### Goals

- [ ] Stable public API with semantic versioning guarantees
- [ ] Comprehensive documentation with video tutorials
- [ ] Published IDE extensions (VS Code Marketplace, JetBrains Marketplace)
- [ ] Battle-tested in production environments

---

## Future Ideas

These are longer-term ideas we're considering. Vote with a thumbs-up on the linked issues!

### Runtime Support
- [ ] Glommio integration (io_uring-based runtime)
- [ ] Monoio integration (thread-per-core runtime)
- [ ] Embassy integration (embedded async)

### Analysis & ML
- [ ] Anomaly detection for unusual task patterns
- [ ] Performance regression prediction
- [ ] Automatic bottleneck recommendations

### Developer Experience
- [ ] VS Code debugger integration (step through await points)
- [ ] Time-travel debugging (replay recorded sessions)
- [ ] Interactive REPL for inspecting live tasks

---

## Contributing

We'd love your help! Here's how to get involved:

1. **Feature Requests** - Open an issue describing your use case
2. **Bug Reports** - Include reproduction steps and `async-inspect` version
3. **Pull Requests** - See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines
4. **Documentation** - Help improve examples and guides

### Good First Issues

Look for issues labeled `good first issue` - these are great starting points for new contributors.

### Priority Areas

- Additional runtime support (async-std, smol are done; more welcome!)
- Performance optimizations
- Documentation improvements
- Example applications

---

## Version History

See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

| Version | Status | Highlights |
|---------|--------|------------|
| v0.1.0 | Current | Core tracking, deadlock detection, TUI, dashboard, IDE plugins, LSP |
| v0.2.0 | Planned | Auto lock tracking, channel viz, source maps |
| v0.3.0 | Planned | Production observability, cloud integrations |
| v0.4.0 | Planned | Distributed tracing |
| v1.0.0 | Planned | Stable API |

---

*Last updated: 2025-01-23*

*Have ideas? Open a [discussion](https://github.com/ibrahimcesar/async-inspect/discussions) or [issue](https://github.com/ibrahimcesar/async-inspect/issues)!*
