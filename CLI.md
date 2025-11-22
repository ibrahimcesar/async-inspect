# async-inspect CLI

Command-line interface for inspecting and monitoring async Rust applications.

## Installation

### From Source

```bash
cargo install --path . --features cli
```

### From Crates.io (when published)

```bash
cargo install async-inspect
```

## Commands

### `monitor` - Interactive TUI Monitor

Launch the real-time Terminal User Interface for monitoring async tasks.

```bash
async-inspect monitor [OPTIONS]
```

**Options:**
- `-i, --interval <MS>` - Update interval in milliseconds (default: 100)
- `-v, --verbose` - Enable verbose output

**Features:**
- Real-time task monitoring
- Sort by: ID, Name, Duration, State, Poll Count
- Filter by: All, Running, Completed, Failed, Blocked
- Keyboard navigation: `q` (quit), `s` (sort), `f` (filter), `↑↓` (navigate), `h/?` (help)

**Example:**
```bash
# Launch TUI with default settings
async-inspect monitor

# Launch with 50ms update interval
async-inspect monitor -i 50
```

### `export` - Export Task Data

Export task and event data to JSON or CSV formats.

```bash
async-inspect export -f <FORMAT> -o <FILE> [--with-events]
```

**Options:**
- `-f, --format <FORMAT>` - Output format: `json` or `csv` (required)
- `-o, --output <FILE>` - Output file path (required)
- `--with-events` - Export events separately (CSV only)

**Examples:**
```bash
# Export to JSON
async-inspect export -f json -o trace.json

# Export to CSV with separate events file
async-inspect export -f csv -o tasks.csv --with-events
```

**Output:**
- **JSON**: Single file with tasks, events, and metadata
- **CSV**: `tasks.csv` with task data, optionally `tasks_events.csv` with event timeline

### `stats` - Show Statistics

Display current task statistics and performance metrics.

```bash
async-inspect stats [OPTIONS]
```

**Options:**
- `-d, --detailed` - Show detailed performance metrics
- `-v, --verbose` - Enable verbose output

**Examples:**
```bash
# Show basic statistics
async-inspect stats

# Show detailed performance analysis
async-inspect stats --detailed
```

### `config` - Configure Settings

Configure async-inspect for different environments.

```bash
async-inspect config <MODE> [OPTIONS]
```

**Modes:**
- `production` - 1% sampling, 1k events, minimal tracking
- `development` - Full sampling, 10k events, full tracking
- `debug` - Unlimited tracking for deep debugging
- `custom` - Custom configuration

**Custom Options:**
- `-s, --sampling-rate <N>` - Track 1 in N tasks
- `-e, --max-events <N>` - Maximum events to retain
- `-t, --max-tasks <N>` - Maximum tasks to track

**Examples:**
```bash
# Configure for production
async-inspect config production

# Configure for development
async-inspect config development

# Custom configuration
async-inspect config custom -s 10 -e 5000 -t 1000
```

**Configuration Presets:**

| Mode | Sampling | Max Events | Max Tasks | Await Tracking | HTML |
|------|----------|------------|-----------|----------------|------|
| Production | 1% (1 in 100) | 1,000 | 500 | ❌ | ❌ |
| Development | 100% (all) | 10,000 | 1,000 | ✅ | ✅ |
| Debug | 100% (all) | Unlimited | Unlimited | ✅ | ✅ |

### `info` - Show Information

Display comprehensive information about async-inspect configuration, state, and capabilities.

```bash
async-inspect info
```

Shows:
- Version and description
- Current configuration
- Task statistics
- Overhead measurements
- Available features
- Quick start guide

**Example:**
```bash
async-inspect info
```

### `version` - Show Version

Display version information and enabled features.

```bash
async-inspect version
```

**Example:**
```bash
async-inspect version
# Output:
# async-inspect 0.0.1
# X-ray vision for async Rust
#
# Features enabled:
#   • cli (TUI support)
#   • tokio
```

## Global Options

All commands support these global options:

- `-v, --verbose` - Enable verbose output
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Usage in Applications

The CLI is designed to work with applications that use the async-inspect library:

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn my_function() {
    // Your async code here
}

#[tokio::main]
async fn main() {
    // Your application code
    my_function().await;

    // Launch TUI from within your app (optional)
    #[cfg(feature = "cli")]
    async_inspect::tui::run_tui(Inspector::global().clone()).unwrap();
}
```

Then run with the CLI:

```bash
# Run your application
cargo run

# Or use the CLI to monitor (when integrated)
async-inspect monitor
```

## Environment Variables

The CLI respects standard Rust environment variables:

- `RUST_LOG` - Set log level (e.g., `RUST_LOG=debug async-inspect monitor`)
- `RUST_BACKTRACE` - Enable backtrace on panic

## Examples

### Basic Monitoring Workflow

```bash
# 1. Configure for development
async-inspect config development

# 2. Run your application with tracing enabled
cargo run --example my_app

# 3. View statistics
async-inspect stats --detailed

# 4. Export for analysis
async-inspect export -f json -o trace.json
```

### Production Monitoring Workflow

```bash
# 1. Configure for production (minimal overhead)
async-inspect config production

# 2. Launch TUI monitor
async-inspect monitor

# 3. Export sampled data
async-inspect export -f csv -o production.csv --with-events
```

### Debugging Workflow

```bash
# 1. Configure for debug mode (unlimited tracking)
async-inspect config debug

# 2. Launch detailed monitoring
async-inspect monitor -i 50

# 3. View detailed performance metrics
async-inspect stats --detailed

# 4. Export complete trace
async-inspect export -f json -o debug_trace.json
```

## Features

The CLI supports these optional features:

- `cli` - Enable TUI monitoring (required for `monitor` command)
- `tokio` - Tokio runtime integration

Build with all features:

```bash
cargo build --features full
```

## Tips

1. **Performance Impact**: Use production mode (`config production`) in production to minimize overhead
2. **Data Size**: Export regularly to prevent memory growth with unlimited tracking
3. **TUI Navigation**: Press `h` or `?` in the TUI for help
4. **Sampling**: Adjust sampling rate based on your needs (higher = less overhead)
5. **Export Formats**: Use JSON for complete data, CSV for spreadsheet analysis

## Troubleshooting

### "No tasks tracked yet"

This means no async functions have been instrumented. Add `#[async_inspect::trace]` to your async functions:

```rust
#[async_inspect::trace]
async fn my_function() {
    // ...
}
```

### TUI not updating

Check that:
1. Tasks are being tracked with `#[async_inspect::trace]`
2. Your async runtime is running
3. Update interval is not too high (`-i` option)

### High memory usage

Configure limits:
```bash
async-inspect config custom -e 10000 -t 1000
```

Or use production mode:
```bash
async-inspect config production
```

## Graph Visualization

The `relationship_graph` example demonstrates comprehensive task relationship analysis:

```bash
cargo run --example relationship_graph
```

**Features:**
- Text-based graph visualization
- Critical path analysis (longest dependency chain)
- Transitive dependency tracking
- Shared resource contention detection
- Channel communication pair mapping
- Deadlock detection via cycle finding
- GraphViz DOT format export

**Export to PNG:**
```bash
# Run example and save DOT output
cargo run --example relationship_graph > graph.dot

# Extract just the DOT content (between digraph { ... })
# Then generate visualization
dot -Tpng graph.dot -o graph.png
```

## See Also

- [Main README](README.md) - Library documentation
- [Examples](examples/) - Example applications
- [API Documentation](https://docs.rs/async-inspect) - API reference
