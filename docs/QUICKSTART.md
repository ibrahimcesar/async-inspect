# async-inspect Quickstart Guide

Get productive with async-inspect in 5 minutes. These guides cover the most common debugging scenarios.

---

## Table of Contents

1. [Debug a Hanging Test](#1-debug-a-hanging-test)
2. [Find Lock Contention](#2-find-lock-contention)
3. [Monitor Production Tasks](#3-monitor-production-tasks)
4. [Export Traces for Analysis](#4-export-traces-for-analysis)

---

## 1. Debug a Hanging Test

**Problem:** Your async test hangs and you can't figure out where.

### Step 1: Add async-inspect to your project

```toml
# Cargo.toml
[dependencies]
async-inspect = "0.1"
```

### Step 2: Wrap your async code with tracking

```rust
use async_inspect::runtime::tokio::spawn_tracked;
use async_inspect::Inspector;

#[tokio::test]
async fn test_that_hangs() {
    // Create an inspector
    let inspector = Inspector::new();

    // Spawn your task with tracking
    let handle = spawn_tracked("test_user_flow", async {
        let user = fetch_user(123).await;
        let posts = fetch_posts(user.id).await;  // <-- Stuck here?
        let friends = fetch_friends(user.id).await;
        (user, posts, friends)
    });

    // Add a timeout with diagnostics
    tokio::select! {
        result = handle => {
            println!("Test completed: {:?}", result);
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            // Print where we're stuck
            println!("TIMEOUT! Task status:");
            for task in inspector.get_all_tasks() {
                println!("  {} - {:?} ({}ms)",
                    task.name,
                    task.state,
                    task.age().as_millis());
            }
        }
    }
}
```

### Step 3: Run with the TUI monitor

```bash
# Run with the interactive TUI
cargo run --example tui_monitor

# Or check task status programmatically
cargo test test_that_hangs -- --nocapture
```

### Expected Output

```
TIMEOUT! Task status:
  test_user_flow - Blocked(fetch_posts) (5023ms)
```

Now you know `fetch_posts` is where the test is stuck!

---

## 2. Find Lock Contention

**Problem:** Your async app is slow and you suspect lock contention.

### Step 1: Use tracked sync primitives

```rust
use async_inspect::sync::{Mutex, RwLock};  // Drop-in replacements
use async_inspect::Inspector;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let inspector = Inspector::global();

    // Use tracked Mutex instead of tokio::sync::Mutex
    let data = Arc::new(Mutex::new(Vec::new()));

    // Spawn multiple tasks that compete for the lock
    let mut handles = vec![];
    for i in 0..10 {
        let data = data.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..100 {
                let mut guard = data.lock().await;
                guard.push(i);
            }
        }));
    }

    // Wait for completion
    for h in handles {
        h.await.unwrap();
    }

    // Check contention metrics
    let metrics = data.metrics();
    println!("Lock Metrics:");
    println!("  Total acquisitions: {}", metrics.acquisitions);
    println!("  Contentions: {}", metrics.contentions);
    println!("  Avg wait time: {:?}", metrics.average_wait_time());
    println!("  Max wait time: {:?}", metrics.max_wait_time);
}
```

### Step 2: Check for deadlocks

```rust
use async_inspect::deadlock::DeadlockDetector;

// Check if any deadlocks exist
let deadlocks = inspector.deadlock_detector().detect();
if !deadlocks.is_empty() {
    println!("DEADLOCK DETECTED!");
    for cycle in &deadlocks {
        println!("  Cycle: {:?}", cycle);
    }
}
```

### Expected Output

```
Lock Metrics:
  Total acquisitions: 1000
  Contentions: 847
  Avg wait time: 2.3ms
  Max wait time: 45ms
```

High contention (847/1000 = 84.7%)! Consider:
- Reducing lock scope
- Using RwLock if reads are common
- Sharding the data

---

## 3. Monitor Production Tasks

**Problem:** You want to monitor async task health in production.

### Step 1: Configure for low overhead

```rust
use async_inspect::config::Config;
use async_inspect::Inspector;

fn setup_monitoring() -> Inspector {
    // Production-safe configuration
    let config = Config::production()
        .with_sampling_rate(100)      // Track 1 in 100 tasks
        .with_max_events(10_000)      // Limit memory usage
        .with_max_tasks(1_000);

    let inspector = Inspector::new();
    // Apply config (implementation varies)
    inspector
}
```

### Step 2: Export to Prometheus

```rust
use async_inspect::integrations::prometheus::PrometheusExporter;

async fn metrics_endpoint() -> String {
    let exporter = PrometheusExporter::new().unwrap();
    exporter.update();
    exporter.gather()
}

// In your web server:
// GET /metrics -> metrics_endpoint()
```

### Step 3: Set up alerts

```yaml
# prometheus/alerts.yml
groups:
  - name: async-inspect
    rules:
      - alert: HighBlockedTasks
        expr: async_inspect_blocked_tasks > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Many async tasks are blocked"

      - alert: TaskDeadlock
        expr: async_inspect_deadlocks_total > 0
        labels:
          severity: critical
        annotations:
          summary: "Deadlock detected in async tasks"
```

### Available Prometheus Metrics

| Metric | Description |
|--------|-------------|
| `async_inspect_tasks_total` | Total tasks created |
| `async_inspect_active_tasks` | Currently active tasks |
| `async_inspect_blocked_tasks` | Tasks waiting on I/O |
| `async_inspect_completed_tasks` | Successfully completed tasks |
| `async_inspect_failed_tasks` | Failed tasks |
| `async_inspect_task_duration_seconds` | Task execution time histogram |

---

## 4. Export Traces for Analysis

**Problem:** You want to analyze async execution in Chrome DevTools or Perfetto.

### Step 1: Record execution

```rust
use async_inspect::Inspector;
use async_inspect::export::ChromeTraceExporter;

#[tokio::main]
async fn main() {
    let inspector = Inspector::global();

    // Run your async workload
    run_workload().await;

    // Export to Chrome Trace format
    ChromeTraceExporter::export_to_file(
        inspector,
        "trace.json"
    ).expect("Failed to export trace");

    println!("Trace exported to trace.json");
    println!("Open in: chrome://tracing or https://ui.perfetto.dev");
}
```

### Step 2: Open in Chrome DevTools

1. Open Chrome/Chromium
2. Navigate to `chrome://tracing`
3. Click "Load" and select `trace.json`
4. Explore the timeline!

### Step 3: Alternative - Perfetto UI (recommended)

1. Go to [https://ui.perfetto.dev](https://ui.perfetto.dev)
2. Drag and drop `trace.json`
3. Use SQL queries for advanced analysis:

```sql
-- Find slowest tasks
SELECT name, dur/1e6 as duration_ms
FROM slice
ORDER BY dur DESC
LIMIT 10;

-- Find tasks that blocked longest
SELECT name, dur/1e6 as blocked_ms
FROM slice
WHERE name LIKE '%await%'
ORDER BY dur DESC
LIMIT 10;
```

### Other Export Formats

```rust
use async_inspect::export::{JsonExporter, CsvExporter, FlamegraphExporter};

// Full JSON export
JsonExporter::export_to_file(&inspector, "data.json")?;

// CSV for spreadsheets
CsvExporter::export_tasks_to_file(&inspector, "tasks.csv")?;
CsvExporter::export_events_to_file(&inspector, "events.csv")?;

// Flamegraph for performance analysis
FlamegraphExporter::export_to_file(&inspector, "flamegraph.txt")?;
// Then: cat flamegraph.txt | inferno-flamegraph > flamegraph.svg
```

---

## Troubleshooting

### "Tasks aren't being tracked"

Make sure you're using `spawn_tracked` instead of `tokio::spawn`:

```rust
// Wrong - not tracked
tokio::spawn(async { ... });

// Correct - tracked
spawn_tracked("my_task", async { ... });
```

### "Inspector is empty"

Use `Inspector::global()` for a shared inspector, or ensure you're using the same instance:

```rust
// Use the global instance
let inspector = Inspector::global();

// Or pass the same instance around
let inspector = Inspector::new();
let inspector_clone = inspector.clone();
```

### "High memory usage with many tasks"

Use compact storage for high-volume scenarios:

```rust
use async_inspect::compact::{TaskPool, MemoryStats};

let mut pool = TaskPool::new(10_000);  // Pre-allocate slots
let (slot, task) = pool.allocate("my_task").unwrap();

// Check memory usage
let stats = MemoryStats::calculate(&pool);
println!("Memory: {} bytes ({:.1}% savings)",
    stats.total_bytes,
    stats.estimated_savings_percent);
```

### "TUI not showing"

Ensure the `cli` feature is enabled:

```toml
[dependencies]
async-inspect = { version = "0.1", features = ["cli"] }
```

---

## Next Steps

- Read the [full documentation](https://docs.rs/async-inspect)
- Explore [examples](../examples/) for more use cases
- Check the [ROADMAP](../ROADMAP.md) for upcoming features
- Report issues on [GitHub](https://github.com/ibrahimcesar/async-inspect/issues)

---

*async-inspect - Because async shouldn't be a black box*
