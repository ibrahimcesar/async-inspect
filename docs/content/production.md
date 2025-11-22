# Production Deployment

Using async-inspect in production environments requires careful configuration to minimize overhead while maintaining useful observability.

## Production Configuration

### Recommended Settings

```rust
use async_inspect::{Inspector, Config};

#[tokio::main]
async fn main() {
    let inspector = Inspector::new(Config {
        // Sample 1% of tasks to reduce overhead
        sampling_rate: 0.01,

        // Limit memory usage
        max_tasks: 10_000,
        max_events: 100_000,

        // Disable expensive features
        capture_backtraces: false,
        track_allocations: false,

        // Production mode optimizations
        mode: async_inspect::Mode::Production,

        ..Default::default()
    });

    // Your application code
}
```

### Configuration Modes

async-inspect provides three operational modes:

#### Development Mode (Default)

```rust
Config {
    mode: Mode::Development,
    sampling_rate: 1.0,  // Track all tasks
    capture_backtraces: true,
    track_allocations: true,
}
```

- **Use when**: Local development, debugging
- **Overhead**: High (5-15%)
- **Features**: All enabled

#### Production Mode

```rust
Config {
    mode: Mode::Production,
    sampling_rate: 0.01,  // Track 1% of tasks
    capture_backtraces: false,
    track_allocations: false,
}
```

- **Use when**: Live production traffic
- **Overhead**: Low `<1%`
- **Features**: Basic metrics only

#### Analysis Mode

```rust
Config {
    mode: Mode::Analysis,
    sampling_rate: 0.1,  // Track 10% of tasks
    capture_backtraces: true,
    track_allocations: false,
}
```

- **Use when**: Performance investigation
- **Overhead**: Medium (2-5%)
- **Features**: Detailed profiling

## Sampling Strategies

### Fixed Rate Sampling

Track a fixed percentage of tasks:

```rust
Config {
    sampling_rate: 0.01,  // 1%
    ..Default::default()
}
```

### Adaptive Sampling

Automatically adjust sampling based on load:

```rust
use async_inspect::sampling::AdaptiveSampler;

let sampler = AdaptiveSampler::new()
    .min_rate(0.001)   // 0.1% minimum
    .max_rate(0.1)     // 10% maximum
    .target_overhead(0.02);  // 2% overhead target

Config {
    sampler: Box::new(sampler),
    ..Default::default()
}
```

### Custom Sampling

Implement custom sampling logic:

```rust
use async_inspect::sampling::Sampler;

struct CustomSampler;

impl Sampler for CustomSampler {
    fn should_sample(&self, task_name: &str) -> bool {
        // Sample all API endpoints
        if task_name.contains("api::") {
            return true;
        }

        // Sample 1% of background tasks
        task_name.contains("background::") && rand::random::<f64>() < 0.01
    }
}
```

## Memory Management

### Setting Limits

Prevent unbounded memory growth:

```rust
Config {
    max_tasks: 10_000,        // Keep last 10k tasks
    max_events: 100_000,      // Keep last 100k events

    // Auto-cleanup old data
    cleanup_interval: Duration::from_secs(60),
    task_retention: Duration::from_secs(300),  // 5 minutes

    ..Default::default()
}
```

### Memory Monitoring

Track memory usage:

```rust
let stats = inspector.memory_stats();
println!("Tasks: {} / {}", stats.task_count, stats.max_tasks);
println!("Events: {} / {}", stats.event_count, stats.max_events);
println!("Memory: {} MB", stats.total_bytes / 1_048_576);

// Alert if approaching limits
if stats.task_count as f64 / stats.max_tasks as f64 > 0.9 {
    eprintln!("WARNING: Approaching task limit");
}
```

## Performance Overhead

### Benchmarks

Typical overhead by mode:

| Mode | Sampling | Overhead | Use Case |
|------|----------|----------|----------|
| Development | `100%` | `5-15%` | Local dev |
| Analysis | `10%` | `2-5%` | Debugging production |
| Production | `1%` | `<1%` | Always-on monitoring |

### Reducing Overhead

1. **Disable expensive features**:
   ```rust
   Config {
       capture_backtraces: false,  // Expensive
       track_allocations: false,   // Very expensive
       ..Default::default()
   }
   ```

2. **Use conditional compilation**:
   ```rust
   #[cfg_attr(feature = "inspect", async_inspect::trace)]
   async fn my_function() {
       // Only traced when "inspect" feature enabled
   }
   ```

3. **Selective tracing**:
   ```rust
   // Only trace critical paths
   #[async_inspect::trace]
   async fn handle_request() { }

   // Skip tracing for hot loops
   async fn process_batch() {
       // Not traced
   }
   ```

## Integration with Monitoring Systems

### Prometheus Export

Expose metrics for Prometheus scraping:

```rust
use async_inspect::integrations::PrometheusExporter;

let exporter = PrometheusExporter::new(inspector.clone());
exporter.start_server("0.0.0.0:9090").await?;
```

Metrics exposed:
- `async_inspect_tasks_total` - Total tasks created
- `async_inspect_tasks_by_state` - Tasks by state (running, blocked, completed)
- `async_inspect_task_duration_seconds` - Task duration histogram
- `async_inspect_events_total` - Total events
- `async_inspect_deadlocks_detected` - Deadlock count

### OpenTelemetry Export

