# Contributing to async-inspect 🔍

Thank you for your interest in contributing to async-inspect! We're excited to have you join us in making async Rust debugging better for everyone.

## 🚀 Project Vision

Make async Rust as easy to debug as synchronous code by providing complete visibility into state machines, execution flow, and task interactions.

## 🤝 Ways to Contribute

### 🐛 Report Bugs

Found a bug? Please [open an issue](https://github.com/ibrahimcesar/async-inspect/issues/new) with:
- Clear description of the problem
- Minimal reproduction code
- Expected vs actual behavior
- Your environment (OS, Rust version, async runtime)

### 💡 Suggest Features

Have an idea? [Start a discussion](https://github.com/ibrahimcesar/async-inspect/discussions) or open an issue with:
- The problem you're trying to solve
- Your proposed solution
- Why this would be valuable to others
- Potential implementation approach

### 📝 Improve Documentation

Documentation improvements are always welcome! This includes:
- Fixing typos or unclear explanations
- Adding examples
- Improving API documentation
- Writing guides or tutorials
- Creating blog posts or videos

### 💻 Contribute Code

Code contributions are highly valued! See the development guide below.

## 🏗️ Development Setup

### Prerequisites

- Rust 1.70 or later
- Node.js 20+ (for documentation site)
- Git

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/async-inspect
cd async-inspect

# Build the project
cargo build

# Run tests
cargo test

# Run examples
cargo run --example basic_inspection

# Run CLI
cargo run --bin async-inspect -- --help

# Build documentation
cargo doc --open
```

### Running Examples

We have many examples demonstrating different features:

```bash
# Basic usage
cargo run --example basic_inspection

# TUI monitor
cargo run --example tui_monitor --features cli

# Relationship graphs
cargo run --example relationship_graph

# Ecosystem integration
cargo run --example ecosystem_integration

# Production configuration
cargo run --example production_ready

# Performance analysis
cargo run --example performance_analysis

# Deadlock detection
cargo run --example deadlock_detection
```

## 📋 Development Workflow

### 1. Find or Create an Issue

- Check existing issues for something to work on
- Or create a new issue describing what you'd like to add/fix
- Discuss approach before starting large changes

### 2. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

### 3. Make Your Changes

- Write clear, idiomatic Rust code
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific example
cargo run --example YOUR_EXAMPLE

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build with all features
cargo build --all-features
```

### 5. Commit Your Changes

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "feat: add new deadlock detection algorithm"
git commit -m "fix: resolve panic in task tracking"
git commit -m "docs: improve README examples"
git commit -m "chore: update dependencies"
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

### 6. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a PR on GitHub with:
- Clear title describing the change
- Description explaining what and why
- Link to related issues
- Screenshots/examples if relevant

## 🎨 Code Style Guidelines

### Rust Code

- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Use descriptive variable and function names
- Add doc comments for public APIs
- Keep functions focused and small
- Prefer clarity over cleverness

### Documentation

- Use clear, concise language
- Include code examples
- Explain the "why" not just the "what"
- Keep examples runnable and tested

### Commit Messages

- Use present tense ("add feature" not "added feature")
- Use imperative mood ("move cursor to..." not "moves cursor to...")
- Reference issues and PRs liberally

## 🧪 Testing

### Test Requirements

- All new features must have tests
- Bug fixes should include regression tests
- Aim for high code coverage
- Test both success and error cases

### Test Types

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Doc tests
cargo test --doc

# All tests with all features
cargo test --all-features
```

### Example Tests

Add tests to the appropriate module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = TaskInfo::new("test".to_string());
        assert_eq!(task.name, "test");
        assert_eq!(task.state, TaskState::Pending);
    }

    #[tokio::test]
    async fn test_async_tracking() {
        // Your async test here
    }
}
```

## 📦 Project Structure

```
async-inspect/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # CLI entry point
│   ├── config.rs           # Configuration
│   ├── inspector/          # Core inspection logic
│   ├── task/               # Task tracking
│   ├── timeline/           # Event timeline
│   ├── graph/              # Relationship graphs
│   ├── deadlock/           # Deadlock detection
│   ├── profile/            # Performance profiling
│   ├── export/             # Data export
│   ├── tui/                # Terminal UI
│   ├── integrations/       # Ecosystem integrations
│   │   ├── tracing_layer.rs
│   │   ├── prometheus.rs
│   │   ├── opentelemetry.rs
│   │   └── tokio_console.rs
│   └── reporter/           # Reporting
├── examples/               # Usage examples
├── async-inspect-macros/   # Proc macros
├── docs/                   # Docusaurus site
└── tests/                  # Integration tests
```

## 🎯 Areas for Contribution

### High Priority

- ✅ Core infrastructure (mostly complete)
- ✅ Ecosystem integration (complete)
- 🔄 Performance optimization
- 🔄 Additional examples
- 🔄 Documentation improvements

### Medium Priority

- Advanced deadlock detection algorithms
- Web-based dashboard
- Browser-based timeline viewer
- Additional runtime support (async-std, smol)
- Grafana dashboard templates

### Future Ideas

- VS Code extension
- Chrome DevTools integration
- Distributed tracing support
- AI-powered anomaly detection
- Historical trace diff tool

## 🔍 Code Review Process

### What We Look For

- Correct functionality
- Good test coverage
- Clear documentation
- Following code style
- No breaking changes (or justified ones)
- Performance considerations

### Timeline

- Initial review: Usually within 2-3 days
- Expect iterative feedback
- Most PRs merge within 1-2 weeks

## 🌟 Recognition

All contributors are recognized in:
- README contributors section
- Release notes
- Documentation

We use [All Contributors](https://allcontributors.org/) to track contributions.

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

## 💬 Communication

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, design discussions, RFCs
- **Pull Requests**: Code contributions
- **Email**: For security issues, contact ibrahim@ibrahimcesar.com

## 🙏 Thank You!

Every contribution, no matter how small, helps make async-inspect better. Whether you're fixing a typo, adding a feature, or just providing feedback—thank you for being part of this project!

## ❓ Questions?

Not sure where to start? Feel free to:
- Open a discussion
- Comment on an existing issue
- Reach out to maintainers

We're here to help! 🚀
