# Understanding Async State Machines

Learn what async state machines are, why they exist, and how they power async Rust.

## What is an Async State Machine?

An **async state machine** is a data structure that represents the execution state of an asynchronous operation. In Rust, every `async fn` is automatically transformed by the compiler into a state machine.

### From Code to State Machine

When you write this simple async function:

```rust
async fn fetch_user(id: u64) -> User {
    let profile = fetch_profile(id).await;
    let posts = fetch_posts(id).await;
    User { profile, posts }
}
```

The Rust compiler transforms it into a state machine (conceptually):

```rust
enum FetchUserState {
    Start { id: u64 },
    WaitingForProfile { id: u64, future: FetchProfileFuture },
    WaitingForPosts { profile: Profile, future: FetchPostsFuture },
    Done { result: User },
}
```

Each `.await` point creates a new state where execution can pause and resume.

## Why Do We Need State Machines?

### The Problem: Blocking Threads

Traditional synchronous code blocks entire threads:

```rust
// Synchronous - thread sits idle waiting
fn fetch_user_sync(id: u64) -> User {
    let profile = http_get_blocking("/profile");  // Thread blocked!
    let posts = http_get_blocking("/posts");      // Blocked again!
    User { profile, posts }
}
```

**With 10,000 concurrent users:**
- Need 10,000 threads
- Each thread: ~2MB stack
- Total: **20GB RAM** just for stacks! 😱

### The Solution: Async State Machines

```rust
// Async - yields control when waiting
async fn fetch_user(id: u64) -> User {
    let profile = fetch_profile(id).await;  // Yields, doesn't block
    let posts = fetch_posts(id).await;      // Yields, doesn't block
    User { profile, posts }
}
```

**With 10,000 concurrent users:**
- Need 4-8 threads
- Each future: ~2KB
- Total: **20MB RAM** for all tasks! ✨

## How State Machines Work

### State Transitions

```bash
┌─────────────┐
│   Start     │
│  (id: 42)   │
└──────┬──────┘
       │ fetch_profile().await
       ▼
┌─────────────┐
│ WaitProfile │  ◄──┐
│ (waiting)   │     │ Poll::Pending
└──────┬──────┘     │ (yield control)
       │ ready?     │
       ├────────────┘
       │ Poll::Ready(profile)
       ▼
┌─────────────┐
│ WaitPosts   │  ◄──┐
│ (waiting)   │     │ Poll::Pending
└──────┬──────┘     │
       │ ready?     │
       ├────────────┘
       │ Poll::Ready(posts)
       ▼
┌─────────────┐
│    Done     │
│  (result)   │
└─────────────┘
```

### The Poll Mechanism

```rust
// Simplified Future trait
trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),      // Done! Here's the result
    Pending,       // Not done yet, wake me later
}
```

When you `.await`:

1. Future is polled
2. If `Pending`: control returns to executor
3. Executor runs other futures
4. When ready, executor polls again
5. If `Ready(value)`: continue execution

## Visual Example: Web Request

```rust
async fn handle_request(req: Request) -> Response {
    let user = db.get_user(req.user_id).await;
    let perms = check_permissions(user).await;
    let data = fetch_data(perms).await;
    Response::ok(data)
}
```

Timeline of execution:

```bash
Time →
Task 1: ████░░░░░░░░████░░░░████████
        ↑   ↑       ↑   ↑   ↑
        │   │       │   │   └─ fetch_data
        │   │       │   └─ check_permissions
        │   │       └─ waiting (yielded)
        │   └─ db.get_user
        └─ start

Task 2:     ████████░░░░████████░░░░
            (runs while Task 1 waits!)

Key: ████ = running  ░░░░ = waiting/yielded
```

## Memory Layout

### State Machine Structure

```rust
// Your async function:
async fn example() {
    sleep(1.secs()).await;
    println!("Done!");
}

// Becomes approximately:
struct ExampleFuture {
    state: ExampleState,
}

enum ExampleState {
    Start,
    Sleeping(Sleep),
    Done,
}
```

**Memory efficiency:**
- Size = largest variant
- Stack allocated (usually)
- No heap overhead!

```
Stack Layout:
┌─────────────────────┐
│ ExampleFuture       │
│ ┌─────────────────┐ │
│ │ state:          │ │
│ │ Sleeping {      │ │  ← Current state
│ │   sleep: Sleep  │ │
│ │ }               │ │
│ └─────────────────┘ │
└─────────────────────┘
Total: ~32 bytes
```

## Zero-Cost Abstraction

Async/await is truly zero-cost:

1. **Compile time**: Everything resolved to state machine
2. **No runtime penalty**: No vtables, no dynamic dispatch
3. **LLVM optimized**: Aggressive inlining and optimization
4. **Predictable**: No garbage collection pauses

## Use Cases

### 1. Web Servers (Primary Use Case)

```rust
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users/:id", get(get_user));

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_user(Path(id): Path<u64>) -> Json<User> {
    let user = db.fetch_user(id).await;
    Json(user)
}
```

**Why async?** Handle thousands of concurrent requests with minimal threads.

### 2. Database Operations

```rust
async fn get_user_data(id: u64) -> UserData {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    user
}
```

**Why async?** Don't block while waiting for database I/O.

### 3. Concurrent I/O

```rust
async fn load_dashboard(user: User) -> Dashboard {
    // All three requests happen concurrently!
    let (profile, stats, notifications) = tokio::join!(
        fetch_profile(user.id),
        fetch_stats(user.id),
        fetch_notifications(user.id)
    );

    Dashboard { profile, stats, notifications }
}
```

**Why async?** Parallel I/O without spawning threads.

