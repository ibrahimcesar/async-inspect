//! Production-ready example
//!
//! This example demonstrates using async-inspect in production environments
//! with minimal overhead through sampling, limits, and exports.

use async_inspect::config::Config;
use async_inspect::export::{CsvExporter, JsonExporter};
use async_inspect::prelude::*;
use async_inspect::runtime::tokio::spawn_tracked;
use colored::Colorize;
use std::time::Duration;
use tokio::time::sleep;

/// Simulated API request
#[async_inspect::trace]
async fn api_request(endpoint: &str, id: u32) -> std::result::Result<String, String> {
    sleep(Duration::from_millis(50 + id as u64 * 10)).await;

    if id % 20 == 0 {
        Err(format!("Failed to fetch {}", endpoint))
    } else {
        Ok(format!("Response from {} (id={})", endpoint, id))
    }
}

/// Database query simulation
#[async_inspect::trace]
async fn db_query(query: &str, _id: u32) -> Vec<String> {
    sleep(Duration::from_millis(30)).await;
    vec![format!("Result for: {}", query)]
}

/// Cache operation
#[async_inspect::trace]
async fn cache_get(key: &str) -> Option<String> {
    sleep(Duration::from_millis(5)).await;
    if key.len() % 2 == 0 {
        Some(format!("Cached: {}", key))
    } else {
        None
    }
}

/// Business logic handler
#[async_inspect::trace]
async fn handle_request(request_id: u32) -> std::result::Result<String, String> {
    // Check cache first
    let cache_key = format!("request_{}", request_id);
    if let Some(cached) = cache_get(&cache_key).await {
        return Ok(cached);
    }

    // Fetch from API
    let api_data = api_request("/users", request_id).await?;

    // Query database
    let db_data = db_query("SELECT * FROM users", request_id).await;

    Ok(format!("Processed: {} with {}", api_data, db_data.len()))
}

/// Simulate production workload with many concurrent requests
async fn production_workload() {
    println!("🚀 Starting production workload simulation...\n");

    // Spawn many concurrent requests (simulating real traffic)
    let mut handles = vec![];

    for i in 1..=1000 {
        let handle = spawn_tracked(format!("request_{}", i), async move {
            let _ = handle_request(i).await;
        });
        handles.push(handle);

        // Stagger the requests slightly
        if i % 10 == 0 {
            sleep(Duration::from_millis(5)).await;
        }
    }

    // Wait for all to complete
    for handle in handles {
        let _ = handle.await;
    }

    println!("✅ Workload complete!\n");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!(
        "║  {} - Production-Ready Example                 ║",
        "[async-inspect]".on_purple().white().bold()
    );
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Configure for production use
    let config = Config::global();
    config.production_mode();

    println!(
        "{}",
        "[STATS] Production Configuration:".on_blue().white().bold()
    );
    println!("   Sampling rate:   1 in {}", config.sampling_rate());
    println!("   Max events:      {}", config.max_events());
    println!("   Max tasks:       {}", config.max_tasks());
    println!("   Track awaits:    {}", config.track_awaits());
    println!("   Enable HTML:     {}", config.enable_html());
    println!();

    // Run the workload
    production_workload().await;

    // Give time for final events
    sleep(Duration::from_millis(100)).await;

    // Print overhead statistics
    let overhead = config.overhead_stats();
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Overhead Analysis                                          │");
    println!("└────────────────────────────────────────────────────────────┘");
    println!("  Total overhead:      {:.2}ms", overhead.total_ms());
    println!("  Instrumentation calls: {}", overhead.calls);
    println!("  Average per call:    {:.2}µs", overhead.avg_us());
    println!(
        "  Overhead percentage: {:.4}%",
        overhead.total_ms() / Inspector::global().stats().timeline_duration.as_secs_f64() / 10.0
    );
    println!();

    // Generate basic summary
    let reporter = Reporter::global();
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Summary Report                                             │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_summary();

    // Export to JSON
    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ Exporting Data                                             │");
    println!("└────────────────────────────────────────────────────────────┘");

    let json_path = "production_export.json";
    match JsonExporter::export_to_file(Inspector::global(), json_path) {
        Ok(_) => println!("  ✅ JSON exported to: {}", json_path),
        Err(e) => println!("  ❌ JSON export failed: {}", e),
    }

    // Export to CSV
    let csv_tasks_path = "production_tasks.csv";
    match CsvExporter::export_tasks_to_file(Inspector::global(), csv_tasks_path) {
        Ok(_) => println!("  ✅ Tasks CSV exported to: {}", csv_tasks_path),
        Err(e) => println!("  ❌ Tasks CSV export failed: {}", e),
    }

    let csv_events_path = "production_events.csv";
    match CsvExporter::export_events_to_file(Inspector::global(), csv_events_path) {
        Ok(_) => println!("  ✅ Events CSV exported to: {}", csv_events_path),
        Err(e) => println!("  ❌ Events CSV export failed: {}", e),
    }

    println!();

    // Performance analysis (only on sampled data)
    let profiler = Inspector::global().build_profiler();
    let perf_reporter = async_inspect::profile::PerformanceReporter::new(&profiler);

    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Performance Analysis (Sampled Data)                        │");
    println!("└────────────────────────────────────────────────────────────┘");
    perf_reporter.print_report();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Production Recommendations                                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("✅ Production Mode Benefits:");
    println!("   • Sampling reduces overhead by 99%");
    println!("   • Limited event/task storage prevents memory bloat");
    println!("   • Disabled HTML generation saves CPU cycles");
    println!("   • Still get statistical insights from sampled data");
    println!();
    println!("📊 Exported Data Can Be:");
    println!("   • Analyzed offline with tools like pandas, Excel");
    println!("   • Integrated with monitoring systems (Prometheus, Datadog)");
    println!("   • Stored in time-series databases for trending");
    println!("   • Used for post-mortem debugging");
    println!();
    println!("🎯 Tuning Recommendations:");
    println!("   • Adjust sampling rate based on traffic volume");
    println!("   • Monitor overhead percentage (should be < 0.1%)");
    println!("   • Export data periodically to external storage");
    println!("   • Use environment variables for runtime configuration");
    println!();

    Ok(())
}
