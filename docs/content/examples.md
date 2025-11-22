# Examples

Learn by example! This page shows real-world usage patterns.

## Basic Examples

### Simple Task Tracking

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn fetch_data(id: u64) -> String {
    // Simulated async work
    tokio::time::sleep(Duration::from_millis(100)).await;
    format!("Data for {}", id)
}

#[tokio::main]
async fn main() {
    let data = fetch_data(42).await;
    println!("{}", data);

    // Print summary
    Reporter::global().print_summary();
}
```

**Output:**
```
📊 Statistics:
  Total Tasks: 1
  Completed: 1
  Avg Duration: 100ms
```

### Multiple Tasks

```rust
use async_inspect::runtime::tokio::spawn_tracked;

#[tokio::main]
async fn main() {
    let mut handles = vec![];

    for i in 0..10 {
        let handle = spawn_tracked(
            format!("worker_{}", i),
            async move {
                tokio::time::sleep(Duration::from_millis(i * 10)).await;
                i * 2
            }
        );
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Reporter::global().print_summary();
}
```

## Web Server Example

### Axum Integration

```rust
use axum::{Router, routing::get, extract::Path, Json};
use async_inspect::prelude::*;

#[derive(serde::Serialize)]
struct User {
    id: u64,
    name: String,
}

#[async_inspect::trace]
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    // Simulated database query
    tokio::time::sleep(Duration::from_millis(50)).await;

    Json(User {
        id,
        name: format!("User {}", id),
    })
}

#[async_inspect::trace]
async fn list_users() -> Json<Vec<User>> {
    // Simulated database query
    tokio::time::sleep(Duration::from_millis(100)).await;

    Json(vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ])
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user));

    println!("Server running on http://localhost:3000");
    println!("Monitor with: async-inspect monitor");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**Monitor output:**
```
Active Tasks: 23
  #145 ⏳ get_user(42)        45ms
  #146 ⏳ get_user(43)        32ms
  #147 ✅ list_users          102ms
```

## Database Example

### SQLx Integration

```rust
use sqlx::PgPool;
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn fetch_user_from_db(pool: &PgPool, id: i64) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "SELECT id, name, email FROM users WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
}

#[async_inspect::trace]
async fn create_user(pool: &PgPool, name: String, email: String) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email",
        name,
        email
    )
    .fetch_one(pool)
    .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgresql://localhost/mydb").await?;

    let user = create_user(&pool, "Alice".into(), "alice@example.com".into()).await?;
    println!("Created user: {:?}", user);

    // Check performance
    Reporter::global().print_summary();

    Ok(())
}
```

## Concurrent Operations

### Parallel Requests

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn fetch_profile(id: u64) -> Profile {
    tokio::time::sleep(Duration::from_millis(100)).await;
    Profile { id }
}

#[async_inspect::trace]
async fn fetch_posts(id: u64) -> Vec<Post> {
    tokio::time::sleep(Duration::from_millis(150)).await;
    vec![Post { id, title: "Hello".into() }]
}

#[async_inspect::trace]
async fn fetch_friends(id: u64) -> Vec<Friend> {
    tokio::time::sleep(Duration::from_millis(80)).await;
    vec![Friend { id: id + 1 }]
}

#[async_inspect::trace]
async fn get_user_data(id: u64) -> UserData {
    // All three run concurrently!
    let (profile, posts, friends) = tokio::join!(
        fetch_profile(id),
        fetch_posts(id),
        fetch_friends(id)
    );

    UserData { profile, posts, friends }
}

#[tokio::main]
async fn main() {
    let data = get_user_data(42).await;

    // See that all three ran in parallel
    Reporter::global().print_summary();
}
```

**Output shows parallel execution:**
```
📊 Timeline:
  fetch_profile  ████████░░░░░░░░  100ms
  fetch_posts    ████████████░░░░  150ms  (overlapped!)
  fetch_friends  ██████░░░░░░░░░░   80ms  (overlapped!)
  Total:         150ms (not 330ms!)
