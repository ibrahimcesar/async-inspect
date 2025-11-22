# Tracing Subscriber Integration

Integrate async-inspect with the `tracing` ecosystem for automatic task tracking.

## Quick Start

```rust
use async_inspect::{Inspector, Config};
use async_inspect::integrations::AsyncInspectLayer;
use tracing_subscriber::prelude::*;

fn main() {
    let inspector = Inspector::new(Config::default());

    // Set up tracing subscriber with async-inspect layer
    tracing_subscriber::registry()
        .with(AsyncInspectLayer::new(inspector.clone()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Now all #[tracing::instrument] functions are automatically tracked!
}
```

## Installation

Add the `tracing-sub` feature:

```toml
[dependencies]
async-inspect = { version = "0.1", features = ["tracing-sub"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["registry"] }
```

## How It Works

The `AsyncInspectLayer` implements `tracing_subscriber::Layer` and automatically:

1. **Captures span lifecycle**: `new_span`, `enter`, `exit`, `close`
2. **Maps spans to tasks**: Async spans become async-inspect tasks
3. **Tracks relationships**: Parent-child span relationships preserved
4. **Records attributes**: Span fields become task metadata

## Basic Usage

### With #[tracing::instrument]

```rust
use tracing::instrument;

#[instrument]
async fn fetch_user(id: u64) -> User {
    // Automatically tracked by async-inspect!
    let profile = fetch_profile(id).await;
    let posts = fetch_posts(id).await;
    User { profile, posts }
}

#[instrument]
async fn fetch_profile(id: u64) -> Profile {
    // Also tracked
    db.query("SELECT * FROM profiles WHERE id = ?", id).await
}
```

### With Manual Spans

```rust
use tracing::info_span;

async fn handle_request(req: Request) -> Response {
    let span = info_span!("handle_request", route = ?req.route());
    let _enter = span.enter();

    // Work here is tracked
    process(req).await
}
```

## Configuration

### Layer Options

```rust
let layer = AsyncInspectLayer::builder()
    .inspector(inspector.clone())
    .track_sync_spans(false)      // Only track async spans (default: false)
    .capture_fields(true)          // Capture span fields (default: true)
    .max_field_length(1024)        // Truncate long fields (default: 1024)
    .build();
```

### Filtering

```rust
use tracing_subscriber::EnvFilter;

tracing_subscriber::registry()
    .with(
        AsyncInspectLayer::new(inspector.clone())
            .with_filter(EnvFilter::new("my_crate=debug"))  // Only track my_crate
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
```

### Multiple Layers

```rust
tracing_subscriber::registry()
    // async-inspect layer
    .with(AsyncInspectLayer::new(inspector.clone()))
    // fmt layer for console output
    .with(tracing_subscriber::fmt::layer())
    // OpenTelemetry layer
    .with(tracing_opentelemetry::layer())
    .init();
```

## Advanced Features

### Capturing Fields

Span fields become task attributes:

```rust
#[instrument(fields(user_id, action))]
async fn audit_log(user_id: u64, action: &str) {
    tracing::Span::current().record("user_id", user_id);
    tracing::Span::current().record("action", action);
    // Fields visible in async-inspect
}
```

View in CLI:
```bash
$ async-inspect monitor

Task: audit_log
  user_id: 12345
  action: "login"
  Duration: 234ms
```

### Events as Task Annotations

```rust
#[instrument]
async fn process_payment(amount: f64) {
    tracing::info!("Starting payment processing");

    validate(amount).await;
    tracing::info!("Payment validated");

    charge(amount).await;
    tracing::info!("Payment charged");

    // Events appear in timeline
}
```

Timeline view:
```
Task: process_payment [500ms]
  ├─ 0ms:   Starting payment processing
  ├─ 100ms: Payment validated
  └─ 500ms: Payment charged
```

### Error Tracking

```rust
#[instrument(err)]
async fn may_fail() -> Result<(), Error> {
    if random() {
        Err(Error::new("Random failure"))  // Automatically tracked
    } else {
        Ok(())
    }
}
```

Failed tasks show in red with error details.

### Custom Metadata

```rust
#[instrument(skip(db), fields(query_type = "read"))]
async fn query_db(db: &Database, id: u64) -> Row {
    // db not logged (skip)
    // query_type visible in async-inspect
    db.get(id).await
}
```

## Integration Patterns

### Web Framework (Axum)

```rust
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // Set up tracing + async-inspect
    let inspector = Inspector::new(Config::default());
    tracing_subscriber::registry()
        .with(AsyncInspectLayer::new(inspector.clone()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Tower middleware adds tracing
    let app = Router::new()
        .route("/users/:id", get(get_user))
        .layer(TraceLayer::new_for_http());

    // All requests automatically tracked!
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[instrument]
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    let user = fetch_user(id).await;
    Json(user)
}
```

### Database (sqlx)

```rust
use sqlx::PgPool;

#[instrument(skip(pool))]
async fn get_user_from_db(pool: &PgPool, id: u64) -> User {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}
```

sqlx already uses tracing internally - you get query tracking for free!

### Background Tasks (tokio)

```rust
#[instrument]
async fn background_worker() {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        process_batch().await;
    }
}

#[tokio::main]
async fn main() {
    // ... setup tracing ...

    tokio::spawn(background_worker());  // Tracked automatically
}
```

## Comparison with Direct Instrumentation

### Using #[async_inspect::trace]