Send traces to OpenTelemetry collector:

```rust
use async_inspect::integrations::OtelExporter;

let exporter = OtelExporter::new(
    inspector.clone(),
    "http://localhost:4317",  // OTLP endpoint
)?;
exporter.start().await?;
```

### Custom Export

Export to custom backends:

```rust
use async_inspect::export::JsonExporter;
use std::time::Duration;

// Export every 60 seconds
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;

        let json = JsonExporter::new(&inspector).export();

        // Send to your backend
        send_to_backend(&json).await;
    }
});
```

## Health Checks

### Readiness Probe

Check if async-inspect is healthy:

```rust
async fn health_check(inspector: Arc<Inspector>) -> Result<(), String> {
    let stats = inspector.memory_stats();

    // Check memory limits
    if stats.task_count >= stats.max_tasks {
        return Err("Task limit reached".to_string());
    }

    // Check for deadlocks
    if inspector.deadlocks().len() > 0 {
        return Err("Deadlocks detected".to_string());
    }

    Ok(())
}
```

### Liveness Probe

Ensure async-inspect is responding:

```rust
async fn liveness_check(inspector: Arc<Inspector>) -> bool {
    // Inspector should always be responsive
    inspector.task_count() >= 0
}
```

## Best Practices

### 1. Start with Low Sampling

Begin with 1% sampling and increase if needed:

```rust
// Production
Config { sampling_rate: 0.01, ..Default::default() }

// Investigation
Config { sampling_rate: 0.1, ..Default::default() }

// Critical issue
Config { sampling_rate: 1.0, ..Default::default() }
```

### 2. Use Feature Flags

Enable async-inspect only when needed:

```toml
# Cargo.toml
[dependencies]
async-inspect = { version = "0.1", optional = true }

[features]
inspect = ["async-inspect"]
```

```rust
#[cfg_attr(feature = "inspect", async_inspect::trace)]
async fn handler() { }
```

### 3. Monitor Performance Impact

Track overhead in production:

```rust
let start = Instant::now();

// Your code

let duration = start.elapsed();
metrics::histogram!("request_duration", duration);
```

### 4. Set Up Alerts

Alert on anomalies:

```yaml
# Prometheus alerts
groups:
  - name: async_inspect
    rules:
      - alert: HighTaskCount
        expr: async_inspect_tasks_by_state{state="running"} > 1000
        for: 5m

      - alert: DeadlockDetected
        expr: async_inspect_deadlocks_detected > 0

      - alert: SlowTasks
        expr: |
          histogram_quantile(0.99,
            async_inspect_task_duration_seconds_bucket
          ) > 10
```

### 5. Rotate Export Files

Prevent disk space issues:

```rust
use async_inspect::export::CsvExporter;

// Export with timestamp
let filename = format!(
    "async-inspect-{}.csv",
    chrono::Utc::now().format("%Y%m%d-%H%M%S")
);

CsvExporter::new(&inspector)
    .export_to_file(&filename)?;

// Clean up old files
cleanup_old_exports("async-inspect-*.csv", 7)?;  // Keep 7 days
```

## Environment Variables

Configure via environment:

```bash
# Sampling rate
export ASYNC_INSPECT_SAMPLING=0.01

# Memory limits
export ASYNC_INSPECT_MAX_TASKS=10000
export ASYNC_INSPECT_MAX_EVENTS=100000

# Export endpoint
export ASYNC_INSPECT_OTLP_ENDPOINT=http://localhost:4317

# Enable/disable
export ASYNC_INSPECT_ENABLED=true
```

```rust
let config = Config::from_env();
let inspector = Inspector::new(config);
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features inspect

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/myapp /usr/local/bin/

# Expose Prometheus metrics
EXPOSE 9090

ENV ASYNC_INSPECT_SAMPLING=0.01
ENV ASYNC_INSPECT_ENABLED=true

CMD ["myapp"]
```

### Kubernetes

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: async-inspect-config
data:
  sampling-rate: "0.01"
  max-tasks: "10000"
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
spec:
  template:
    spec:
      containers:
      - name: app
        image: myapp:latest
        env:
        - name: ASYNC_INSPECT_SAMPLING
          valueFrom:
            configMapKeyRef:
              name: async-inspect-config
              key: sampling-rate
        ports:
        - containerPort: 9090
          name: metrics
```

## Security Considerations

### 1. Sensitive Data

Avoid logging sensitive information:

```rust
#[async_inspect::trace(skip_args)]
async fn process_payment(card_number: &str) {
    // card_number not logged
}
```

### 2. Access Control

Restrict metrics endpoint:

```rust
// Require authentication
async fn metrics_handler(auth: Auth) -> Result<String, Error> {
    auth.require_admin()?;
    Ok(PrometheusExporter::render_metrics())
}
```

### 3. Rate Limiting

Prevent abuse of export endpoints:

```rust
use tower::limit::RateLimit;

let metrics_service = RateLimit::new(
    metrics_handler,
    Rate::new(10, Duration::from_secs(60)),  // 10 req/min
);
```

## Troubleshooting Production Issues

See [Troubleshooting Guide](./troubleshooting.md) for common production issues and solutions.

## Next Steps

- [Prometheus Integration](./integrations/prometheus.md)
- [OpenTelemetry Setup](./integrations/opentelemetry.md)
- [Troubleshooting](./troubleshooting.md)