### 4. Real-Time Communication

```rust
async fn handle_websocket(socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let response = process(msg).await;
        socket.send(response).await;
    }
}
```

**Why async?** Efficient handling of many simultaneous connections.

## When NOT to Use Async

❌ **CPU-bound work**: Use threads or `spawn_blocking`
❌ **Simple scripts**: Overhead not worth it
❌ **Single I/O operation**: Just use sync
❌ **Pure computation**: No waiting involved

```rust
// ❌ DON'T DO THIS
async fn add(a: i32, b: i32) -> i32 {
    a + b  // No I/O! Async overhead for nothing!
}

// ✅ DO THIS
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

## Advantages

### 1. Resource Efficiency

| Approach | 10K Concurrent Ops | Memory Usage |
|----------|-------------------|--------------|
| Threads  | 10,000 threads    | ~20 GB       |
| Async    | 4-8 threads       | ~20 MB       |

### 2. Scalability

Handle 100K+ concurrent connections on commodity hardware.

### 3. Composability

Easy to combine async operations:

```rust
// Sequential
let a = fetch_a().await;
let b = fetch_b().await;

// Concurrent
let (a, b) = join!(fetch_a(), fetch_b());

// Racing
let first = select! {
    a = fetch_a() => a,
    b = fetch_b() => b,
};
```

### 4. Backpressure Handling

```rust
stream::iter(items)
    .map(|item| process(item))
    .buffer_unordered(10)  // Max 10 concurrent
    .collect()
    .await;
```

## Trade-offs

### Complexity

Async code is more complex:
- Must understand futures, `Pin`, `Send`/`Sync`
- "Function coloring" (async spreads through codebase)
- Learning curve

#### Function Coloring Problem

Once one function is async, **everything that calls it must be async too**. This "colors" your entire codebase:

```rust
// ❌ Can't call async from sync!
fn sync_handler() {
    let user = fetch_user(42).await;  // ERROR: can't use .await in non-async
}

// ✅ Must make caller async
async fn async_handler() {
    let user = fetch_user(42).await;  // OK
}
```

**How async-inspect helps:**

Use the CLI to visualize which parts of your codebase are async:

```bash
$ async-inspect analyze --show-call-graph

Call Graph (async functions):
┌─────────────────────────────────┐
│ main (async)                    │
│  ├─ handle_request (async)      │
│  │   ├─ fetch_user (async)      │  ← async spreads up
│  │   │   └─ db_query (async)    │
│  │   └─ render_template (sync)  │  ← can mix sync calls
│  └─ shutdown (async)             │
└─────────────────────────────────┘
```

The call graph shows:
- **Red functions**: Must be async (call async code)
- **Green functions**: Can be sync (no async calls)
- **Yellow functions**: Performance bottlenecks

This helps you understand:
1. **Where async is necessary** vs just convenient
2. **Async boundaries** in your codebase
3. **Opportunities to break async chains** with `spawn_blocking`

### Debugging Difficulty

Async stack traces are opaque:

```
thread 'tokio-runtime-worker' panicked
  at <impl Future for ...>::poll
  ??? (state machine internals)
```

**This is why async-inspect exists!** 🔍

We give you visibility into:
- Current state of each task
- Where tasks are blocked
- State transition history
- Task relationships

## Best Practices

### 1. Don't Block the Executor

```rust
// ❌ BAD
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));  // Blocks executor!
}

// ✅ GOOD
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;  // Yields
}
```

### 2. Use Timeouts

```rust
use tokio::time::timeout;

async fn with_timeout() -> Result<Data> {
    timeout(Duration::from_secs(5), fetch_data())
        .await
        .map_err(|_| Error::Timeout)?
}
```

### 3. Limit Concurrency

```rust
stream::iter(tasks)
    .map(|t| process(t))
    .buffer_unordered(100)  // Only 100 concurrent
    .collect()
    .await
```

### 4. Instrument Your Code

```rust
#[async_inspect::trace]  // ← Add this!
async fn important_function() {
    // Now you can debug it with async-inspect!
}
```

## Common Pitfalls

### 1. Forgetting `.await`

```rust
// ❌ Returns future, doesn't execute!
fetch_user(42);

// ✅ Actually executes
fetch_user(42).await;
```

### 2. Sequential Instead of Concurrent

```rust
// ❌ SLOW - waits for each one
let a = fetch_a().await;
let b = fetch_b().await;

// ✅ FAST - both run at once
let (a, b) = join!(fetch_a(), fetch_b());
```

### 3. Blocking in Async Context

```rust
// ❌ Blocks executor thread
async fn bad() {
    expensive_cpu_work();  // Blocks everyone!
}

// ✅ Runs on thread pool
async fn good() {
    spawn_blocking(|| expensive_cpu_work()).await
}
```

## How async-inspect Helps

### Visibility Into State Machines

Instead of this:
```
??? mysterious hang ???
```

You see:

```bash
┌─────────────────────────────────┐
│ Task #42: fetch_user            │
│ State: WaitingForPosts (2.3s)   │
│ Location: src/api.rs:156        │
│                                 │
│ Stuck at: fetch_posts().await   │
│ Reason: HTTP timeout            │
└─────────────────────────────────┘
```

### Features

- 📊 **Real-time monitoring** - See all task states
- 📈 **Timeline view** - Visualize execution flow
- 🔗 **Relationship graph** - Understand dependencies
- 💀 **Deadlock detection** - Find circular waits
- ⚡ **Performance analysis** - Identify bottlenecks

## Further Reading

- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Pin and Suffering](https://fasterthanli.me/articles/pin-and-suffering)

---

Next: [Installation Guide](./installation) →
