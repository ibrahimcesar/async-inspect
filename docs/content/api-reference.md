---
sidebar_position: 10
title: API Reference
description: Complete API documentation with stability guarantees
---

# API Reference

This document describes the stable public API for `async-inspect`. Types and functions listed here follow semantic versioning guarantees.

## Stability Guarantees

Starting from **v1.0.0**, the public API will be stable with the following guarantees:

- **Stable**: Items marked stable will not have breaking changes in minor/patch releases
- **Unstable**: Items marked with `#[doc(hidden)]` or in modules prefixed with `_` are internal
- **Feature-gated**: Some APIs require specific feature flags and may evolve independently

### Current Status (v0.x)

During the 0.x series, the API may change between minor versions. However, we aim to minimize breaking changes and will document them in the CHANGELOG.

---

## Core Types

### Prelude

The prelude provides the most commonly used types:

```rust
use async_inspect::prelude::*;
```

Includes:
- `Inspector`, `InspectorStats` - Core inspection functionality
- `TaskId`, `TaskInfo`, `TaskState` - Task representation
- `TaskFilter`, `TaskSortBy`, `SortDirection` - Filtering and sorting
- `Event`, `EventKind` - Timeline events
- `Error`, `Result` - Error handling
- `Mutex`, `RwLock`, `Semaphore` (with `tokio` feature) - Tracked sync primitives

---

## Macros

### `#[async_inspect::trace]`

Instruments an async function for full tracing:

```rust
#[async_inspect::trace]
async fn my_function() {
    // Function body is automatically traced
}
```

### `#[async_inspect::inspect]`

Lighter-weight inspection without full tracing:

```rust
#[async_inspect::inspect]
async fn my_function() {
    // Basic inspection enabled
}
```

---

## Configuration (`async_inspect::config`)

### `Config`

Global configuration for async-inspect behavior.

```rust
use async_inspect::config::Config;

// Get global config
let config = Config::global();

// Preset modes
config.production_mode();  // Low overhead, sampling enabled
config.debug_mode();       // Full tracking, all features enabled

// Individual settings
config.set_sampling_rate(100);        // Track 1 in 100 tasks
config.set_max_events(10_000);        // Limit stored events
config.set_max_tasks(1_000);          // Limit tracked tasks
config.set_track_awaits(true);        // Track await points
```

### Adaptive Sampling (Production)

```rust
config.enable_adaptive_sampling();
config.set_adaptive_bounds(1, 1000);       // Min/max sampling rates
config.set_adaptive_target_overhead_ms(5.0); // Target overhead budget

// Check if task should be sampled
if config.adaptive_should_sample() {
    // Track this task
}

// Get statistics
let stats = config.adaptive_stats();
println!("Current rate: {}, within budget: {}",
    stats.current_rate, stats.is_within_budget());
```

---

## Task Tracking (`async_inspect::task`)

### `TaskId`

Unique identifier for a tracked task:

```rust
use async_inspect::task::TaskId;

let id = TaskId::new();           // Generate new ID
let id = TaskId::from_u64(42);    // From specific value
let raw: u64 = id.as_u64();       // Get raw value
```

### `TaskInfo`

Information about a tracked task:

```rust
use async_inspect::task::{TaskInfo, TaskState};

// Task properties
let name: &str = task.name();
let state: TaskState = task.state();
let created: Instant = task.created_at();
let duration: Duration = task.duration();
let poll_count: u64 = task.poll_count();
```

### `TaskState`

```rust
pub enum TaskState {
    Pending,    // Created but not yet polled
    Running,    // Currently being polled
    Blocked,    // Waiting on async operation
    Completed,  // Finished successfully
    Failed,     // Cancelled or panicked
}
```

### Filtering and Sorting

```rust
use async_inspect::task::{TaskFilter, TaskSortBy, SortDirection, sort_tasks};

// Create filter
let filter = TaskFilter::new()
    .with_state(TaskState::Running)
    .with_name_pattern("fetch_*")
    .with_min_duration(Duration::from_millis(100));

// Apply filter
let filtered: Vec<&TaskInfo> = filter.apply(&tasks);

// Sort tasks
sort_tasks(&mut tasks, TaskSortBy::Duration, SortDirection::Descending);
```

---

## Timeline (`async_inspect::timeline`)

### `Timeline`

Event timeline with two storage modes:

```rust
use async_inspect::timeline::{Timeline, TimelineConfig, StorageMode};

// Development mode (unbounded)
let timeline = Timeline::new();

// Production mode (fixed memory)
let timeline = Timeline::with_ring_buffer(10_000);

// Using config
let timeline = Timeline::with_config(TimelineConfig::production());
```

### Methods

```rust
// Add events
timeline.add_event(event);

// Query events
let all: Vec<Event> = timeline.events_vec();
let recent: Vec<Event> = timeline.recent_events(100);
let for_task: Vec<Event> = timeline.events_for_task_vec(task_id);

// Statistics
let count: usize = timeline.len();
let memory: usize = timeline.memory_usage();

// Ring buffer stats (production mode only)
if let Some(stats) = timeline.ring_stats() {
    println!("Overwrites: {}, Utilization: {:.1}%",
        stats.overwrites, stats.utilization * 100.0);
}
```

### `Event` and `EventKind`

