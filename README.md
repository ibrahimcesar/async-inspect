<div align="center">

# async-inspect 🔍

_X-ray vision for async Rust_

[![CI](https://github.com/ibrahimcesar/async-inspect/actions/workflows/ci.yml/badge.svg)](https://github.com/ibrahimcesar/async-inspect/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/async-inspect.svg)](https://crates.io/crates/async-inspect)
[![Documentation](https://docs.rs/async-inspect/badge.svg)](https://docs.rs/async-inspect)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![codecov](https://codecov.io/gh/ibrahimcesar/async-inspect/branch/main/graph/badge.svg)](https://codecov.io/gh/ibrahimcesar/async-inspect)

**async-inspect** is a debugging tool that visualizes and inspects async state machines in Rust. See exactly what your futures are doing, where they're stuck, and why.

[Documentation](https://ibrahimcesar.github.io/async-inspect/) | [Crates.io](https://crates.io/crates/async-inspect) | [API Docs](https://docs.rs/async-inspect)

</div>

## 😰 The Problem

Debugging async Rust is frustrating:

```rust
#[tokio::test]
async fn test_user_flow() {
    let user = fetch_user(123).await;      // Where is this stuck?
    let posts = fetch_posts(user.id).await; // Or here?
    let friends = fetch_friends(user.id).await; // Or here?
    
    // Test hangs... but WHERE? WHY? 😱
}
```

**What you see in a regular debugger:**

```bash
Thread blocked in:
  tokio::runtime::park
  std::sys::unix::thread::Thread::sleep
  ???
```

❌ Useless! You can't tell:

- Which `.await` is blocked
- What the future is waiting for
- How long it's been waiting
- What state the async state machine is in

**Common async debugging nightmares:**

- 🐌 Tests hang forever (where?)
- 🔄 Deadlocks with no stack trace
- ⏰ Timeouts that shouldn't happen
- 🎲 Flaky tests (race conditions)
- 📉 Performance issues (lock contention? slow I/O?)

**Current "solutions":**

```rust
// Solution 1: Add prints everywhere 😭
async fn fetch_user(id: u64) -> User {
    println!("Starting fetch_user");
    let result = http_get(url).await;
    println!("Finished fetch_user");
    result
}

// Solution 2: Use tokio-console (limited visibility)
// Solution 3: Give up and add timeouts everywhere 🤷
```

---

## 💡 The Solution

**async-inspect** gives you complete visibility into async execution:
```
$ async-inspect run ./my-app

┌─────────────────────────────────────────────────────────────┐
│ async-inspect - Task Inspector                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Task #42: fetch_user_data(user_id=12345)                    │
│ Status: BLOCKED (2.3s)                                      │
│ State: WaitingForPosts                                      │
│                                                             │
│ Progress: ▓▓▓▓▓░░░ 2/4 steps                                │
│                                                             │
│ ✅ fetch_user() - Completed (145ms)                         │
│ ⏳ fetch_posts() - IN PROGRESS (2.3s) ◄─── STUCK HERE       │
│    └─> http::get("api.example.com/posts/12345")             │
│        └─> TCP: ESTABLISHED, waiting for response           │
│        └─> Timeout in: 27.7s                                │
│ ⏸️  fetch_friends() - Not started                           │ 
│ ⏸️  build_response() - Not started                          │
│                                                             │
│ State Machine Polls: 156 (avg: 14.7ms between polls)        │
│                                                             │
│ Press 'd' for details | 't' for timeline | 'g' for graph    │
└─────────────────────────────────────────────────────────────┘
```

**Now you know EXACTLY:**
- ✅ Which step is stuck (`fetch_posts`)
- ✅ What it's waiting for (HTTP response)
- ✅ How long it's been waiting (2.3s)
- ✅ What will happen next (timeout in 27.7s)
- ✅ Complete execution history

---

## 🎯 Why async-inspect?

### Motivation

Async Rust is powerful but opaque. When you write:

```rust
async fn complex_operation() {
    let a = step_a().await;
    let b = step_b(a).await;
    let c = step_c(b).await;
}
```

The compiler transforms this into a **state machine**:

```rust
// Simplified - the real thing is more complex
enum ComplexOperationState {
    WaitingForStepA { /* ... */ },
    WaitingForStepB { a: ResultA, /* ... */ },
    WaitingForStepC { a: ResultA, b: ResultB, /* ... */ },
    Done,
}
```

**The problem:** This state machine is **invisible** to debuggers!

Traditional debuggers show you:
- ❌ Stack frames (useless - points to runtime internals)
- ❌ Variable values (many are "moved" or "uninitialized")
- ❌ Current line (incorrect - shows scheduler code)

**async-inspect** understands async state machines and shows you:
- ✅ Current state name and position
- ✅ All captured variables and their values
- ✅ Which `.await` you're blocked on
- ✅ Why you're blocked (I/O, lock, sleep, etc.)
- ✅ Complete execution timeline
- ✅ Dependencies between tasks

---

## 🆚 Comparison with Existing Tools

### tokio-console

[tokio-console](https://github.com/tokio-rs/console) is excellent but limited:
```bash
$ tokio-console
```

**What tokio-console shows:**
```
Task    Duration    Polls   State
#42     2.3s        156     Running
#43     0.1s        5       Idle
#44     5.2s        892     Running
```

**What it DOESN'T show:**
- ❌ Which `.await` is blocked
- ❌ Internal state machine state
- ❌ What the task is waiting for
- ❌ Variable values
- ❌ Deadlock detection
- ❌ Timeline visualization

### Comparison Table

| Feature | async-inspect | tokio-console | gdb/lldb | println! |
|---------|---------------|---------------|----------|----------|
| **See current `.await`** | ✅ | ❌ | ❌ | ⚠️ Manual |
| **State machine state** | ✅ | ❌ | ❌ | ❌ |
| **Variable inspection** | ✅ | ❌ | ⚠️ Limited | ❌ |
| **Waiting reason** | ✅ | ❌ | ❌ | ❌ |
| **Timeline view** | ✅ | ⚠️ Basic | ❌ | ❌ |
| **Deadlock detection** | ✅ | ❌ | ❌ | ❌ |
| **Dependency graph** | ✅ | ⚠️ Basic | ❌ | ❌ |
| **Runtime agnostic** | ✅ | ❌ Tokio only | ✅ | ✅ |
| **Zero code changes** | ✅ | ⚠️ Requires tracing | ✅ | ❌ |

**async-inspect is complementary to tokio-console:**
- tokio-console: High-level task monitoring
- async-inspect: Deep state machine inspection

Use both together for complete visibility!

### Runtime Support

**async-inspect** works with multiple async runtimes:

- ✅ **Tokio** - Full support with `tokio` feature
- ✅ **async-std** - Full support with `async-std-runtime` feature
- ✅ **smol** - Full support with `smol-runtime` feature

Example usage with different runtimes:

```rust
// Tokio
use async_inspect::runtime::tokio::{spawn_tracked, InspectExt};

#[tokio::main]
async fn main() {
    spawn_tracked("my_task", async {
        // Your code here
    }).await;

    let result = fetch_data()
        .inspect("fetch_data")
        .await;
}

// async-std
use async_inspect::runtime::async_std::{spawn_tracked, InspectExt};

fn main() {
    async_std::task::block_on(async {
        spawn_tracked("my_task", async {
            // Your code here
        }).await;
    });
}

// smol
use async_inspect::runtime::smol::{spawn_tracked, InspectExt};

fn main() {
    smol::block_on(async {
        spawn_tracked("my_task", async {
            // Your code here
        }).await;
    });
}
```

See the [examples/](https://github.com/ibrahimcesar/async-inspect/tree/main/examples) directory for complete working examples.

---

## ✨ Features (Planned)

### Core Features

- 🔍 **State Machine Inspection** - See current state and variables
- ⏱️ **Execution Timeline** - Visualize async execution over time
- 🎯 **Breakpoints** - Pause at specific states or `.await` points
- 🔗 **Dependency Tracking** - See which tasks are waiting on others
- 💀 **Deadlock Detection** - Automatically find circular dependencies
- 📊 **Performance Analysis** - Identify slow operations and contention
- 🎮 **Interactive Debugging** - Step through async state transitions
- 📸 **Snapshot & Replay** - Record execution and replay later

### Advanced Features

- 🌐 **Distributed Tracing** - Track async across services
- 🔥 **Flamegraphs** - Visualize where time is spent
- 🎛️ **Live Inspection** - Attach to running processes
- 📝 **Export & Share** - Save traces for collaboration
- 🤖 **CI Integration** - Detect hangs in test suites
- 🎨 **Custom Views** - Plugin system for specialized visualization

---

## 🚧 Status

**Work in Progress** - Early development

Current version: `0.1.0-alpha`

---

## 🚀 Quick Start (Planned API)

### Installation
```bash
# Not yet published
cargo install async-inspect

# Or build from source
git clone https://github.com/yourusername/async-inspect
cd async-inspect
cargo install --path .
```

### Basic Usage
```bash
# Run your app with inspection enabled
async-inspect run ./my-app

# Attach to running process
async-inspect attach --pid 12345

# Run tests with inspection
async-inspect test

# Start web dashboard
async-inspect serve --port 8080
```

### In Code (Optional Instrumentation)
```rust
// Add to Cargo.toml
[dependencies]
async-inspect = "0.1"

// Instrument specific functions
#[async_inspect::trace]
async fn fetch_user(id: u64) -> User {
    // Automatically instrumented
    let profile = fetch_profile(id).await;
    let posts = fetch_posts(id).await;
    User { profile, posts }
}

// Or use manual inspection points
use async_inspect::prelude::*;

async fn complex_operation() {
    inspect_point!("starting");
    
    let data = fetch_data().await;
    
    inspect_point!("data_fetched", data.len());
    
    process(data).await
}
```

---

## 📖 Use Cases

### 1. Find Where Test is Stuck
```rust
#[tokio::test]
async fn test_timeout() {
    // This test hangs... but where?
    let result = timeout(
        Duration::from_secs(30),
        long_operation()
    ).await;
}
```

**With async-inspect:**
```bash
$ async-inspect test

Found test stuck at:
  test_timeout
    └─> long_operation()
        └─> fetch_data().await  ◄─── BLOCKED (5m 23s)
            └─> Waiting for: HTTP response
            └─> URL: https://slow-api.example.com/data
            └─> Timeout: None (will wait forever!)
            
Suggestion: Add timeout to HTTP client
```

### 2. Debug Deadlock
```rust
async fn deadlock_example() {
    let mutex_a = Arc::new(Mutex::new(0));
    let mutex_b = Arc::new(Mutex::new(0));
    
    // Task 1: locks A then B
    tokio::spawn(async move {
        let _a = mutex_a.lock().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        let _b = mutex_b.lock().await; // DEADLOCK!
    });
    
    // Task 2: locks B then A
    tokio::spawn(async move {
        let _b = mutex_b.lock().await;
        tokio::time::sleep(Duration::from_millis(10)).await;
        let _a = mutex_a.lock().await; // DEADLOCK!
    });
}
```

**With async-inspect:**
```
💀 DEADLOCK DETECTED!

Task #42: waiting for Mutex<i32> @ 0x7f8a9c0
  └─> Held by: Task #89
  
Task #89: waiting for Mutex<i32> @ 0x7f8a9d0
  └─> Held by: Task #42

Circular dependency:
  Task #42 → Mutex A → Task #89 → Mutex B → Task #42

Suggestion:
  • Acquire locks in consistent order (A before B)
  • Use try_lock() with timeout
  • Consider lock-free alternatives
```

### 3. Performance Investigation
```bash
$ async-inspect profile ./my-app

Performance Report:
  
Slowest Operations:
  1. fetch_posts() - avg 2.3s (called 450x)
     └─> 98% time in: HTTP request
     └─> Suggestion: Add caching or batch requests
  
  2. acquire_lock() - avg 340ms (called 1200x)
     └─> Lock contention: 50 tasks waiting
     └─> Suggestion: Reduce lock scope or use RwLock

Hot Paths:
  1. process_request → fetch_user → fetch_posts (89% of requests)
  2. handle_webhook → validate → store (11% of requests)
```

### 4. CI/CD Integration
```yaml
# .github/workflows/test.yml
- name: Run tests with async inspection
  run: async-inspect test --timeout 30s --fail-on-hang
  
- name: Upload trace on failure
  if: failure()
  uses: actions/upload-artifact@v3
  with:
    name: async-trace
    path: async-inspect-trace.json
```

---

## 🛠️ How It Works

### Compiler Instrumentation
```rust
// Your code
async fn fetch_user(id: u64) -> User {
    let profile = fetch_profile(id).await;
    let posts = fetch_posts(id).await;
    User { profile, posts }
}

// With instrumentation (conceptual)
async fn fetch_user(id: u64) -> User {
    __async_inspect_enter("fetch_user", id);
    
    __async_inspect_await_start("fetch_profile");
    let profile = fetch_profile(id).await;
    __async_inspect_await_end("fetch_profile");
    
    __async_inspect_await_start("fetch_posts");
    let posts = fetch_posts(id).await;
    __async_inspect_await_end("fetch_posts");
    
    let result = User { profile, posts };
    __async_inspect_exit("fetch_user", &result);
    result
}
```

### Runtime Integration

- **Tokio:** Hooks into task spawning and polling
- **async-std:** Custom executor wrapper
- **smol:** Runtime instrumentation
- **Generic:** Works with any runtime via proc macros

### Zero Overhead When Disabled
```toml
# Production build - no overhead
[profile.release]
debug = false

# Debug build - full instrumentation
[profile.dev]
debug = true
```

---

## 🌐 Ecosystem Integration

async-inspect works seamlessly with your existing Rust async ecosystem tools:

### Prometheus Metrics

Export metrics for monitoring dashboards:

```rust
use async_inspect::integrations::prometheus::PrometheusExporter;

let exporter = PrometheusExporter::new()?;
exporter.update();

// In your /metrics endpoint:
let metrics = exporter.gather();
```

**Available metrics:**
- `async_inspect_tasks_total` - Total tasks created
- `async_inspect_active_tasks` - Currently active tasks
- `async_inspect_blocked_tasks` - Tasks waiting on I/O
- `async_inspect_task_duration_seconds` - Task execution times
- `async_inspect_tasks_failed_total` - Failed task count

### OpenTelemetry Export

Send traces to Jaeger, Zipkin, or any OTLP backend:

```rust
use async_inspect::integrations::opentelemetry::OtelExporter;

let exporter = OtelExporter::new("my-service");
exporter.export_tasks();
```

### Tracing Integration

Automatic capture via `tracing-subscriber`:

```rust
use tracing_subscriber::prelude::*;
use async_inspect::integrations::tracing_layer::AsyncInspectLayer;

tracing_subscriber::registry()
    .with(AsyncInspectLayer::new())
    .init();
```

### Tokio Console Compatibility

Use alongside tokio-console for complementary insights:

```bash
# Terminal 1: Run with tokio-console
RUSTFLAGS="--cfg tokio_unstable" cargo run

# Terminal 2: Monitor with tokio-console
tokio-console

# async-inspect exports provide historical analysis
cargo run --example ecosystem_integration
```

### Grafana Dashboards

Import async-inspect metrics into Grafana:

1. Configure Prometheus scraping
2. Import dashboard template (coming soon)
3. Monitor key metrics:
   - Task creation rate
   - Active/blocked task ratio
   - Task duration percentiles
   - Error rates

**Feature Flags:**

```toml
[dependencies]
async-inspect = { version = "0.0.1", features = [
    "prometheus-export",     # Prometheus metrics
    "opentelemetry-export",  # OTLP traces
    "tracing-sub",           # Tracing integration
] }
```

---

## 📤 Export Formats

async-inspect supports multiple industry-standard export formats for visualization and analysis:

### JSON Export

Export complete task and event data as structured JSON:

```rust
use async_inspect::export::JsonExporter;

// Export to file
JsonExporter::export_to_file(&inspector, "data.json")?;

// Or get as string
let json = JsonExporter::export_to_string(&inspector)?;
```

**Use with:** `jq`, Python pandas, JavaScript tools, data pipelines

### CSV Export

Export tasks and events in spreadsheet-compatible format:

```rust
use async_inspect::export::CsvExporter;

// Export tasks (id, name, duration, poll_count, etc.)
CsvExporter::export_tasks_to_file(&inspector, "tasks.csv")?;

// Export events (event_id, task_id, timestamp, kind, details)
CsvExporter::export_events_to_file(&inspector, "events.csv")?;
```

**Use with:** Excel, Google Sheets, pandas, data analysis

### Chrome Trace Event Format

Export for visualization in chrome://tracing or Perfetto UI:

```rust
use async_inspect::export::ChromeTraceExporter;

ChromeTraceExporter::export_to_file(&inspector, "trace.json")?;
```

**How to visualize:**

1. **Chrome DevTools** (built-in):
   - Open Chrome/Chromium
   - Navigate to `chrome://tracing`
   - Click "Load" and select `trace.json`
   - Explore the interactive timeline!

2. **Perfetto UI** (recommended):
   - Go to [https://ui.perfetto.dev/](https://ui.perfetto.dev/)
   - Click "Open trace file"
   - Select `trace.json`
   - Get advanced analysis features:
     - Thread-level view
     - SQL-based queries
     - Statistical summaries
     - Custom tracks

**What you see:**
- Task spawning and completion as events
- Poll operations with precise durations
- Await points showing blocking time
- Complete async execution timeline
- Task relationships and dependencies

### Flamegraph Export

Generate flamegraphs for performance analysis:

```rust
use async_inspect::export::{FlamegraphExporter, FlamegraphBuilder};

// Basic export (folded stack format)
FlamegraphExporter::export_to_file(&inspector, "flamegraph.txt")?;

// Customized export
FlamegraphBuilder::new()
    .include_polls(false)      // Exclude poll events
    .include_awaits(true)       // Include await points
    .min_duration_ms(10)        // Filter < 10ms operations
    .export_to_file(&inspector, "flamegraph_filtered.txt")?;

// Generate SVG directly (requires 'flamegraph' feature)
#[cfg(feature = "flamegraph")]
FlamegraphExporter::generate_svg(&inspector, "flamegraph.svg")?;
```

**How to visualize:**

1. **Speedscope** (easiest, online):
   - Go to [https://www.speedscope.app/](https://www.speedscope.app/)
   - Drop `flamegraph.txt` onto the page
   - Explore interactive flamegraph

2. **inferno** (local SVG generation):
   ```bash
   cargo install inferno
   cat flamegraph.txt | inferno-flamegraph > output.svg
   open output.svg
   ```

3. **flamegraph.pl** (classic):
   ```bash
   git clone https://github.com/brendangregg/FlameGraph
   ./FlameGraph/flamegraph.pl flamegraph.txt > output.svg
   ```

**What you see:**
- Call stacks showing task hierarchies
- Time spent in each async operation
- Hotspots and bottlenecks
- Parent-child task relationships

### Comprehensive Example

See [examples/export_formats.rs](examples/export_formats.rs) for a complete example:

```bash
cargo run --example export_formats
```

This demonstrates:
- All export formats in one workflow
- Realistic async operations
- Multiple concurrent tasks
- Export to JSON, CSV, Chrome Trace, and Flamegraph
- Usage instructions for each format

**Output files:**
```
async_inspect_exports/
├── data.json                    # Complete JSON export
├── tasks.csv                    # Task metrics
├── events.csv                   # Event timeline
├── trace.json                   # Chrome Trace Event Format
├── flamegraph.txt               # Basic flamegraph
└── flamegraph_filtered.txt      # Filtered flamegraph
```

---

## 🗺️ Roadmap

### Phase 1: Core Inspector (Current)
- [ ] Basic state machine inspection
- [ ] Task listing and status
- [ ] Simple TUI interface
- [ ] Tokio runtime integration

### Phase 2: Advanced Debugging
- [ ] Variable inspection
- [ ] Breakpoints on states
- [ ] Step-by-step execution
- [ ] Timeline visualization

### Phase 3: Analysis Tools
- [ ] Deadlock detection
- [ ] Performance profiling
- [ ] Lock contention analysis
- [ ] Flamegraphs

### Phase 4: Production Ready
- [ ] Web dashboard
- [ ] Live process attachment
- [ ] Distributed tracing
- [ ] CI/CD integration
- [ ] Plugin system

### Phase 5: Ecosystem
- [ ] async-std support
- [ ] smol support
- [ ] IDE integration (VS Code, IntelliJ)
- [ ] Cloud deployment monitoring

---

## 🎨 Interface Preview (Planned)

### TUI (Terminal)
```
┌─ async-inspect ─────────────────────────────────────────┐
│ [Tasks] [Timeline] [Graph] [Profile]          [?] Help  │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ Active Tasks: 23                 CPU: ████░░ 45%       │
│ Blocked: 8                       Mem: ██░░░░ 20%       │
│ Running: 15                                             │
│                                                          │
│ Task    State            Duration    Details            │
│ ─────────────────────────────────────────────────────── │
│ #42  ⏳ WaitingPosts    2.3s      http::get()          │
│ #43  ✅ Done            0.1s      Completed             │
│ #44  💀 Deadlock        5.2s      Mutex wait            │
│ #45  🏃 Running         0.03s     Computing             │
│                                                          │
│ [←→] Navigate  [Enter] Details  [g] Graph  [q] Quit    │
└──────────────────────────────────────────────────────────┘
```

### Web Dashboard
```
http://localhost:8080

┌────────────────────────────────────────────────┐
│  async-inspect                      [Settings] │
├────────────────────────────────────────────────┤
│                                                │
│  📊 Overview           🕒 Last updated: 2s ago │
│                                                │
│  ● 23 Tasks Active     ▁▃▅▇█▇▅▃▁ Activity     │
│  ⏸️  8 Blocked                                 │
│  💀 1 Deadlock         [View Details →]       │
│                                                │
│  📈 Performance                                │
│  ├─ Avg Response: 145ms                       │
│  ├─ 99th percentile: 2.3s                     │
│  └─ Slowest: fetch_posts() - 5.2s            │
│                                                │
│  [View Timeline] [Export Trace] [Filter...]   │
└────────────────────────────────────────────────┘
```

---

## 🤝 Contributing

Contributions welcome! This is a challenging project that needs expertise in:

- 🦀 Rust compiler internals
- 🔧 Async runtime implementation
- 🎨 UI/UX design
- 📊 Data visualization
- 🐛 Debugger implementation

**Priority areas:**
- [ ] State machine introspection
- [ ] Runtime hooks (Tokio, async-std)
- [ ] TUI implementation
- [ ] Deadlock detection algorithms
- [ ] Documentation and examples

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

---

## 📊 Telemetry

This project uses **[telemetry-kit](https://telemetry-kit.dev/)** to collect anonymous usage analytics. This helps us understand how async-inspect is used in the real world, enabling data-driven decisions instead of relying solely on GitHub issues.

**What we collect:**
- Commands executed (e.g., `monitor`, `export`, `stats`)
- Feature usage patterns
- Command execution times
- Opt-out rates (anonymous)

**What we DON'T collect:**
- No personal information
- No code or file contents
- No IP addresses or location data
- No identifying information

### Why Telemetry Matters

Open source projects often make decisions based on a vocal minority. Telemetry gives us visibility into:
- Which features are actually used vs. which are requested
- Real-world performance characteristics
- Usage patterns across different environments
- Where to focus development effort

We will publish a **public dashboard** showing aggregated, anonymous usage data at: *Coming soon*

### Disabling Telemetry

You can disable telemetry in several ways:

**1. Environment variable (recommended):**
```bash
export ASYNC_INSPECT_NO_TELEMETRY=1
```

**2. Standard DO_NOT_TRACK:**
```bash
export DO_NOT_TRACK=1
```

**3. Compile-time (excludes telemetry code entirely):**
```toml
[dependencies]
async-inspect = { version = "0.1", default-features = false, features = ["cli", "tokio"] }
```

Even when telemetry is disabled, we send a single anonymous opt-out signal. This helps us understand the opt-out rate without collecting any identifying information.

**Learn more:**
- [telemetry-kit.dev](https://telemetry-kit.dev/) - Project homepage
- [docs.telemetry-kit.dev](https://docs.telemetry-kit.dev/) - Documentation

---

## 🔒 Security

async-inspect is designed to be used in development and CI/CD environments for analyzing async code. We take security seriously:

### Supply Chain Security

- **SLSA Level 3 Provenance**: All release binaries include [SLSA](https://slsa.dev/) provenance attestations for verifiable builds
- **Dependency Scanning**: Automated dependency review on all pull requests
- **License Compliance**: Only permissive licenses (MIT, Apache-2.0, BSD) - GPL/AGPL excluded
- **Security Audits**: Continuous monitoring via `cargo-audit` and `cargo-deny`

### Build Verification

You can verify the provenance of any release binary:

```bash
# Install GitHub CLI attestation verification
gh attestation verify async-inspect-linux-x86_64.tar.gz \
  --owner ibrahimcesar
```

### Reporting Security Issues

If you discover a security vulnerability, please email security@ibrahimcesar.com instead of using the issue tracker.

---

## 📝 License

MIT OR Apache-2.0

---

## 🙏 Acknowledgments

Inspired by:
- [tokio-console](https://github.com/tokio-rs/console) - Task monitoring for Tokio
- [async-backtrace](https://github.com/tokio-rs/async-backtrace) - Async stack traces
- [tracing](https://github.com/tokio-rs/tracing) - Instrumentation framework
- Chrome DevTools - JavaScript async debugging
- Go's runtime tracer - Goroutine visualization
- [rr](https://rr-project.org/) - Time-travel debugging

---

**async-inspect** - *Because async shouldn't be a black box* 🔍

*Status: 🚧 Pre-alpha - Architecture design phase*

**Star** ⭐ this repo to follow development!

## 💬 Discussion

Have ideas or feedback? Open an issue or discussion!

**Key questions we're exploring:**
- How to minimize runtime overhead?
- Best UI for visualizing state machines?
- How to support multiple runtimes?
- What features would help you most?
