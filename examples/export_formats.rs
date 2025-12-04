//! Export Formats Example
//!
//! This example demonstrates all available export formats:
//! - JSON: Structured data for programmatic analysis
//! - CSV: Tasks and events in spreadsheet-compatible format
//! - Chrome Trace Event Format: For chrome://tracing and Perfetto UI
//! - Flamegraph: Folded stack format for performance visualization
//!
//! The example creates a realistic async workflow and exports it to all formats.

use async_inspect::export::{
    ChromeTraceExporter, CsvExporter, FlamegraphBuilder, FlamegraphExporter, JsonExporter,
};
use async_inspect::prelude::*;
use async_inspect::runtime::tokio::spawn_tracked;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;

/// Simulates an HTTP request
async fn http_request(url: &str, delay_ms: u64) -> String {
    inspect_point!("http_start", format!("GET {}", url));
    sleep(Duration::from_millis(delay_ms)).await;
    inspect_point!("http_complete");
    format!("Response from {}", url)
}

/// Simulates database query
async fn db_query(query: &str, delay_ms: u64) -> Vec<String> {
    inspect_point!("db_start", format!("Query: {}", query));
    sleep(Duration::from_millis(delay_ms)).await;
    inspect_point!("db_complete");
    vec!["row1".to_string(), "row2".to_string(), "row3".to_string()]
}

/// Simulates data processing
async fn process_data(data: Vec<String>) -> String {
    inspect_point!("processing_start", format!("{} items", data.len()));

    for (i, item) in data.iter().enumerate() {
        sleep(Duration::from_millis(20)).await;
        inspect_point!("process_item", format!("Processing item {}: {}", i, item));
    }

    inspect_point!("processing_complete");
    format!("Processed {} items", data.len())
}

/// Simulates caching operation
async fn cache_write(key: &str, _value: String) {
    inspect_point!("cache_start", format!("Key: {}", key));
    sleep(Duration::from_millis(30)).await;
    inspect_point!("cache_complete");
}