```rust
use async_inspect::timeline::{Event, EventKind};

pub enum EventKind {
    TaskSpawned { name: String },
    TaskCompleted,
    TaskFailed { reason: String },
    PollStarted,
    PollEnded { duration: Duration },
    StateChange { from: TaskState, to: TaskState },
    AwaitEntered { name: String },
    AwaitExited { duration: Duration },
    ResourceAcquired { resource: String },
    ResourceReleased { resource: String },
    Custom { name: String, data: String },
}
```

---

## Ring Buffer (`async_inspect::ringbuf`)

Fixed-size buffer for production environments:

```rust
use async_inspect::ringbuf::{RingBuffer, RingBufferStats};

// Create buffer
let buffer: RingBuffer<Event> = RingBuffer::new(10_000);

// Operations
buffer.push(event);
let all: Vec<Event> = buffer.to_vec();
let recent: Vec<Event> = buffer.recent(100);

// Statistics
let stats: RingBufferStats = buffer.stats();
println!("Capacity: {}, Used: {}, Overwrites: {}",
    stats.capacity, stats.len, stats.overwrites);
println!("Utilization: {:.1}%, High churn: {}",
    stats.utilization * 100.0, stats.is_high_churn());
```

---

## Synchronization Primitives (`async_inspect::sync`)

*Requires `tokio` feature*

Drop-in replacements for `tokio::sync` with automatic tracking:

### `Mutex<T>`

```rust
use async_inspect::sync::Mutex;

let mutex = Mutex::new(data);
let guard = mutex.lock().await;

// Get contention metrics
let metrics = mutex.metrics();
println!("Acquisitions: {}, Contentions: {}, Avg wait: {:?}",
    metrics.acquisitions, metrics.contentions, metrics.avg_wait_time);
```

### `RwLock<T>`

```rust
use async_inspect::sync::RwLock;

let lock = RwLock::new(data);
let read_guard = lock.read().await;
let write_guard = lock.write().await;
```

### `Semaphore`

```rust
use async_inspect::sync::Semaphore;

let sem = Semaphore::new(10);
let permit = sem.acquire().await?;
```

---

## Channels (`async_inspect::channel`)

*Requires `tokio` feature*

Tracked channel primitives:

```rust
use async_inspect::channel::mpsc;

let (tx, rx) = mpsc::channel(100);
tx.send(value).await?;
let received = rx.recv().await;
```

---

## Inspector (`async_inspect::inspector`)

Core inspection functionality:

```rust
use async_inspect::Inspector;

let inspector = Inspector::new();

// Register tasks
let task_id = inspector.register_task("my_task");

// Update state
inspector.set_state(task_id, TaskState::Running);
inspector.record_poll(task_id, duration);

// Query
let task = inspector.get_task(task_id);
let all_tasks = inspector.tasks();
let stats = inspector.stats();
```

---

## Memory Optimization

### String Interning (`async_inspect::intern`)

Reduce memory for repeated strings:

```rust
use async_inspect::intern::{intern, resolve};

let interned = intern("my_task_name");
let original: &str = resolve(interned);
```

### Compact Storage (`async_inspect::compact`)

For high-performance scenarios:

```rust
use async_inspect::compact::{TaskPool, CompactTaskInfo};

let mut pool = TaskPool::new(10_000);
let (slot, task_id) = pool.allocate("task_name")?;
pool.free(slot);

let stats = pool.memory_stats();
println!("Memory: {} bytes, Utilization: {:.1}%",
    stats.total_bytes, stats.utilization * 100.0);
```

---

## Deadlock Detection (`async_inspect::deadlock`)

```rust
use async_inspect::deadlock::DeadlockDetector;

let detector = DeadlockDetector::new();

// Record lock acquisitions
detector.record_lock_attempt(task_id, lock_id);
detector.record_lock_acquired(task_id, lock_id);
detector.record_lock_released(task_id, lock_id);

// Check for deadlocks
if let Some(cycle) = detector.detect_deadlock() {
    println!("Deadlock detected: {:?}", cycle);
}
```

---

## Export (`async_inspect::export`)

Export data in various formats:

```rust
use async_inspect::export::{JsonExporter, CsvExporter};

// JSON
let json = JsonExporter::export_tasks(&tasks)?;
let json = JsonExporter::export_timeline(&timeline)?;

// CSV
let csv = CsvExporter::export_tasks(&tasks)?;
```

---

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `cli` | TUI interface | Yes |
| `tokio` | Tokio runtime support | Yes |
| `telemetry` | Anonymous usage analytics | Yes |
| `async-std-runtime` | async-std support | No |
| `smol-runtime` | smol support | No |
| `dashboard` | Web dashboard | No |
| `lsp` | LSP server | No |
| `prometheus-export` | Prometheus metrics | No |
| `opentelemetry-export` | OpenTelemetry traces | No |
| `flamegraph` | Flamegraph SVG generation | No |
| `full` | All features | No |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ASYNC_INSPECT_NO_TELEMETRY` | Disable telemetry (`1` to disable) |
| `DO_NOT_TRACK` | Disable telemetry (standard) |
| `ASYNC_INSPECT_LOG` | Set log level |

---

## Error Handling

```rust
use async_inspect::{Error, Result};

pub enum Error {
    Inspection(String),     // Inspection-related errors
    Runtime(String),        // Runtime errors
    Serialization(serde_json::Error),
    Io(std::io::Error),
}
```

---

## Migration Guide

### From 0.1.x to 0.2.x

*To be documented when 0.2.0 is released*

---

## See Also

- [CHANGELOG.md](CHANGELOG.md) - Version history and breaking changes
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [examples/](examples/) - Example code
