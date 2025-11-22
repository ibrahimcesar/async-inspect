# OpenTelemetry Integration

Export async-inspect traces to OpenTelemetry for distributed tracing and observability.

## Quick Start

```rust
use async_inspect::{Inspector, Config};
use async_inspect::integrations::OtelExporter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create inspector
    let inspector = Inspector::new(Config::default());

    // Export to OpenTelemetry Collector
    let exporter = OtelExporter::new(
        inspector.clone(),
        "http://localhost:4317",  // OTLP endpoint
    )?;
    exporter.start().await?;

    println!("Exporting traces to OpenTelemetry");

    // Your application code
    Ok(())
}
```

## Installation

Add the `opentelemetry-export` feature:

```toml
[dependencies]
async-inspect = { version = "0.1", features = ["opentelemetry-export"] }
```

## Configuration

### Basic Setup

```rust
let exporter = OtelExporter::builder()
    .endpoint("http://localhost:4317")
    .service_name("my-service")
    .service_version("0.1.0")
    .build(inspector.clone())?;
```

### With Headers (Authentication)

```rust
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("x-api-key".to_string(), "your-api-key".to_string());

let exporter = OtelExporter::builder()
    .endpoint("https://api.honeycomb.io:443")
    .headers(headers)
    .build(inspector.clone())?;
```

### Export Interval

```rust
let exporter = OtelExporter::builder()
    .endpoint("http://localhost:4317")
    .export_interval(Duration::from_secs(5))  // Default: 1s
    .build(inspector.clone())?;
```

### Batch Configuration

```rust
let exporter = OtelExporter::builder()
    .endpoint("http://localhost:4317")
    .max_batch_size(512)         // Default: 512
    .max_export_batch_size(512)  // Default: 512
    .build(inspector.clone())?;
```

## OpenTelemetry Collector

### Docker Compose

```yaml
version: '3'
services:
  # Your application
  app:
    build: .
    environment:
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
    depends_on:
      - otel-collector

  # OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
      - "13133:13133" # Health check
```

### Collector Configuration

**otel-collector-config.yaml**:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 1s
    send_batch_size: 1024

  # Add resource attributes
  resource:
    attributes:
      - key: service.name
        value: async-inspect-app
        action: upsert

exporters:
  # Export to Jaeger
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true

  # Export to Zipkin
  zipkin:
    endpoint: http://zipkin:9411/api/v2/spans

  # Export to Honeycomb
  otlp/honeycomb:
    endpoint: api.honeycomb.io:443
    headers:
      x-honeycomb-team: YOUR_API_KEY

  # Debug output
  logging:
    loglevel: debug

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch, resource]
      exporters: [jaeger, logging]
```

## Backend Integration

### Jaeger

```yaml
# docker-compose.yml
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # UI
      - "14250:14250"  # gRPC
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

Access UI: http://localhost:16686

### Zipkin

```yaml
services:
  zipkin:
    image: openzipkin/zipkin:latest
    ports:
      - "9411:9411"
```

Access UI: http://localhost:9411

### Honeycomb

```rust
let exporter = OtelExporter::builder()
    .endpoint("https://api.honeycomb.io:443")
    .header("x-honeycomb-team", "YOUR_API_KEY")
    .header("x-honeycomb-dataset", "async-inspect")
    .build(inspector.clone())?;
```

### Grafana Tempo

```yaml
# otel-collector-config.yaml
exporters:
  otlp/tempo:
    endpoint: tempo:4317
    tls:
      insecure: true
```

### AWS X-Ray

```yaml
exporters:
  awsxray:
    region: us-west-2
```

### Google Cloud Trace

```yaml
exporters:
  googlecloud:
    project: your-project-id
```

## Trace Structure

### Span Attributes

Each async task becomes an OpenTelemetry span with:

**Standard attributes**:
- `service.name`: Service name
- `span.kind`: Internal
- `thread.id`: Thread ID
- `task.id`: Async task ID
- `task.name`: Function name
- `task.parent_id`: Parent task ID (if any)

**Custom attributes**:
- `task.state`: Current state (running, blocked, completed)
- `task.poll_count`: Number of times polled
- `task.location`: Source code location
- `task.duration_ms`: Total duration

**Events**:
- `poll`: Task polled
- `wake`: Task woken
- `blocked`: Task blocked
- `completed`: Task completed
- `failed`: Task failed with error

### Example Trace

```
Span: handle_request [200ms]
  ├─ Span: auth_user [50ms]
  │  ├─ Event: poll (0ms)
  │  ├─ Event: blocked (25ms)
  │  └─ Event: completed (50ms)
  │
  ├─ Span: fetch_data [100ms]
  │  ├─ Span: db_query [80ms]
  │  │  ├─ Event: poll (0ms)
  │  │  ├─ Event: blocked (40ms)
  │  │  └─ Event: completed (80ms)
  │  └─ Event: completed (100ms)
  │
  └─ Span: render_response [50ms]
     └─ Event: completed (50ms)
```

## Advanced Usage

### Sampling

```rust
let exporter = OtelExporter::builder()
    .endpoint("http://localhost:4317")
    .sampler(OtelSampler::ParentBased(Box::new(
        OtelSampler::TraceIdRatioBased(0.1)  // Sample 10%
    )))
    .build(inspector.clone())?;
```

