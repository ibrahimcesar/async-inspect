# Getting Started with async-inspect

**async-inspect** is a powerful inspection and debugging tool for async Rust applications. Think of it as X-ray vision for your async code—see exactly what your futures are doing, where they're stuck, and why.

## 🎯 What is async-inspect?

async-inspect provides:

- **🔍 Task Inspection**: Real-time visibility into async task execution
- **📊 Performance Analysis**: Identify bottlenecks and slow operations
- **💀 Deadlock Detection**: Find circular dependencies before they bite
- **📈 Execution Timeline**: Visualize the async execution flow
- **🔗 Relationship Graphs**: Understand task dependencies and communication
- **🌐 Ecosystem Integration**: Works with Prometheus, OpenTelemetry, Grafana

## Installation

Add async-inspect to your `Cargo.toml`:

```toml
[dependencies]
async-inspect = "0.0.1"
```

For TUI monitoring:

```toml
[dependencies]
async-inspect = { version = "0.0.1", features = ["cli"] }
```

## Quick Start

### 1. Annotate Your Async Functions

Use the `#[async_inspect::trace]` attribute to automatically instrument your async functions:

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn fetch_user(id: u64) -> User {
    let profile = fetch_profile(id).await;
    let posts = fetch_posts(id).await;
    User { profile, posts }
}
```

### 2. View Real-Time Stats

```rust
use async_inspect::prelude::*;

#[tokio::main]
async fn main() {
    // Your async code here
    my_async_function().await;

    // Print summary
    Reporter::global().print_summary();
}
```

### 3. Launch the TUI Monitor (Optional)

```bash
cargo install async-inspect
async-inspect monitor
```

## What's Next?

- Core Concepts - Understand the fundamentals
- Examples - See async-inspect in action
- CLI Guide - Learn the command-line interface
- Production Use - Deploy with confidence
- Ecosystem Integration - Connect with your tools

## Community

- [GitHub Repository](https://github.com/ibrahimcesar/async-inspect)
- [crates.io](https://crates.io/crates/async-inspect)
- [API Documentation](https://docs.rs/async-inspect)

## License

async-inspect is licensed under the MIT License.
