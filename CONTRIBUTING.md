# Contributing to async-inspect 🔍

Thank you for your interest in contributing!

## 🚧 Project Status

This project is in **early development**. We're building foundational infrastructure for async debugging.

## 🎯 Vision

Make async Rust as easy to debug as synchronous code by providing complete visibility into state machines, execution flow, and task interactions.

## 🤝 How to Contribute

### Reporting Issues

- **Bugs**: Describe the issue with minimal reproduction
- **Feature Requests**: Explain the use case and why it's valuable
- **Questions**: Use Discussions for general questions

### Code Contributions

This is a complex project touching many areas:

1. **Compiler Integration** - Instrumenting async code
2. **Runtime Hooks** - Integrating with Tokio, async-std, etc.
3. **UI/UX** - TUI, web dashboard, CLI
4. **Algorithms** - Deadlock detection, performance analysis
5. **Visualization** - Timeline, dependency graphs

**Process:**
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a Pull Request

## 📋 Development Setup
```bash
# Clone
git clone https://github.com/yourusername/async-inspect
cd async-inspect

# Build
cargo build

# Run
cargo run -- --help

# Test
cargo test

# Run examples
cargo run --example basic_inspection
```

## 🎯 Priority Areas

### Phase 1 (Current)
- [ ] Core inspector architecture
- [ ] Basic task tracking
- [ ] Tokio runtime integration
- [ ] Simple TUI

### Future Phases
- [ ] State machine introspection
- [ ] Deadlock detection
- [ ] Performance profiling
- [ ] Web dashboard

## 🧪 Testing

Test async debugging tools is meta! We need:

- Unit tests for core logic
- Integration tests with real async code
- Manual testing with example apps
- Performance benchmarks

## 📝 Code Style

- Follow Rust conventions (`cargo fmt`)
- Pass `cargo clippy`
- Add documentation for public APIs
- Keep PRs focused and small

## 🏗️ Architecture Decisions

We're still deciding on:

- How to instrument code (proc macros vs compiler plugin)
- Runtime overhead (acceptable cost for debug builds)
- Cross-runtime compatibility
- Data collection and storage format

Join discussions to share your thoughts!

## 📜 License

By contributing, you agree that your contributions will be licensed under MIT OR Apache-2.0.

## 💬 Communication

- **Issues**: Bug reports, feature requests
- **Discussions**: General questions, design decisions
- **PRs**: Code contributions

## 🙏 Thank You!

Building developer tools is challenging but incredibly rewarding. Every contribution helps make async Rust easier for everyone!

└── tests/
    └── integration.rs (to be created)
