# Prometheus Integration

Export async-inspect metrics to Prometheus for monitoring and alerting.

## Quick Start

```rust
use async_inspect::{Inspector, Config};
use async_inspect::integrations::PrometheusExporter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create inspector
    let inspector = Inspector::new(Config::default());

    // Start Prometheus exporter
    let exporter = PrometheusExporter::new(inspector.clone());
    exporter.start_server("0.0.0.0:9090").await?;

    println!("Metrics available at http://localhost:9090/metrics");

    // Your application code
    Ok(())
}
```

## Installation

Add the `prometheus-export` feature:

```toml
[dependencies]
async-inspect = { version = "0.1", features = ["prometheus-export"] }
```

## Exported Metrics

### Task Metrics

#### `async_inspect_tasks_total`

**Type**: Counter
**Description**: Total number of tasks created
**Labels**:
- `name`: Task function name

```promql
# Rate of task creation
rate(async_inspect_tasks_total[5m])

# Top 10 most created tasks
topk(10, sum by (name) (async_inspect_tasks_total))
```

#### `async_inspect_tasks_by_state`

**Type**: Gauge
**Description**: Current number of tasks in each state
**Labels**:
- `state`: `running`, `blocked`, `completed`, `failed`

```promql
# Currently running tasks
async_inspect_tasks_by_state{state="running"}

# Percentage of blocked tasks
async_inspect_tasks_by_state{state="blocked"}
  / sum(async_inspect_tasks_by_state) * 100
```

#### `async_inspect_task_duration_seconds`

**Type**: Histogram
**Description**: Task execution duration
**Labels**:
- `name`: Task function name
- `state`: Final state (`completed`, `failed`)

**Buckets**: 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0

```promql
# 99th percentile task duration
histogram_quantile(0.99,
  sum(rate(async_inspect_task_duration_seconds_bucket[5m])) by (le, name)
)

# Average task duration by name
sum(rate(async_inspect_task_duration_seconds_sum[5m])) by (name)
  / sum(rate(async_inspect_task_duration_seconds_count[5m])) by (name)
```

### Event Metrics

#### `async_inspect_events_total`

**Type**: Counter
**Description**: Total number of events recorded
**Labels**:
- `type`: `poll`, `wake`, `drop`, `spawn`

```promql
# Event rate by type
rate(async_inspect_events_total[5m])

# Most active event types
topk(5, sum by (type) (rate(async_inspect_events_total[5m])))
```

#### `async_inspect_poll_count`

**Type**: Histogram
**Description**: Number of polls per task
**Labels**:
- `name`: Task function name

```promql
# Tasks with excessive polling
async_inspect_poll_count > 1000
```

### Deadlock Metrics

#### `async_inspect_deadlocks_detected`

**Type**: Counter
**Description**: Number of deadlocks detected

```promql
# Any deadlocks in last 5 minutes?
increase(async_inspect_deadlocks_detected[5m]) > 0
```

### Resource Metrics

#### `async_inspect_memory_bytes`

**Type**: Gauge
**Description**: Memory used by async-inspect
**Labels**:
- `component`: `tasks`, `events`, `metadata`

```promql
# Total memory usage
sum(async_inspect_memory_bytes)

# Memory by component
async_inspect_memory_bytes
```

## Configuration

### Custom Port

```rust
exporter.start_server("0.0.0.0:9091").await?;
```

### Custom Endpoint Path

```rust
let exporter = PrometheusExporter::builder()
    .path("/custom/metrics")  // Default: /metrics
    .port(9090)
    .build(inspector.clone());
```

### Update Interval

```rust
let exporter = PrometheusExporter::builder()
    .update_interval(Duration::from_secs(5))  // Default: 1s
    .build(inspector.clone());
```

### Custom Registry

```rust
use prometheus::Registry;

let registry = Registry::new();
let exporter = PrometheusExporter::with_registry(
    inspector.clone(),
    registry
);
```

## Grafana Dashboard

### Import Pre-built Dashboard

