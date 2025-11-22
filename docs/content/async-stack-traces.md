# Solving the Async Stack Trace Problem

One of the top pain points in the [Rust 2025 Survey](https://blog.rust-lang.org/2025/02/rust-survey-2025.html) was **poor async stack traces**. async-inspect directly addresses this problem.

## The Problem

When an async function panics, traditional stack traces are nearly useless:

```
thread 'tokio-runtime-worker' panicked at 'database connection failed'
stack backtrace:
   0: std::panicking::begin_panic
   1: <core::pin::Pin<P> as core::future::future::Future>::poll
   2: tokio::runtime::task::core::Core<T,S>::poll
   3: tokio::runtime::task::harness::Harness<T,S>::poll
             at ~/.cargo/registry/src/tokio-1.0/src/runtime/task/harness.rs:150
   4: tokio::runtime::blocking::pool::Inner::run
   5: std::sys_common::backtrace::__rust_begin_short_backtrace
```

### What's Wrong?

❌ **No task context**: Which async task failed?
❌ **No async call chain**: What function called what?
❌ **No await point**: Where was the task blocked?
❌ **No state information**: What was the task doing?
❌ **Runtime internals only**: Just tokio/async-std internals

### Why Does This Happen?

Async functions are compiled into **state machines**:

```rust
// You write:
async fn fetch_user(id: u64) -> User {
    let profile = db.get_profile(id).await;
    let posts = db.get_posts(id).await;
    User { profile, posts }
}

// Compiler generates:
enum FetchUserState {
    Start { id: u64 },
    WaitProfile { id: u64, future: ProfileFuture },
    WaitPosts { profile: Profile, future: PostsFuture },
    Done,
}
```

When it panics, you only see the **poll machinery**, not your actual async code.

## The Solution: async-inspect

async-inspect captures **async-specific context** that normal stack traces can't provide.

### 1. Full Async Call Chain

**Traditional stack trace**:
```
tokio::runtime::task::harness::Harness<T,S>::poll
  at src/runtime/task/harness.rs:150
```

**async-inspect**:
```bash
$ async-inspect analyze --show-failures

Task #42: handle_request [PANICKED]
  ├─ Location: src/api/handlers.rs:23
  ├─ Duration: 5.2s before panic
  │
  └─ Async Call Chain:
     1. main::spawn_handler         (src/main.rs:45)
     2. handle_request(req)          (src/api/handlers.rs:23)
     3. ├─ authenticate_user(token)  (src/auth.rs:67)  [50ms] ✅
     4. ├─ fetch_user_data(id: 123) (src/users.rs:34)
     5. │  └─ db_query(sql)          (src/db.rs:89)    [5.1s] ❌ PANICKED
     6. └─ ❌ PANIC: "connection timeout"
```

### 2. Current Await Point

See **exactly where** the task was stuck:

```bash
Task #42 State:
  Status: PANICKED
  Blocked At: db_query().await
  Source: src/db.rs:89

  Code Context:
    87:     .bind(user_id)
    88:     .fetch_one(&pool)
    89: >>> .await?;  ← STUCK HERE FOR 5.2s
    90:
    91:     Ok(user)
```

### 3. Task Timeline

Understand what led to the panic:

```bash
$ async-inspect timeline --task 42

Task #42 Timeline:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  0ms   │ ● Task spawned
        │   handle_request(req: Request)
        │
  5ms   │ ● Entered: authenticate_user
 10ms   │ ○ Poll::Pending (awaiting auth)
 45ms   │ ● Poll::Ready(token)
 50ms   │ ✓ authenticate_user completed
        │
 55ms   │ ● Entered: fetch_user_data
 60ms   │   └─ db_query started
 65ms   │     ○ Poll::Pending (awaiting connection)
100ms   │     ○ Poll::Pending (waiting...)
200ms   │     ○ Poll::Pending (waiting...)
500ms   │     ○ Poll::Pending (still waiting...)
        │     ... [polled 847 times]
5200ms  │ ❌ TIMEOUT → PANIC
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️  Warning: Task polled 847 times without progress
    Possible cause: busy-wait loop or resource starvation
```

### 4. Related Tasks Analysis

Find patterns across failures:

```bash
$ async-inspect analyze --correlate

🔍 Failure Analysis

Found 10 related failures (last 5 minutes):

Pattern: Database connection timeout
  Tasks: #38, #39, #40, #41, #42, #43, #44, #45, #46, #47
  All blocked at: db_query().await
  Common cause: Connection pool exhausted

Connection Pool Status:
  ┌──────────────────────────┐
  │ Active:    10/10  [FULL] │ ⚠️
  │ Idle:       0/10         │
  │ Waiting:   37 tasks      │ ← Tasks waiting for connections
  └──────────────────────────┘

Diagnosis: Connection pool saturation
  - All 10 connections in use
  - 37 tasks waiting for available connection
  - Average wait time: 5.2s → timeout

Recommendations:
  1. Increase max_connections in database config
  2. Add connection timeout (currently unlimited)
  3. Implement connection retry with backoff
  4. Review slow queries holding connections
```

## Real-World Example

### Scenario: Production API Panic

Your production API starts panicking with this error:

```
thread 'tokio-runtime-worker' panicked at 'database error: connection timeout'
note: run with `RUST_BACKTRACE=1` for a backtrace
```

### Traditional Debugging Process

1. ❌ Stack trace shows only runtime internals
2. ❌ Add logging to every function manually
3. ❌ Reproduce locally (can't replicate production load)
4. ❌ Deploy, wait for it to happen again
5. ❌ Check logs, still not enough context
6. 😫 Hours/days of debugging

### With async-inspect

1. ✅ Check dashboard immediately:

```bash
$ async-inspect monitor

Active Tasks: 47
Failed Tasks (last 5m): 12
Deadlocks: 0
⚠️  High failure rate detected!

Failed Tasks:
  Task #42: handle_request [PANICKED] 5.2s
  Task #43: handle_request [PANICKED] 5.3s
  Task #44: handle_request [PANICKED] 5.1s
  ... [9 more]

Press 'd' for detailed analysis
```

2. ✅ See the pattern:

```bash
$ async-inspect analyze --failures

Common Failure Pattern:
  Location: src/db.rs:89 (db_query().await)
  Cause: Connection timeout after 5s
  Affected: 12 tasks

Root Cause Analysis:
  ┌─────────────────────────────────────┐
  │ Database connection pool exhausted  │
  │ 10/10 connections active            │
  │ 35+ tasks waiting                   │
  └─────────────────────────────────────┘
```

3. ✅ Fix immediately:

```rust
// Before (no timeout, no limit)
let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(db_url).await?;

// After (with timeout and more connections)
let pool = PgPoolOptions::new()
    .max_connections(50)           // ← Increase pool
    .acquire_timeout(Duration::from_secs(2))  // ← Add timeout
    .connect(db_url).await?;
```

4. ✅ Verify fix:

```bash
$ async-inspect monitor

Active Tasks: 52
Failed Tasks (last 5m): 0  ✓
Average response time: 45ms

Connection Pool:
  Active:   12/50
  Idle:     38/50  ✓ Healthy
  Waiting:  0
```

**Total time: 5 minutes** instead of hours/days.

## How to Use async-inspect for Stack Traces

### Setup

1. **Add instrumentation**:

```rust
use async_inspect::Inspector;

#[tokio::main]
async fn main() {
    // Initialize inspector
    let inspector = Inspector::new(Default::default());

    // Your app code
    run_server().await;
}

#[async_inspect::trace]  // ← Add to async functions
async fn handle_request(req: Request) -> Response {
    let user = authenticate(req).await?;
    let data = fetch_data(user.id).await?;
    render(data)
}
```

2. **Run with monitoring**:

```bash
# Terminal 1: Run your app
cargo run

# Terminal 2: Monitor tasks
async-inspect monitor
```

### When Something Panics

**Immediate triage**:

```bash
# See what failed
async-inspect analyze --show-failures

# Get detailed trace
async-inspect trace --task <id>

# Export for investigation
async-inspect export --json panic_trace.json
```

### Development Workflow

```bash
# During development
cargo run  # Inspector automatically enabled in debug mode

# In another terminal
async-inspect tui  # Live dashboard
```

### Production Deployment

```rust
// Low-overhead production config
let inspector = Inspector::new(Config {
    sampling_rate: 0.01,  // Only track 1% for low overhead
    capture_backtraces: false,
    mode: Mode::Production,
    ..Default::default()
});

// Export failures automatically
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;

        let failures = inspector.failed_tasks();
        if !failures.is_empty() {
            // Export to logging/monitoring system
            log::error!("Task failures: {:#?}", failures);
        }
    }
});
```

## Comparison with Other Solutions

### vs. RUST_BACKTRACE=1

| Feature | RUST_BACKTRACE | async-inspect |
|---------|----------------|---------------|
| Shows async call chain | ❌ No | ✅ Yes |
| Shows await points | ❌ No | ✅ Yes |
| Shows task state | ❌ No | ✅ Yes |
| Shows task relationships | ❌ No | ✅ Yes |
| Time in each state | ❌ No | ✅ Yes |
| Works in production | ✅ Yes | ✅ Yes (low overhead) |
| Zero cost when disabled | ✅ Yes | 🟡 Small |

### vs. tokio-console

| Feature | tokio-console | async-inspect |
|---------|---------------|---------------|
| Live task monitoring | ✅ Yes | ✅ Yes |
| Historical analysis | ❌ No | ✅ Yes |
| Panic analysis | ❌ Limited | ✅ Full |
| Deadlock detection | ✅ Yes | ✅ Yes |
| Export traces | ❌ No | ✅ JSON/CSV |
| Production safe | 🟡 High overhead | ✅ Low overhead |

**Best approach**: Use both!
- tokio-console for runtime observability
- async-inspect for debugging and failure analysis

### vs. tracing + tracing-subscriber

| Feature | tracing | async-inspect |
|---------|---------|---------------|
| Manual instrumentation | ✅ Flexible | ✅ Automatic |
| Task relationships | ❌ Limited | ✅ Full graph |
| State machine visibility | ❌ No | ✅ Yes |
| Await point tracking | ❌ No | ✅ Yes |
| Integration | ✅ Ecosystem | ✅ Compatible |

async-inspect works **with** tracing via the `AsyncInspectLayer`!

## Future Improvements

The Rust project is working on better async diagnostics:

- [RFC: async stack traces](https://github.com/rust-lang/rfcs/pull/3457)
- Improved panic messages for async
- Better debugger integration

async-inspect will **complement** these improvements:

```rust
// Future: Better built-in stack traces
// + async-inspect: Full task context, relationships, timeline

// Best of both worlds!
```

## Common Scenarios

### Scenario 1: "My async function panics randomly"

```bash
# Run with async-inspect
async-inspect monitor --watch

# When it panics, you see:
Task #123: process_payment [PANICKED]
  Blocked at: external_api_call().await
  Poll count: 1 (panicked on first poll!)
  Error: "TLS handshake failed"

# Diagnosis: Network issue, not your code
```

### Scenario 2: "Tests fail intermittently"

```bash
# Run tests with tracing
RUST_LOG=async_inspect=debug cargo test

# Failing test shows:
Task #5: test_user_creation [FAILED]
  Deadlock detected!
  - Task #5 waiting on Task #6 (mutex)
  - Task #6 waiting on Task #5 (channel)

# Diagnosis: Classic deadlock
```

### Scenario 3: "Production slow requests"

```bash
async-inspect analyze --slow --threshold 1s

Slow Tasks (>1s):
  Task #42: handle_checkout [2.3s]
    ├─ validate_cart      [50ms]  ✅
    ├─ charge_payment     [2.1s]  ⚠️  SLOW
    │  └─ external_api    [2.0s]  ← Problem!
    └─ send_confirmation  [100ms] ✅

# Diagnosis: External API slow, add timeout
```

## Best Practices

### 1. Annotate Critical Paths

```rust
// ✅ GOOD: Annotate user-facing handlers
#[async_inspect::trace]
async fn api_handler() { }

// ✅ GOOD: Annotate error-prone code
#[async_inspect::trace]
async fn risky_operation() { }

// ❌ BAD: Don't annotate everything
#[async_inspect::trace]
async fn tiny_helper() { }  // Too fine-grained
```

### 2. Use in Tests

```rust
#[tokio::test]
#[async_inspect::trace]  // ← Add this
async fn test_concurrent_access() {
    // If test fails, you get full async context
}
```

### 3. Production Sampling

```rust
Config {
    sampling_rate: 0.01,  // 1% overhead
    capture_backtraces: false,  // Expensive
    mode: Mode::Production,
}
```

### 4. Export Failures

```rust
// Auto-export failures for analysis
if let Some(failure) = inspector.last_failure() {
    let json = serde_json::to_string(&failure)?;
    log::error!("Task failure: {}", json);
}
```

## Limitations

async-inspect helps tremendously but **doesn't solve everything**:

- ❌ Doesn't replace proper error handling
- ❌ Doesn't fix bugs, just helps find them faster
- ❌ Small overhead even when optimized
- ❌ Requires instrumentation (manual or via tracing)

## Get Started

1. **Install**:
   ```bash
   cargo install async-inspect
   ```

2. **Add to project**:
   ```toml
   [dependencies]
   async-inspect = "0.1"
   ```

3. **Instrument**:
   ```rust
   #[async_inspect::trace]
   async fn your_function() { }
   ```

4. **Monitor**:
   ```bash
   async-inspect tui
   ```

That's it! Next time you hit an async panic, you'll have the context you need.

## Learn More

- [Installation Guide](./installation.md)
- [Quick Start](./quickstart.md)
- [Troubleshooting](./troubleshooting.md)
- [Examples](./examples.md)

---

**The async stack trace problem is real, but it's solvable.** async-inspect gives you the visibility you need to debug async Rust with confidence.
