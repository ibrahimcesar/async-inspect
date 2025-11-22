//! Proc macro test example
//!
//! This example demonstrates the #[async_inspect::trace] attribute macro
//! for automatic instrumentation of async functions.

use async_inspect::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

/// Fetch user profile - automatically instrumented
#[async_inspect::trace]
async fn fetch_user_profile(user_id: u32) -> String {
    println!("  📥 Fetching profile for user {}...", user_id);
    sleep(Duration::from_millis(80)).await;

    let profile = format!("Profile(id={})", user_id);
    println!("  ✓ Profile fetched: {}", profile);
    profile
}

/// Fetch user posts - automatically instrumented
#[async_inspect::trace]
async fn fetch_user_posts(user_id: u32) -> Vec<String> {
    println!("  📥 Fetching posts for user {}...", user_id);
    sleep(Duration::from_millis(120)).await;

    let posts = vec![
        format!("Post 1 by user {}", user_id),
        format!("Post 2 by user {}", user_id),
    ];
    println!("  ✓ Posts fetched: {} items", posts.len());
    posts
}

/// Process user data - automatically instrumented with multiple await points
#[async_inspect::trace]
async fn process_user_data(user_id: u32) -> (String, Vec<String>) {
    println!("⚙️  Processing user {}...", user_id);

    // These .await points will be automatically labeled and tracked!
    let profile = fetch_user_profile(user_id).await;
    let posts = fetch_user_posts(user_id).await;

    println!("✓ Processing complete for user {}", user_id);
    (profile, posts)
}

/// Aggregate data from multiple users - complex workflow
#[async_inspect::trace]
async fn aggregate_user_data(user_ids: Vec<u32>) -> Vec<(String, Vec<String>)> {
    println!("📊 Aggregating data for {} users...", user_ids.len());

    let mut results = Vec::new();

    for user_id in user_ids {
        // Each call has multiple await points - all automatically tracked!
        let user_data = process_user_data(user_id).await;
        results.push(user_data);
    }

    println!("✓ Aggregation complete");
    results
}

/// Complex pipeline with error handling
#[async_inspect::trace]
async fn complex_pipeline() -> std::result::Result<String, String> {
    println!("🚀 Starting complex pipeline...");

    // Step 1: Fetch data
    let data = fetch_user_profile(1).await;

    // Step 2: Validate (simulated)
    sleep(Duration::from_millis(50)).await;

    // Step 3: Transform
    let transformed = format!("Transformed({})", data);

    // Step 4: Save (simulated)
    sleep(Duration::from_millis(40)).await;

    println!("✓ Pipeline complete");
    Ok(transformed)
}

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  async-inspect - Proc Macro Test Example                  ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("🎯 This example tests the #[async_inspect::trace] macro\n");
    println!("   All async functions are automatically instrumented!");
    println!("   Every .await point is labeled and tracked.\n");

    // Scenario 1: Simple function with one await
    println!("═══ Scenario 1: Simple Function ═══");
    let _ = fetch_user_profile(42).await;

    sleep(Duration::from_millis(50)).await;

    // Scenario 2: Function with multiple awaits
    println!("\n═══ Scenario 2: Multiple Awaits ═══");
    let _ = process_user_data(123).await;

    sleep(Duration::from_millis(50)).await;

    // Scenario 3: Complex nested workflow
    println!("\n═══ Scenario 3: Complex Workflow ═══");
    let _ = aggregate_user_data(vec![1, 2, 3]).await;

    sleep(Duration::from_millis(50)).await;

    // Scenario 4: Pipeline with error handling
    println!("\n═══ Scenario 4: Pipeline ═══");
    match complex_pipeline().await {
        Ok(result) => println!("✓ Result: {}", result),
        Err(e) => println!("❌ Error: {}", e),
    }

    sleep(Duration::from_millis(100)).await;

    println!("\n✅ All scenarios complete!\n");

    // Generate reports
    let reporter = Reporter::global();

    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Summary Report                                             │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_summary();

    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ Gantt Timeline                                             │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_gantt_timeline();

    // Generate HTML report
    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ Generating HTML Report                                     │");
    println!("└────────────────────────────────────────────────────────────┘");

    let html_reporter = HtmlReporter::global();
    let html_path = "proc_macro_test_report.html";

    match html_reporter.save_to_file(html_path) {
        Ok(_) => {
            println!("\n✅ HTML report saved to: {}", html_path);
            println!("\n🌐 Look for these automatic instrumentations:");
            println!("   • fetch_user_profile::await#1");
            println!("   • fetch_user_posts::await#1");
            println!("   • process_user_data::await#1 (fetch_user_profile)");
            println!("   • process_user_data::await#2 (fetch_user_posts)");
            println!("   • aggregate_user_data::await#1 (process_user_data loop)");
            println!("   • complex_pipeline::await#1, #2, #3");
            println!(
                "\n   Open: file://{}/{}",
                std::env::current_dir().unwrap().display(),
                html_path
            );
        }
        Err(e) => {
            println!("\n❌ Failed to save HTML report: {}", e);
        }
    }

    let stats = Inspector::global().stats();
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Proc Macro Test Results                                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!("\n📊 Total tasks: {}", stats.total_tasks);
    println!("✅ Completed: {}", stats.completed_tasks);
    println!("📋 Total events: {}", stats.total_events);
    println!(
        "⏱️  Duration: {:.2}s",
        stats.timeline_duration.as_secs_f64()
    );

    println!("\n💡 The proc macro automatically:");
    println!("   ✓ Registers each function as a tracked task");
    println!("   ✓ Labels every .await point (await#1, await#2, etc.)");
    println!("   ✓ Tracks execution time for each await");
    println!("   ✓ Records completion or failure");
    println!("\n   No manual instrumentation needed!");
    println!();
}