```rust
#[async_inspect::trace]
async fn fetch_user(id: u64) -> User {
    // Directly tracked by async-inspect
}
```

**Pros**:
- Direct integration
- Lower overhead
- More control

**Cons**:
- async-inspect-specific
- Can't use with other tracing tools

### Using #[tracing::instrument] + Layer

```rust
#[tracing::instrument]
async fn fetch_user(id: u64) -> User {
    // Tracked via AsyncInspectLayer
}
```

**Pros**:
- Works with entire tracing ecosystem
- Compatible with tokio-console, OpenTelemetry, etc.
- More flexible filtering

**Cons**:
- Slightly higher overhead
- Indirect integration

**Recommendation**: Use tracing layer for flexibility, direct instrumentation for performance-critical paths.

## Performance

### Overhead Comparison

| Method | Overhead |
|--------|----------|
| No instrumentation | 0% |
| #[async_inspect::trace] | 2-3% |
| #[tracing::instrument] alone | 1-2% |
| #[tracing::instrument] + Layer | 3-5% |

### Optimization Tips

1. **Filter aggressively**:
   ```rust
   .with_filter(EnvFilter::new("my_crate::important=trace,my_crate=info"))
   ```

2. **Disable field capture** if not needed:
   ```rust
   AsyncInspectLayer::builder()
       .capture_fields(false)
       .build()
   ```

3. **Use sampling**:
   ```rust
   Inspector::new(Config {
       sampling_rate: 0.1,  // 10%
       ..Default::default()
   })
   ```

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=async_inspect=debug,tracing=debug cargo run
```

### Verify Layer is Active

```rust
let inspector = Inspector::new(Config::default());
let layer = AsyncInspectLayer::new(inspector.clone());

tracing_subscriber::registry()
    .with(layer)
    .init();

// Should see spans in async-inspect
tracing::info_span!("test").in_scope(|| {
    println!("Tasks: {}", inspector.task_count());  // Should be > 0
});
```

### Check Span Mapping

```rust
#[instrument]
async fn test() {
    println!("Span ID: {:?}", tracing::Span::current().id());
}

// Check if span is in async-inspect
test().await;
println!("Tasks: {:#?}", inspector.tasks());
```

## Examples

### Complete Web Server

```rust
use async_inspect::{Inspector, Config};
use async_inspect::integrations::AsyncInspectLayer;
use axum::{Router, routing::get, Json};
use tracing::instrument;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // Initialize inspector
    let inspector = Inspector::new(Config::default());

    // Set up tracing with async-inspect
    tracing_subscriber::registry()
        .with(AsyncInspectLayer::new(inspector.clone()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Start metrics server
    let metrics_inspector = inspector.clone();
    tokio::spawn(async move {
        metrics_server(metrics_inspector).await
    });

    // Start web server
    let app = Router::new()
        .route("/users/:id", get(get_user))
        .route("/health", get(health));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[instrument]
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    let user = fetch_user(id).await;
    Json(user)
}

#[instrument]
async fn fetch_user(id: u64) -> User {
    // Automatically tracked!
    User { id, name: "Alice".into() }
}

async fn metrics_server(inspector: Inspector) {
    // Expose metrics
    async_inspect::server::start(inspector, "0.0.0.0:9090").await;
}
```

### With Custom Filtering

```rust
use tracing_subscriber::{EnvFilter, Layer};

let inspector = Inspector::new(Config::default());

let async_inspect_layer = AsyncInspectLayer::new(inspector.clone())
    .with_filter(
        EnvFilter::new("my_crate=trace")
            .add_directive("sqlx=info".parse().unwrap())  // Reduce sqlx noise
            .add_directive("hyper=warn".parse().unwrap())  // Reduce hyper noise
    );

tracing_subscriber::registry()
    .with(async_inspect_layer)
    .with(tracing_subscriber::fmt::layer())
    .init();
```

## Compatibility

### Works With

- ✅ **tokio-console**: Use both simultaneously
- ✅ **tracing-opentelemetry**: Export to multiple backends
- ✅ **tracing-subscriber**: Full compatibility
- ✅ **tracing-appender**: File logging
- ✅ **tracing-flame**: Flamegraph generation

### Doesn't Work With

- ❌ **Multiple registries**: Only one registry per process
- ❌ **Global subscriber after init**: Can't change after `init()`

## Troubleshooting

### Spans not appearing

1. **Check layer is registered**:
   ```rust
   .with(AsyncInspectLayer::new(inspector.clone()))  // ← Must be called
   ```

2. **Verify span is entered**:
   ```rust
   let span = info_span!("test");
   let _enter = span.enter();  // ← Must enter
   ```

3. **Check filter**:
   ```bash
   RUST_LOG=trace cargo run  # Allow all spans
   ```

### High memory usage

Reduce captured fields:
```rust
AsyncInspectLayer::builder()
    .max_field_length(256)  // Truncate long values
    .build()
```

### Performance degradation

Use EnvFilter to reduce span volume:
```rust
.with_filter(EnvFilter::new("my_crate::critical=trace,my_crate=warn"))
```

## Best Practices

1. **Use tracing consistently**: Either use tracing everywhere or async_inspect macros everywhere
2. **Filter at registration**: More efficient than filtering at runtime
3. **Keep field values small**: Large values increase overhead
4. **Use skip for sensitive data**: `#[instrument(skip(password))]`

## Next Steps

- [Prometheus Integration](./prometheus.md)
- [OpenTelemetry Integration](./opentelemetry.md)
- [Production Deployment](../production.md)
