//! Task hierarchy example
//!
//! This example demonstrates parent-child task relationships
//! and how they appear in the task relationship graph.

use async_inspect::prelude::*;
use async_inspect::runtime::tokio::spawn_tracked;
use std::time::Duration;
use tokio::time::sleep;

/// Root coordinator that spawns multiple child tasks
async fn coordinator() {
    println!("📊 Coordinator: Starting workflow...");

    // Spawn data fetching tasks (children of coordinator)
    let fetch1 = spawn_tracked("fetch_data_1", fetch_data(1)).await.unwrap();
    let fetch2 = spawn_tracked("fetch_data_2", fetch_data(2)).await.unwrap();
    let fetch3 = spawn_tracked("fetch_data_3", fetch_data(3)).await.unwrap();

    println!("📊 Coordinator: All data fetched");

    // Spawn processing task with the fetched data
    let _processed = spawn_tracked(
        "process_all",
        process_all_data(vec![fetch1, fetch2, fetch3]),
    )
    .await
    .unwrap();

    println!("📊 Coordinator: Processing complete");
}

/// Fetches data (simulated)
async fn fetch_data(id: u32) -> Vec<u8> {
    println!("  📥 Fetch #{}: Starting...", id);
    sleep(Duration::from_millis(50 + id as u64 * 20)).await;

    let data = vec![id as u8; 10];
    println!("  📥 Fetch #{}: Complete ({} bytes)", id, data.len());
    data
}

/// Processes all fetched data
async fn process_all_data(datasets: Vec<Vec<u8>>) -> Vec<u8> {
    println!("  ⚙️  Process: Starting with {} datasets", datasets.len());

    let mut results = Vec::new();

    for (i, data) in datasets.into_iter().enumerate() {
        // Spawn a worker task for each dataset
        let result = spawn_tracked(
            format!("worker_{}", i + 1),
            process_single_dataset(data, i as u32),
        )
        .await
        .unwrap();

        results.extend(result);
    }

    println!("  ⚙️  Process: Complete");
    results
}

/// Processes a single dataset
async fn process_single_dataset(data: Vec<u8>, id: u32) -> Vec<u8> {
    println!("    🔧 Worker #{}: Processing {} bytes", id + 1, data.len());
    sleep(Duration::from_millis(30)).await;

    let processed: Vec<u8> = data.iter().map(|&b| b.wrapping_mul(2)).collect();
    println!("    🔧 Worker #{}: Done", id + 1);
    processed
}

/// Manager that coordinates multiple teams
async fn manager() {
    println!("\n👔 Manager: Organizing teams...");

    // Spawn team leaders
    let team_a = spawn_tracked("team_leader_a", team_leader("A".to_string(), 2));
    let team_b = spawn_tracked("team_leader_b", team_leader("B".to_string(), 3));

    // Wait for both teams
    let _ = team_a.await;
    let _ = team_b.await;

    println!("👔 Manager: All teams complete");
}

/// Team leader that manages workers
async fn team_leader(name: String, num_workers: u32) {
    println!("  👨‍💼 Team {}: Starting with {} workers", name, num_workers);

    let mut workers = Vec::new();

    for i in 1..=num_workers {
        let worker_name = format!("worker_{}_{}", name, i);
        let team_name = name.clone();
        let handle = spawn_tracked(worker_name, worker_task(team_name, i));
        workers.push(handle);
    }

    // Wait for all workers
    for worker in workers {
        let _ = worker.await;
    }

    println!("  👨‍💼 Team {}: Complete", name);
}

/// Individual worker task
async fn worker_task(team: String, id: u32) {
    println!("    👷 Worker {}-{}: Working...", team, id);
    sleep(Duration::from_millis(40 + id as u64 * 10)).await;
    println!("    👷 Worker {}-{}: Done", team, id);
}

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  async-inspect - Task Hierarchy Example                   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("🎯 This example demonstrates parent-child task relationships\n");

    // Scenario 1: Data pipeline with hierarchy
    println!("═══ Scenario 1: Data Pipeline ═══");
    spawn_tracked("coordinator", coordinator()).await.unwrap();

    sleep(Duration::from_millis(50)).await;

    // Scenario 2: Team management hierarchy
    println!("\n═══ Scenario 2: Team Management ═══");
    spawn_tracked("manager", manager()).await.unwrap();

    sleep(Duration::from_millis(100)).await;

    println!("\n✅ All scenarios complete!\n");

    // Generate reports
    let reporter = Reporter::global();

    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Summary                                                    │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_summary();

    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ Gantt Timeline                                             │");
    println!("└────────────────────────────────────────────────────────────┘");
    reporter.print_gantt_timeline();

    // Generate HTML with relationship graph
    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ Generating HTML Report                                     │");
    println!("└────────────────────────────────────────────────────────────┘");

    let html_reporter = HtmlReporter::global();
    let html_path = "task_hierarchy_report.html";

    match html_reporter.save_to_file(html_path) {
        Ok(_) => {
            println!("\n✅ HTML report saved to: {}", html_path);
            println!("\n🌐 Open it to see the Task Relationship Graph showing:");
            println!("   📊 Hierarchical structure");
            println!("   👥 Parent-child relationships");
            println!("   🔗 Task dependencies");
            println!("   🎨 Color-coded by state");
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
    println!("║  Summary                                                   ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!("\n📊 Total tasks: {}", stats.total_tasks);
    println!("✅ Completed: {}", stats.completed_tasks);
    println!("📋 Total events: {}", stats.total_events);
    println!(
        "⏱️  Duration: {:.2}s",
        stats.timeline_duration.as_secs_f64()
    );

    println!("\n💡 The HTML report now shows a hierarchical graph with:");
    println!("   • Coordinator spawning fetch tasks");
    println!("   • Process task spawning worker tasks");
    println!("   • Manager spawning team leaders");
    println!("   • Team leaders spawning workers");
    println!("\n   This demonstrates the power of relationship visualization!");
    println!();
}
