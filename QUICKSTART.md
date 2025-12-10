# async-inspect Quickstart Guide

Get started debugging async Rust in under 5 minutes.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
async-inspect = "0.1"
```

## 1. Basic Usage - Trace Your Functions

Add the `#[trace]` attribute to any async function:

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn fetch_data(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    response.text().await
}

#[async_inspect::trace]
async fn process_users() {
    let users = fetch_data("https://api.example.com/users").await;
    // ... process users
}
```

Run your app and you'll see task execution in the TUI:

```bash
cargo run
```

## 2. Launch the TUI Monitor

Run the built-in terminal UI to monitor tasks in real-time:

```bash
# In your code, add before your async runtime starts:
async_inspect::tui::run_tui().await;

# Or run standalone:
cargo run --example tui_monitor
```

**TUI Keyboard Shortcuts:**
- `↑/↓` or `j/k` - Navigate tasks
- `/` - Search tasks
- `f` - Filter by state
- `s` - Sort tasks
- `d` - Show task details
- `?` - Help
- `q` - Quit

## 3. Find Deadlocks

async-inspect automatically detects potential deadlocks:

```rust
use async_inspect::sync::Mutex;  // Use tracked Mutex

let lock_a = Mutex::new(1);
let lock_b = Mutex::new(2);

// This pattern can deadlock - async-inspect will warn you!
let task1 = async {
    let a = lock_a.lock().await;
    let b = lock_b.lock().await;  // Warning: potential deadlock
};

let task2 = async {
    let b = lock_b.lock().await;
    let a = lock_a.lock().await;  // Cycle detected!
};
```

## 4. Profile Slow Tasks

Find performance bottlenecks:

```rust
use async_inspect::task::{TaskFilter, TaskSortBy, sort_tasks};

// Filter for slow tasks
let filter = TaskFilter::new()
    .with_min_duration(Duration::from_millis(100));

let slow_tasks = filter.apply(&inspector.tasks());

// Sort by duration
sort_tasks(&mut tasks, TaskSortBy::Duration, SortDirection::Descending);

for task in slow_tasks.iter().take(10) {
    println!("{}: {:?}", task.name(), task.duration());
}
```

## 5. Production Mode

For production use with minimal overhead:

```rust
use async_inspect::config::Config;
use async_inspect::timeline::{Timeline, TimelineConfig};

fn setup_production() {
    let config = Config::global();

    // Enable production mode (low overhead)
    config.production_mode();

    // Or fine-tune settings:
    config.set_sampling_rate(100);  // Track 1 in 100 tasks
    config.enable_adaptive_sampling();
    config.set_adaptive_target_overhead_ms(1.0);  // 1ms overhead budget

    // Use ring buffer for bounded memory
    let timeline = Timeline::with_config(TimelineConfig::production());
}
```

## 6. Export Data

Export task data for analysis:

```rust
use async_inspect::export::JsonExporter;

// Export to JSON
let json = JsonExporter::export_tasks(&tasks)?;
std::fs::write("tasks.json", json)?;

// Export timeline
let timeline_json = JsonExporter::export_timeline(&timeline)?;
```

## Common Use Cases

### Debug a Hanging Task

```rust
// 1. Find blocked tasks
let blocked = TaskFilter::new()
    .with_state(TaskState::Blocked)
    .apply(&inspector.tasks());

// 2. Check what they're waiting on
for task in blocked {
    println!("{} blocked for {:?}", task.name(), task.duration());
    for event in timeline.events_for_task_vec(task.id()) {
        if let EventKind::AwaitEntered { name } = &event.kind {
            println!("  Waiting on: {}", name);
        }
    }
}
```

### Monitor Lock Contention

```rust
use async_inspect::sync::Mutex;

let mutex = Mutex::new(data);

// ... use mutex ...

// Check contention
let metrics = mutex.metrics();
if metrics.contention_rate() > 0.5 {
    println!("High contention! {} contentions / {} acquisitions",
        metrics.contentions, metrics.acquisitions);
}
```

### Track Channel Backpressure

```rust
use async_inspect::channel::mpsc;

let (tx, rx) = mpsc::channel(100);

// Check if channel is backing up
let stats = tx.stats();
if stats.pending > 80 {
    println!("Channel nearly full: {}/100", stats.pending);
}
```

## Web Dashboard

Enable the dashboard feature for a web-based UI:

```toml
[dependencies]
async-inspect = { version = "0.1", features = ["dashboard"] }
```

```rust
// Start dashboard on port 3000
async_inspect::dashboard::start(3000).await;
```

Open `http://localhost:3000` in your browser.

## IDE Integration

### VS Code

Install the `async-inspect` extension from the marketplace, or:

```bash
cd vscode-extension
npm install && npm run package
code --install-extension async-inspect-*.vsix
```

### JetBrains (IntelliJ, CLion, RustRover)

Install from the JetBrains Marketplace or:

```bash
cd intellij-plugin
./gradlew buildPlugin
# Install from intellij-plugin/build/distributions/
```

## Troubleshooting

### High Overhead

```rust
// Switch to production mode
Config::global().production_mode();

// Or enable adaptive sampling
Config::global().enable_adaptive_sampling();
```

### Too Many Tasks

```rust
// Limit tracked tasks
Config::global().set_max_tasks(1000);

// Use ring buffer for events
let timeline = Timeline::with_ring_buffer(10_000);
```

### Memory Usage

```rust
// Check memory stats
let stats = timeline.ring_stats().unwrap();
println!("Memory: {} bytes", timeline.memory_usage());
println!("Utilization: {:.1}%", stats.utilization * 100.0);
```

## Next Steps

- Read the [API Reference](API.md) for detailed documentation
- Check [examples/](examples/) for more use cases
- See [CONTRIBUTING.md](CONTRIBUTING.md) to contribute
- Report issues at [GitHub Issues](https://github.com/ibrahimcesar/async-inspect/issues)

## Feature Flags

```toml
# All features
async-inspect = { version = "0.1", features = ["full"] }

# Minimal (no TUI, no telemetry)
async-inspect = { version = "0.1", default-features = false, features = ["tokio"] }

# With dashboard
async-inspect = { version = "0.1", features = ["dashboard"] }
```
