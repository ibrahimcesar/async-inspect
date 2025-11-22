# Installation

Get started with async-inspect in minutes.

## Quick Install

### From crates.io

```bash
cargo install async-inspect
```

### From Source

```bash
git clone https://github.com/ibrahimcesar/async-inspect
cd async-inspect
cargo install --path .
```

## Verify Installation

```bash
async-inspect --version
```

You should see:
```
async-inspect 0.1.0
```

## Add to Your Project

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
async-inspect = "0.1.0"
```

### With Features

```toml
[dependencies]
async-inspect = { version = "0.1.0", features = ["cli", "prometheus-export"] }
```

Available features:
- `cli` - Terminal UI and CLI tools
- `tokio` - Tokio runtime integration (default)
- `tracing-sub` - Tracing subscriber integration
- `prometheus-export` - Prometheus metrics
- `opentelemetry-export` - OpenTelemetry traces
- `full` - All features enabled

## Development Dependencies

For examples and testing:

```toml
[dev-dependencies]
tokio = { version = "1.35", features = ["full"] }
async-inspect = { version = "0.1.0", features = ["cli"] }
```

## System Requirements

- **Rust**: 1.70 or later
- **OS**: Linux, macOS, Windows
- **Optional**: Node.js 20+ (for documentation site)

## VS Code Extension

Install the async-inspect extension from VS Code Marketplace:

1. Open VS Code
2. Go to Extensions (Cmd+Shift+X / Ctrl+Shift+X)
3. Search for "async-inspect"
4. Click Install

Or install from VSIX:

```bash
code --install-extension async-inspect-0.1.0.vsix
```

## Troubleshooting

### Command Not Found

If `async-inspect` command is not found after installation:

```bash
# Add cargo bin to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Or use full path
~/.cargo/bin/async-inspect --version
```

### Build Errors

If you get compilation errors:

1. Update Rust:
   ```bash
   rustup update stable
   ```

2. Clean build cache:
   ```bash
   cargo clean
   cargo install async-inspect
   ```

### Feature Conflicts

If you see feature-related errors, make sure you're using compatible features:

```toml
# ✅ GOOD
[dependencies]
async-inspect = { version = "0.1.0", features = ["cli", "tokio"] }

# ❌ BAD - conflicting features
[dependencies]
async-inspect = { version = "0.1.0", features = ["cli"], default-features = false }
tokio = { version = "1.0", features = ["rt"] }  # May conflict
```

## Next Steps

- [Quick Start Guide](./quickstart) - Get up and running in 5 minutes
- [CLI Usage](./cli-usage) - Learn the command-line interface
- [Examples](./examples) - See it in action

---

[← Back to Intro](./intro) | [Quick Start →](./quickstart)
