# async-inspect Quick Start Guide

## 🎯 What You Have Now

A **working async debugging tool** for Rust that tracks task execution, records events, and provides beautiful visualizations—all with manual instrumentation.

## 🚀 Installation

```bash
# Clone your repo
git clone https://github.com/ibrahimcesar/async-inspect
cd async-inspect

# Build
cargo build

# Run tests
cargo test

# Run example
cargo run --example simple_test
```

## 📖 Basic Usage

### 1. Add to your project

```toml
# Cargo.toml
[dependencies]
async-inspect = { path = "../async-inspect" }  # Update path as needed
tokio = { version = "1", features = ["full"] }
```

### 2. Instrument your code

```rust
use async_inspect::prelude::*;
use std::time::Duration;

async fn fetch_user(id: u64) -> User {
    // RAII guard automatically tracks this task
    let _guard = TaskGuard::new(format!("fetch_user({})", id));

    inspect_point!("start");

    // Simulate some async work
    let profile = fetch_profile(id).await;
    inspect_point!("profile_loaded");

    let posts = fetch_posts(id).await;
    inspect_point!("posts_loaded");

    inspect_point!("done");

    User { profile, posts }
}

async fn fetch_profile(id: u64) -> Profile {
    let _guard = TaskGuard::new(format!("fetch_profile({})", id));

    tokio::time::sleep(Duration::from_millis(100)).await;

    Profile { id, name: format!("User {}", id) }
}

async fn fetch_posts(id: u64) -> Vec<Post> {
    let _guard = TaskGuard::new(format!("fetch_posts({})", id));

    tokio::time::sleep(Duration::from_millis(150)).await;

    vec![Post { content: "Hello".into() }]
}
```

### 3. Generate reports

```rust
#[tokio::main]
async fn main() {
    // Reset inspector (clears previous data)
    Inspector::global().reset();

    // Run your async code
    let user = fetch_user(42).await;
    println!("Got user: {:?}", user);

    // Generate beautiful reports
    let reporter = Reporter::global();

    // Option 1: Full summary with task list
    reporter.print_summary();

    // Option 2: Timeline view
    reporter.print_timeline();

    // Option 3: Compact one-liner
    reporter.print_compact_summary();

    // Option 4: Generate text report
    let report = reporter.generate_report();
    std::fs::write("async-trace.txt", report).unwrap();
}
```

## 📊 Output Examples

### Task Summary
```
┌─────────────────────────────────────────────────────────────┐
│ async-inspect - Task Summary                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Total Tasks:       3                                        │
│ Active:            0 (Running: 0, Blocked: 0)              │
│ Completed:         3                                        │
│ Failed:            0                                        │
│ Total Events:     18                                        │
│ Duration:        0.31s                                      │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ Tasks                                                       │
├─────────────────────────────────────────────────────────────┤
│ #1 ✅ fetch_user(42)                                         │
│ #2 ✅ fetch_profile(42)                                      │
│ #3 ✅ fetch_posts(42)                                        │
└─────────────────────────────────────────────────────────────┘
```

### Timeline
```
┌─────────────────────────────────────────────────────────────┐
│ async-inspect - Timeline                                    │
├─────────────────────────────────────────────────────────────┤
│ [0.313s] #1: Spawned: fetch_user(42)                        │
│ [0.313s] #1: Inspection[start]                              │
│ [0.261s] #2: Spawned: fetch_profile(42)                     │
│ [0.156s] #2: Completed (0.10s)                              │
│ [0.156s] #1: Inspection[profile_loaded]                     │
│ [0.104s] #3: Spawned: fetch_posts(42)                       │
│ [0.052s] #3: Completed (0.15s)                              │
│ [0.000s] #1: Inspection[done]                               │
│ [0.000s] #1: Completed (0.31s)                              │
└─────────────────────────────────────────────────────────────┘
```

### Compact Summary
```
async-inspect: 3 tasks (0 active, 3 completed, 0 failed) | 18 events | 0.31s
```

## 🎨 API Reference

### Core Types