1. Download dashboard JSON: [async-inspect-dashboard.json](https://github.com/ibrahimcesar/async-inspect/grafana/dashboard.json)
2. Grafana → Dashboards → Import
3. Upload JSON file
4. Select Prometheus data source

### Dashboard Panels

The included dashboard has:

1. **Overview**
   - Total tasks
   - Active tasks
   - Task creation rate
   - Deadlock count

2. **Performance**
   - Task duration (p50, p95, p99)
   - Slowest tasks
   - Poll count distribution

3. **Task States**
   - Running tasks (time series)
   - Blocked tasks (time series)
   - Completed vs Failed ratio

4. **Events**
   - Event rate by type
   - Poll/Wake ratio
   - Spawn rate

### Custom Dashboard Example

```json
{
  "panels": [
    {
      "title": "Active Tasks",
      "targets": [
        {
          "expr": "async_inspect_tasks_by_state{state=\"running\"}"
        }
      ]
    },
    {
      "title": "Task Duration (p99)",
      "targets": [
        {
          "expr": "histogram_quantile(0.99, sum(rate(async_inspect_task_duration_seconds_bucket[5m])) by (le, name))"
        }
      ]
    }
  ]
}
```

## Alerting

### Prometheus Alert Rules

```yaml
groups:
  - name: async_inspect_alerts
    rules:
      # Alert on high number of running tasks
      - alert: HighTaskCount
        expr: async_inspect_tasks_by_state{state="running"} > 1000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High number of concurrent tasks"
          description: "{{ $value }} tasks currently running"

      # Alert on deadlocks
      - alert: DeadlockDetected
        expr: increase(async_inspect_deadlocks_detected[5m]) > 0
        labels:
          severity: critical
        annotations:
          summary: "Deadlock detected in async tasks"
          description: "{{ $value }} deadlock(s) detected"

      # Alert on slow tasks
      - alert: SlowTasksDetected
        expr: |
          histogram_quantile(0.99,
            sum(rate(async_inspect_task_duration_seconds_bucket[5m])) by (le, name)
          ) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Tasks taking longer than expected"
          description: "p99 duration: {{ $value }}s"

      # Alert on high blocked task percentage
      - alert: HighBlockedTaskPercentage
        expr: |
          async_inspect_tasks_by_state{state="blocked"}
            / sum(async_inspect_tasks_by_state) * 100 > 50
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "More than 50% of tasks are blocked"
          description: "{{ $value }}% of tasks blocked"

      # Alert on memory usage
      - alert: HighMemoryUsage
        expr: sum(async_inspect_memory_bytes) > 1e9  # 1GB
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "async-inspect using excessive memory"
          description: "Memory usage: {{ $value | humanize }}B"
```

## Recording Rules

Pre-aggregate expensive queries:

```yaml
groups:
  - name: async_inspect_recording
    interval: 10s
    rules:
      # Task creation rate
      - record: async_inspect:task_creation_rate:5m
        expr: rate(async_inspect_tasks_total[5m])

      # Average task duration
      - record: async_inspect:task_duration_avg:5m
        expr: |
          sum(rate(async_inspect_task_duration_seconds_sum[5m]))
            / sum(rate(async_inspect_task_duration_seconds_count[5m]))

      # Blocked task percentage
      - record: async_inspect:blocked_percentage
        expr: |
          async_inspect_tasks_by_state{state="blocked"}
            / sum(async_inspect_tasks_by_state) * 100
```

## Advanced Usage

### Multiple Inspectors

```rust
let inspector1 = Inspector::new(Config::default());
let inspector2 = Inspector::new(Config::default());

// Single exporter for both
let exporter = PrometheusExporter::multi(vec![
    inspector1.clone(),
    inspector2.clone(),
]);
```

### Custom Metrics

Add your own metrics to the same registry:

```rust
use prometheus::{Counter, Registry};

let registry = Registry::new();

// Your custom metric
let requests = Counter::new("http_requests_total", "Total requests")?;
registry.register(Box::new(requests.clone()))?;

// async-inspect metrics
let exporter = PrometheusExporter::with_registry(
    inspector.clone(),
    registry.clone()
);
```

### Filtering Metrics

```rust
let exporter = PrometheusExporter::builder()
    .enable_task_metrics(true)
    .enable_event_metrics(false)  // Disable event metrics
    .enable_memory_metrics(true)
    .build(inspector.clone());
```

## Integration with Existing Monitoring

### Kubernetes

```yaml
apiVersion: v1
kind: Service
metadata:
  name: myapp
  labels:
    app: myapp
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
    prometheus.io/path: "/metrics"
spec:
  selector:
    app: myapp
  ports:
  - name: metrics
    port: 9090
    targetPort: 9090
```

### Docker Compose

```yaml
version: '3'
services:
  app:
    build: .
    ports:
      - "9090:9090"
    labels:
      - "prometheus.scrape=true"
      - "prometheus.port=9090"

  prometheus:
    image: prom/prometheus
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
```

**prometheus.yml**:
```yaml
scrape_configs:
  - job_name: 'async-inspect'
    static_configs:
      - targets: ['app:9090']
```

### Systemd Service

```ini
[Unit]
Description=My Async App
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/myapp
Environment="ASYNC_INSPECT_METRICS_PORT=9090"
Restart=always

[Install]
WantedBy=multi-user.target
```

## Best Practices

### 1. Use Recording Rules

Pre-aggregate expensive queries:
```yaml
# Instead of calculating p99 in Grafana
- record: async_inspect:duration_p99:5m
  expr: histogram_quantile(0.99, ...)
```

### 2. Limit Cardinality

Avoid high-cardinality labels:
```rust
// ❌ BAD - too many unique values
task_duration{task_id="12345"}

// ✅ GOOD - bounded cardinality
task_duration{name="fetch_user"}
```

### 3. Set Appropriate Scrape Intervals

```yaml
scrape_configs:
  - job_name: 'async-inspect'
    scrape_interval: 15s  # Balance freshness vs load
```

### 4. Retention

Configure retention based on needs:
```yaml
# Prometheus
--storage.tsdb.retention.time=30d
--storage.tsdb.retention.size=50GB
```

## Troubleshooting

### Metrics not appearing

1. Check endpoint is accessible:
   ```bash
   curl http://localhost:9090/metrics
   ```

2. Verify Prometheus scraping:
   ```bash
   # Check targets in Prometheus UI
   http://localhost:9091/targets
   ```

3. Enable debug logging:
   ```rust
   env_logger::Builder::from_env(
       env_logger::Env::default().default_filter_or("async_inspect=debug")
   ).init();
   ```

### High scrape duration

Reduce metrics:
```rust
let exporter = PrometheusExporter::builder()
    .enable_event_metrics(false)  // Disable if not needed
    .build(inspector.clone());
```

## Examples

Complete example: [examples/prometheus_integration.rs](https://github.com/ibrahimcesar/async-inspect/examples/prometheus_integration.rs)

## Next Steps

- [OpenTelemetry Integration](./opentelemetry.md)
- [Tracing Subscriber](./tracing.md)
- [Production Deployment](../production.md)