```

## Error Handling

### With Results

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn risky_operation() -> Result<String, Box<dyn std::error::Error>> {
    if rand::random() {
        Ok("Success!".into())
    } else {
        Err("Failed!".into())
    }
}

#[tokio::main]
async fn main() {
    for i in 0..10 {
        match risky_operation().await {
            Ok(result) => println!("#{} {}", i, result),
            Err(e) => eprintln!("#{} Error: {}", i, e),
        }
    }

    // See success/failure ratio
    let stats = Inspector::global().stats();
    println!("Success rate: {}/{}",
        stats.completed_tasks,
        stats.total_tasks
    );
}
```

## Performance Analysis

### Finding Slow Operations

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn slow_operation(id: u64) {
    tokio::time::sleep(Duration::from_millis(id * 100)).await;
}

#[async_inspect::trace]
async fn fast_operation(id: u64) {
    tokio::time::sleep(Duration::from_millis(id)).await;
}

#[tokio::main]
async fn main() {
    for i in 1..=5 {
        tokio::join!(
            slow_operation(i),
            fast_operation(i)
        );
    }

    // Export for analysis
    JsonExporter::export_to_file(&Inspector::global(), "profile.json").unwrap();

    println!("Analyze with:");
    println!("  cat profile.json | jq '.tasks | sort_by(.duration_ms) | reverse | .[0:5]'");
}
```

## Deadlock Detection

### Finding Circular Dependencies

```rust
use async_inspect::prelude::*;
use async_inspect::graph::TaskGraph;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let mutex_a = Arc::new(Mutex::new(0));
    let mutex_b = Arc::new(Mutex::new(0));

    let a = mutex_a.clone();
    let b = mutex_b.clone();
    let task1 = tokio::spawn(async move {
        let _lock_a = a.lock().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _lock_b = b.lock().await;  // Might deadlock!
    });

    let a = mutex_a.clone();
    let b = mutex_b.clone();
    let task2 = tokio::spawn(async move {
        let _lock_b = b.lock().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _lock_a = a.lock().await;  // Might deadlock!
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Check for deadlocks
    let graph = TaskGraph::from_inspector(&Inspector::global());
    let deadlocks = graph.detect_potential_deadlocks();

    if !deadlocks.is_empty() {
        eprintln!("⚠️  Detected {} potential deadlocks!", deadlocks.len());
        for cycle in deadlocks {
            eprintln!("  Cycle: {:?}", cycle);
        }
    }
}
```

## Resource Contention

### Analyzing Shared Resources

```rust
use async_inspect::graph::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let db_pool = Arc::new(Mutex::new(DatabasePool::new()));

    // Spawn many workers
    let mut handles = vec![];
    for i in 0..100 {
        let pool = db_pool.clone();
        handles.push(tokio::spawn(async move {
            let _conn = pool.lock().await;  // Contention point!
            query_database().await;
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Analyze contention
    let graph = TaskGraph::from_inspector(&Inspector::global());
    let shared = graph.find_tasks_sharing_resource("db_pool");

    println!("Tasks contending for db_pool: {}", shared.len());
    println!("Consider increasing pool size!");
}
```

## Testing

### Debugging Flaky Tests

```rust
#[tokio::test]
#[async_inspect::trace]
async fn test_user_creation() {
    let user = create_user("Alice").await;
    assert!(user.is_valid());

    // If this fails, async-inspect shows where time was spent
    Reporter::global().print_summary();
}

#[tokio::test(flavor = "multi_thread")]
#[async_inspect::trace]
async fn test_concurrent_access() {
    let mut handles = vec![];

    for i in 0..100 {
        handles.push(tokio::spawn(async move {
            access_shared_resource(i).await
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Check for race conditions
    let stats = Inspector::global().stats();
    assert_eq!(stats.failed_tasks, 0, "Some tasks failed!");
}
```

## More Examples

Check out the [examples/ directory](https://github.com/ibrahimcesar/async-inspect/tree/main/examples) in the repository for more:

- `basic_inspection.rs` - Simple task tracking
- `tokio_integration.rs` - Tokio runtime integration
- `deadlock_detection.rs` - Finding deadlocks
- `performance_analysis.rs` - Profiling slow code
- `relationship_graph.rs` - Task relationships
- `ecosystem_integration.rs` - Prometheus, OpenTelemetry
- `production_ready.rs` - Production configuration
- `tui_monitor.rs` - Terminal UI usage

Run any example:

```bash
cargo run --example basic_inspection
cargo run --example tui_monitor --features cli
```

---

[← CLI Usage](./cli-usage) | [Production Guide →](./production)