/// Main workflow demonstrating concurrent operations
async fn run_workflow() {
    println!("🚀 Starting async workflow...\n");

    // Spawn multiple concurrent API calls
    let api_tasks: Vec<_> = (1..=3)
        .map(|i| {
            spawn_tracked(format!("api_call_{}", i), async move {
                http_request(
                    &format!("https://api.example.com/endpoint/{}", i),
                    100 + i * 20,
                )
                .await
            })
        })
        .collect();

    // Spawn database operations
    let db_task = spawn_tracked("database_query".to_string(), async {
        db_query("SELECT * FROM users WHERE active = true", 150).await
    });

    // Wait for API calls
    let mut api_results = Vec::new();
    for task in api_tasks {
        if let Ok(result) = task.await {
            api_results.push(result);
        }
    }

    // Wait for database query
    let db_results = db_task.await.unwrap();

    // Process the combined results
    let processing_task = spawn_tracked("data_processing".to_string(), async move {
        let mut all_data = api_results;
        all_data.extend(db_results);
        process_data(all_data).await
    });

    let processed = processing_task.await.unwrap();

    // Cache the results
    let cache_task = spawn_tracked("cache_write".to_string(), async move {
        cache_write("workflow_result", processed).await;
    });

    cache_task.await.unwrap();

    // Give time for all events to be recorded
    sleep(Duration::from_millis(50)).await;

    println!("✅ Workflow completed!\n");
}

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  async-inspect - Export Formats Example                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Run the workflow
    run_workflow().await;

    let inspector = Inspector::global();
    let stats = inspector.stats();

    println!(
        "📊 Captured {} tasks and {} events",
        stats.total_tasks, stats.total_events
    );
    println!(
        "⏱️  Timeline duration: {:.2}ms\n",
        stats.timeline_duration.as_secs_f64() * 1000.0
    );

    // Create output directory
    let output_dir = "async_inspect_exports";
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Exporting to Multiple Formats                              │");
    println!("└────────────────────────────────────────────────────────────┘\n");

    // 1. Export to JSON
    println!("1️⃣  JSON Export");
    let json_path = format!("{}/data.json", output_dir);
    match JsonExporter::export_to_file(&inspector, &json_path) {
        Ok(_) => println!("   ✅ Saved to: {}", json_path),
        Err(e) => println!("   ❌ Failed: {}", e),
    }
    println!("   📝 Contains: Complete task data, events, and metadata");
    println!("   🔧 Use with: jq, JSON parsers, data analysis tools");
    println!();

    // 2. Export to CSV (Tasks)
    println!("2️⃣  CSV Export (Tasks)");
    let csv_tasks_path = format!("{}/tasks.csv", output_dir);
    match CsvExporter::export_tasks_to_file(&inspector, &csv_tasks_path) {
        Ok(_) => println!("   ✅ Saved to: {}", csv_tasks_path),
        Err(e) => println!("   ❌ Failed: {}", e),
    }
    println!("   📝 Contains: Task metrics (duration, polls, state)");
    println!("   🔧 Use with: Excel, Google Sheets, pandas");
    println!();

    // 3. Export to CSV (Events)
    println!("3️⃣  CSV Export (Events)");
    let csv_events_path = format!("{}/events.csv", output_dir);
    match CsvExporter::export_events_to_file(&inspector, &csv_events_path) {
        Ok(_) => println!("   ✅ Saved to: {}", csv_events_path),
        Err(e) => println!("   ❌ Failed: {}", e),
    }
    println!("   📝 Contains: Detailed event timeline");
    println!("   🔧 Use with: Excel, Google Sheets, pandas");
    println!();

    // 4. Export to Chrome Trace Event Format
    println!("4️⃣  Chrome Trace Event Format");
    let chrome_path = format!("{}/trace.json", output_dir);
    match ChromeTraceExporter::export_to_file(&inspector, &chrome_path) {
        Ok(_) => println!("   ✅ Saved to: {}", chrome_path),
        Err(e) => println!("   ❌ Failed: {}", e),
    }
    println!("   📝 Contains: Chrome-compatible trace events");
    println!("   🔧 Use with: chrome://tracing or Perfetto UI");
    println!("   🌐 Perfetto: https://ui.perfetto.dev/");
    println!();

    // 5. Export to Flamegraph (basic)
    println!("5️⃣  Flamegraph Export (Basic)");
    let flamegraph_path = format!("{}/flamegraph.txt", output_dir);
    match FlamegraphExporter::export_to_file(&inspector, &flamegraph_path) {
        Ok(_) => println!("   ✅ Saved to: {}", flamegraph_path),
        Err(e) => println!("   ❌ Failed: {}", e),
    }
    println!("   📝 Contains: Folded stack format with durations");
    println!("   🔧 Use with: flamegraph.pl, inferno, speedscope");
    println!("   🌐 Speedscope: https://www.speedscope.app/");
    println!();

    // 6. Export to Flamegraph (customized)
    println!("6️⃣  Flamegraph Export (Customized)");
    let flamegraph_custom_path = format!("{}/flamegraph_filtered.txt", output_dir);
    let custom_result = FlamegraphBuilder::new()
        .include_polls(false) // Exclude poll events for cleaner view
        .min_duration_ms(10) // Only include operations >= 10ms
        .export_to_file(&inspector, &flamegraph_custom_path);

    match custom_result {
        Ok(_) => println!("   ✅ Saved to: {}", flamegraph_custom_path),
        Err(e) => println!("   ❌ Failed: {}", e),
    }
    println!("   📝 Contains: Filtered flamegraph (no polls, 10ms+ only)");
    println!("   🔧 Use with: flamegraph.pl, inferno, speedscope");
    println!();

    // 7. Generate SVG Flamegraph (if feature enabled)
    #[cfg(feature = "flamegraph")]
    {
        println!("7️⃣  Flamegraph SVG (Direct visualization)");
        let svg_path = format!("{}/flamegraph.svg", output_dir);
        match FlamegraphExporter::generate_svg(&inspector, &svg_path) {
            Ok(_) => {
                println!("   ✅ Saved to: {}", svg_path);
                println!("   📝 Contains: Self-contained SVG visualization");
                println!("   🔧 Use with: Web browser, image viewer");
            }
            Err(e) => println!("   ❌ Failed: {}", e),
        }
        println!();
    }

    #[cfg(not(feature = "flamegraph"))]
    {
        println!("7️⃣  Flamegraph SVG");
        println!("   ⚠️  Feature not enabled. Rebuild with --features flamegraph");
        println!();
    }

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  Export Complete                                           ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    println!("📂 All files saved to: {}/", output_dir);
    println!();

    println!("🎯 Next Steps:");
    println!();
    println!("For Chrome Trace Event Format:");
    println!("  1. Open Chrome/Chromium browser");
    println!("  2. Navigate to: chrome://tracing");
    println!("  3. Click 'Load' and select: {}", chrome_path);
    println!("  4. Explore the interactive timeline!");
    println!();
    println!("  OR use Perfetto UI (recommended):");
    println!("  1. Go to: https://ui.perfetto.dev/");
    println!("  2. Click 'Open trace file'");
    println!("  3. Select: {}", chrome_path);
    println!();

    println!("For Flamegraph:");
    println!("  Online (easiest):");
    println!("  1. Go to: https://www.speedscope.app/");
    println!("  2. Drop the file: {}", flamegraph_path);
    println!();
    println!("  Or generate SVG locally:");
    println!("  cargo install inferno");
    println!(
        "  cat {} | inferno-flamegraph > flamegraph.svg",
        flamegraph_path
    );
    println!();

    println!("For CSV Analysis:");
    println!("  import pandas as pd");
    println!("  tasks = pd.read_csv('{}')", csv_tasks_path);
    println!("  events = pd.read_csv('{}')", csv_events_path);
    println!("  tasks.describe()  # Statistical summary");
    println!();

    println!("For JSON Analysis:");
    println!("  # Pretty print with jq");
    println!("  jq . {} | less", json_path);
    println!();
    println!("  # Extract task names");
    println!("  jq '.tasks[].name' {}", json_path);
    println!();
    println!("  # Find slowest task");
    println!(
        "  jq '.tasks | sort_by(.duration_ms) | reverse | .[0]' {}",
        json_path
    );
    println!();
}
