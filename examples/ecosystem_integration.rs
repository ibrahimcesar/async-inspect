//! Ecosystem Integration Example
//!
//! This example demonstrates how to integrate async-inspect with the broader
//! Rust async ecosystem including:
//! - Tracing subscriber
//! - Prometheus metrics (optional)
//! - OpenTelemetry export (optional)
//! - Tokio-console compatibility
//!
//! Run with different feature flags to test integrations:
//! - Basic: cargo run --example ecosystem_integration
//! - With Prometheus: cargo run --example ecosystem_integration --features prometheus-export
//! - With OpenTelemetry: cargo run --example ecosystem_integration --features opentelemetry-export
//! - Full: cargo run --example ecosystem_integration --features full

use async_inspect::prelude::*;
use async_inspect::runtime::tokio::spawn_tracked;
use std::time::Duration;
use tokio::time::sleep;

/// Sample async task for demonstration
#[async_inspect::trace]
async fn fetch_data(id: u32) -> std::result::Result<String, String> {
    sleep(Duration::from_millis(100 + id as u64 * 10)).await;

    if id % 5 == 0 {
        Err(format!("Failed to fetch data for id {}", id))
    } else {
        Ok(format!("Data for id {}", id))
    }
}

/// Process data with multiple steps
#[async_inspect::trace]
async fn process_data(id: u32) {
    match fetch_data(id).await {
        Ok(data) => {
            // Simulate processing
            sleep(Duration::from_millis(50)).await;
            println!("Processed: {}", data);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

/// Orchestrator task that spawns workers
#[async_inspect::trace]
async fn orchestrator(count: u32) {
    let mut handles = vec![];

    for i in 1..=count {
        let handle = spawn_tracked(format!("worker_{}", i), process_data(i));
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  async-inspect - Ecosystem Integration Example            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Print tokio-console integration info
    async_inspect::integrations::tokio_console::print_integration_info();

    println!("🚀 Starting workload...\n");

    // Run the orchestrator
    orchestrator(10).await;

    // Give time for all events to be recorded
    sleep(Duration::from_millis(100)).await;

    println!("\n✅ Workload complete!\n");

    // Print basic statistics
    let reporter = Reporter::global();
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Basic Statistics                                           │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_summary();

    // Prometheus export (if enabled)
    #[cfg(feature = "prometheus-export")]
    {
        println!("\n┌────────────────────────────────────────────────────────────┐");
        println!("│ Prometheus Metrics Export                                  │");
        println!("└────────────────────────────────────────────────────────────┘\n");

        let exporter = async_inspect::integrations::prometheus::PrometheusExporter::new().unwrap();
        exporter.update();

        let metrics = exporter.gather();
        println!("{}", metrics);

        println!("💡 Tip: Expose these metrics on /metrics endpoint in production\n");
    }

    // OpenTelemetry export (if enabled)
    #[cfg(feature = "opentelemetry-export")]
    {
        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ OpenTelemetry Export                                       │");
        println!("└────────────────────────────────────────────────────────────┘\n");

        let otel_exporter =
            async_inspect::integrations::opentelemetry::OtelExporter::new("async-inspect-example");
        otel_exporter.export_tasks();

        println!(
            "✅ Exported {} tasks to OpenTelemetry",
            Inspector::global().stats().total_tasks
        );
        println!("💡 Tip: Configure OTLP endpoint to send to Jaeger/Zipkin\n");
    }

    // JSON export for post-mortem analysis
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Export for Offline Analysis                                │");
    println!("└────────────────────────────────────────────────────────────┘\n");

    let json_path = "ecosystem_trace.json";
    async_inspect::export::JsonExporter::export_to_file(&Inspector::global(), json_path)?;
    println!("✅ Exported trace to: {}", json_path);

    // Print integration recommendations
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Integration Recommendations                               ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("📊 Tracing Integration:");
    println!("   Use AsyncInspectLayer to automatically capture all async tasks:");
    println!("   ```rust");
    println!("   use tracing_subscriber::prelude::*;");
    println!("   tracing_subscriber::registry()");
    println!("       .with(AsyncInspectLayer::new())");
    println!("       .init();");
    println!("   ```\n");

    #[cfg(feature = "prometheus-export")]
    {
        println!("📈 Prometheus Integration:");
        println!("   Expose metrics endpoint in your web server:");
        println!("   ```rust");
        println!("   let exporter = Arc::new(PrometheusExporter::new());");
        println!("   exporter.clone().start_background_updater(Duration::from_secs(5));");
        println!("   // In your web handler: exporter.gather()");
        println!("   ```\n");
    }

    #[cfg(feature = "opentelemetry-export")]
    {
        println!("🔭 OpenTelemetry Integration:");
        println!("   Configure OTLP exporter for your observability backend:");
        println!("   ```rust");
        println!("   let exporter = create_otlp_exporter(");
        println!("       \"my-service\",");
        println!("       \"http://jaeger:4317\"");
        println!("   );");
        println!("   ```\n");
    }

    println!("🎯 Production Setup:");
    println!("   1. Enable production mode for minimal overhead");
    println!("   2. Choose ONE export method (Prometheus OR OpenTelemetry)");
    println!("   3. Use sampling to reduce data volume");
    println!("   4. Export JSON traces periodically for archival\n");

    println!("🔗 Grafana Dashboard:");
    println!("   Import async-inspect Prometheus metrics into Grafana");
    println!("   Key metrics to monitor:");
    println!("   • async_inspect_active_tasks");
    println!("   • async_inspect_blocked_tasks");
    println!("   • async_inspect_task_duration_seconds");
    println!("   • async_inspect_tasks_failed_total\n");

    Ok(())
}