```rust
// Task tracking
TaskId              // Unique task identifier
TaskInfo            // Task metadata and state
TaskState           // Pending | Running | Blocked | Completed | Failed

// Events
Event               // Timestamped execution event
EventKind           // Type of event (spawn, poll, await, complete, etc.)
Timeline            // Collection of events

// Inspector
Inspector           // Main inspector singleton
InspectorStats      // Statistics about tracked tasks
Reporter            // Report generation and formatting
```

### Macros

```rust
// Mark an inspection point
inspect_point!("label");
inspect_point!("label", "additional message");

// Manual task tracking (if not using TaskGuard)
let task_id = inspect_task_start!("task_name");
// ... do work ...
inspect_task_complete!(task_id);
// or
inspect_task_failed!(task_id, "error message");
```

### RAII Guards

```rust
// Automatic task tracking
let _guard = TaskGuard::new("task_name");
// Task automatically marked as completed when guard drops

// Low-level guards (for advanced use)
let _poll_guard = PollGuard::new(task_id);
let _await_guard = AwaitGuard::new(task_id, "await_point_name");
```

### Inspector API

```rust
let inspector = Inspector::global();

// Control
inspector.enable();
inspector.disable();
inspector.is_enabled();
inspector.reset();
inspector.clear();

// Task management
let task_id = inspector.register_task("name".into());
inspector.update_task_state(task_id, TaskState::Running);
inspector.poll_started(task_id);
inspector.poll_ended(task_id, duration);
inspector.await_started(task_id, "await_point".into(), None);
inspector.await_ended(task_id, "await_point".into(), duration);
inspector.task_completed(task_id);
inspector.task_failed(task_id, Some("error".into()));
inspector.inspection_point(task_id, "label".into(), None);

// Queries
let task = inspector.get_task(task_id);
let all_tasks = inspector.get_all_tasks();
let events = inspector.get_events();
let task_events = inspector.get_task_events(task_id);
let stats = inspector.stats();
```

## 🔍 Debugging Scenarios

### Find Slow Operations

```rust
// Run your code
run_my_async_code().await;

// Check timeline for long-running tasks
let reporter = Reporter::global();
reporter.print_timeline();

// Look for large gaps in timestamps
```

### Track Task Relationships

```rust
async fn parent_task() {
    let _guard = TaskGuard::new("parent");

    spawn_child_task().await;
}

async fn spawn_child_task() {
    let _guard = TaskGuard::new("child");
    // Parent-child relationship tracked
}
```

### Monitor Task States

```rust
// Get real-time stats
let stats = Inspector::global().stats();
println!("Active tasks: {}", stats.running_tasks + stats.blocked_tasks);
println!("Completed: {}", stats.completed_tasks);
println!("Failed: {}", stats.failed_tasks);
```

## 🚧 Current Limitations

1. **Manual Instrumentation Required** - You must add `TaskGuard` to each function
2. **No Automatic Tokio Integration** - Spawned tasks aren't tracked automatically
3. **No Proc Macro** - Can't automatically instrument all `.await` points
4. **No Real-time TUI** - Output is generated after execution
5. **No Deadlock Detection** - Not yet implemented
6. **No Performance Profiling** - Just basic timing

## ✨ What's Coming Next

See [PROGRESS.md](PROGRESS.md) for the full development plan:

- **Phase 2**: Automatic Tokio runtime integration
- **Phase 3**: Proc macro for automatic instrumentation
- **Phase 4**: Enhanced visualization
- **Phase 5**: Deadlock detection
- **Phase 6**: Performance profiling
- **Phase 7**: Real-time TUI
- **Phase 8**: Production release

## 📝 Examples

See the `examples/` directory:

- `simple_test.rs` - Basic usage with sequential tasks
- `basic_inspection.rs` - More complex example (has spawning issues to fix)

Run with:
```bash
cargo run --example simple_test
```

## 🤝 Contributing

This is an early-stage project! Contributions welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

MIT - See LICENSE file

---

**Built with ❤️ for the Rust async community**

Making async Rust debugging as easy as synchronous code!