### Resource Attributes

```rust
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;

let resource = Resource::new(vec![
    KeyValue::new("service.name", "my-service"),
    KeyValue::new("service.version", "0.1.0"),
    KeyValue::new("deployment.environment", "production"),
    KeyValue::new("host.name", hostname()),
]);

let exporter = OtelExporter::builder()
    .endpoint("http://localhost:4317")
    .resource(resource)
    .build(inspector.clone())?;
```

### Custom Span Processors

```rust
let exporter = OtelExporter::builder()
    .endpoint("http://localhost:4317")
    .span_processor(MyCustomProcessor::new())
    .build(inspector.clone())?;
```

### Propagation

Automatically propagate trace context:

```rust
use opentelemetry::global;
use opentelemetry_http::HeaderExtractor;

// Extract context from HTTP headers
let parent_cx = global::get_text_map_propagator(|propagator| {
    propagator.extract(&HeaderExtractor(req.headers()))
});

// Attach to current scope
let _guard = parent_cx.attach();

// Now async-inspect will link spans to parent trace
handle_request().await;
```

## Querying Traces

### Jaeger Queries

Find slow tasks:
```
duration > 1s
service.name=my-service
task.state=completed
```

Find specific task:
```
task.name=fetch_user
task.id=12345
```

Find errors:
```
task.state=failed
error=true
```

### Honeycomb Queries

```sql
-- Average duration by task name
BREAKDOWN: task.name
CALCULATE: AVG(task.duration_ms)
WHERE: service.name = "my-service"

-- 99th percentile by endpoint
BREAKDOWN: http.route
CALCULATE: HEATMAP(task.duration_ms)
WHERE: task.name = "handle_request"

-- Error rate
CALCULATE: COUNT
WHERE: task.state = "failed"
GROUP BY: task.name
```

## Alerts

### Honeycomb

```yaml
trigger:
  - frequency: 5m
    threshold:
      op: ">"
      value: 1000
      column: "task.duration_ms"
      column_type: "float"

notification:
  - type: slack
    target: "#alerts"
```

### Grafana + Tempo

```yaml
groups:
  - name: async_inspect_traces
    rules:
      - alert: SlowTraces
        expr: |
          histogram_quantile(0.99,
            sum(rate(traces_spanmetrics_latency_bucket{service="my-service"}[5m])) by (le)
          ) > 1000
        for: 5m
```

## Best Practices

### 1. Meaningful Span Names

```rust
// ✅ GOOD - descriptive names
#[async_inspect::trace(name = "fetch_user_profile")]
async fn fetch_profile(id: u64) { }

// ❌ BAD - generic names
#[async_inspect::trace(name = "task")]
async fn process(id: u64) { }
```

### 2. Add Context Attributes

```rust
#[async_inspect::trace]
async fn fetch_user(id: u64) {
    // Add custom attribute
    tracing::info!(user_id = id, "Fetching user");
}
```

### 3. Limit Cardinality

```rust
// ❌ BAD - unbounded cardinality
#[async_inspect::trace(attributes = { "user_id": id })]
async fn process(id: u64) { }

// ✅ GOOD - bounded values
#[async_inspect::trace(attributes = { "operation": "read" })]
async fn process(id: u64) { }
```

### 4. Sample Strategically

```rust
// High-traffic endpoint: sample 1%
#[async_inspect::trace(sample_rate = 0.01)]
async fn health_check() { }

// Critical path: sample 100%
#[async_inspect::trace(sample_rate = 1.0)]
async fn payment_processing() { }
```

## Performance

### Overhead

| Configuration | Overhead |
|---------------|----------|
| No export | `0%` |
| Export (no sampling) | `2-5%` |
| Export (10% sampling) | `0.5-1%` |
| Export (1% sampling) | `<0.1%` |

### Optimization Tips

1. **Batch exports**:
   ```rust
   .max_batch_size(1024)
   ```

2. **Increase export interval**:
   ```rust
   .export_interval(Duration::from_secs(5))
   ```

3. **Use sampling**:
   ```rust
   .sampler(OtelSampler::TraceIdRatioBased(0.1))
   ```

4. **Compress data**:
   ```rust
   .compression(Compression::Gzip)
   ```

## Troubleshooting

### No traces appearing

1. **Check collector is running**:
   ```bash
   curl http://localhost:13133/  # Health check
   ```

2. **Verify endpoint**:
   ```rust
   RUST_LOG=async_inspect=debug cargo run
   ```

3. **Check firewall**:
   ```bash
   telnet localhost 4317
   ```

### High latency

1. **Increase batch size**:
   ```rust
   .max_batch_size(2048)
   ```

2. **Increase export interval**:
   ```rust
   .export_interval(Duration::from_secs(10))
   ```

### Spans not linking

Ensure context propagation:
```rust
use opentelemetry::global;

let parent_cx = global::get_text_map_propagator(|p| {
    p.extract(&HeaderExtractor(headers))
});
let _guard = parent_cx.attach();
```

## Examples

Complete example: [examples/opentelemetry_integration.rs](https://github.com/ibrahimcesar/async-inspect/examples/opentelemetry_integration.rs)

## Next Steps

- [Prometheus Integration](./prometheus.md)
- [Tracing Subscriber](./tracing.md)
- [Production Deployment](../production.md)
