# Quick Start

Get async-inspect running in 5 minutes.

## Step 1: Install

```bash
cargo install async-inspect
```

## Step 2: Add to Your Project

```toml
[dependencies]
async-inspect = "0.1.0"
```

## Step 3: Instrument Your Code

Add the `#[async_inspect::trace]` attribute to async functions you want to monitor:

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn fetch_user(id: u64) -> User {
    let profile = fetch_profile(id).await;
    let posts = fetch_posts(id).await;
    User { profile, posts }
}

#[async_inspect::trace]
async fn fetch_profile(id: u64) -> Profile {
    // Your implementation
}

#[async_inspect::trace]
async fn fetch_posts(id: u64) -> Vec<Post> {
    // Your implementation
}
```

## Step 4: Run Your Code

```rust
#[tokio::main]
async fn main() {
    // Your async code
    let user = fetch_user(42).await;

    // Print summary
    Reporter::global().print_summary();
}
```

## Step 5: See the Results!

```
╔════════════════════════════════════════════════════════════╗
║  async-inspect Summary                                     ║
╚════════════════════════════════════════════════════════════╝

📊 Statistics:
  Total Tasks:     12
  Running:          0
  Blocked:          0
  Completed:       12
  Failed:           0

⏱️  Performance:
  Avg Duration:   145ms
  Max Duration:   567ms
  Total Events:    48

Top Tasks by Duration:
  #1  fetch_posts      567ms  (12 polls)
  #2  fetch_profile    234ms  (8 polls)
  #3  fetch_user       145ms  (4 polls)
```

## CLI Usage

### Monitor in Real-Time

```bash
async-inspect monitor
```

This launches a TUI (Terminal UI) showing live task updates:

```
┌─ async-inspect ─────────────────────────────────────────┐
│ [Tasks] [Timeline] [Graph] [Profile]          [?] Help  │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ Active Tasks: 3                  CPU: ████░░ 45%       │
│ Blocked: 1                       Mem: ██░░░░ 20%       │
│                                                          │
│ Task    State            Duration    Details            │
│ ─────────────────────────────────────────────────────── │
│ #42  ⏳ WaitingPosts    2.3s      http::get()          │
│ #43  ✅ Done            0.1s      Completed             │
│ #44  🏃 Running         0.03s     Computing             │
└──────────────────────────────────────────────────────────┘
```

### Export Data

Export to JSON:
```bash
async-inspect export --format json --output trace.json
```

Export to CSV:
```bash
async-inspect export --format csv --output tasks.csv
```

### Show Statistics

```bash
async-inspect stats
```

### Production Mode

Enable production optimizations:

```bash
async-inspect config production
```

This sets:
- Sampling rate to 0.01 (1%)
- Max tasks to 1000
- Minimal overhead

## Examples

### Basic Inspection

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn main_task() {
    println!("Starting...");
    task_a().await;
    task_b().await;
    println!("Done!");
}

#[tokio::main]
async fn main() {
    main_task().await;

    // View results
    Reporter::global().print_summary();
}
```

### With Tokio Spawn

```rust
use async_inspect::runtime::tokio::spawn_tracked;

#[tokio::main]
async fn main() {
    let handle = spawn_tracked("worker", async {
        // Your work here
        heavy_task().await
    });

    handle.await.unwrap();
    Reporter::global().print_summary();
}
```

### Performance Analysis

```rust
#[tokio::main]
async fn main() {
    run_application().await;

    // Export for analysis
    let inspector = Inspector::global();
    JsonExporter::export_to_file(&inspector, "profile.json")?;

    println!("Profile saved to profile.json");
    println!("Analyze with: async-inspect analyze profile.json");
}
```

## VS Code Integration

1. Install the VS Code extension
2. Open your Rust project
3. Click the async-inspect icon in sidebar
4. Click "Start Monitoring"
5. Run your code
6. See tasks appear in real-time!

Features:
- Task tree view
- Inline performance stats
- Interactive timeline
- Deadlock detection

## Next Steps

- [CLI Reference](./cli-usage) - All commands and options
- [Examples](./examples) - More detailed examples
- [Production Guide](./production) - Deploy with confidence
- [Troubleshooting](./troubleshooting) - Common issues and solutions

## Common Patterns

### Test Debugging

```rust
#[tokio::test]
#[async_inspect::trace]
async fn test_user_flow() {
    let user = create_user().await;
    assert!(user.is_valid());

    // If test hangs, async-inspect shows where!
}
```

### Deadlock Detection

```rust
#[tokio::main]
async fn main() {
    run_app().await;

    // Check for deadlocks
    let graph = TaskGraph::from_inspector(&Inspector::global());
    let deadlocks = graph.detect_potential_deadlocks();

    if !deadlocks.is_empty() {
        eprintln!("⚠️  Found {} potential deadlocks!", deadlocks.len());
    }
}
```

### Resource Contention

```rust
#[tokio::main]
async fn main() {
    run_app().await;

    // Find shared resource contention
    let graph = TaskGraph::from_inspector(&Inspector::global());
    let shared = graph.find_tasks_sharing_resource("database_pool");

    println!("Tasks using database_pool: {}", shared.len());
}
```

---

[← Installation](./installation) | [CLI Usage →](./cli-usage)
