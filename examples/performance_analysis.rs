//! Performance analysis example
//!
//! This example demonstrates performance profiling capabilities:
//! - Task duration tracking
//! - Hot path detection
//! - Statistical analysis (p50, p95, p99)
//! - Bottleneck identification
//! - Efficiency analysis

use async_inspect::prelude::*;
use async_inspect::profile::PerformanceReporter;
use async_inspect::runtime::tokio::spawn_tracked;
use std::time::Duration;
use tokio::time::sleep;

/// Fast operation - completes quickly
#[async_inspect::trace]
async fn fast_operation(id: u32) -> u32 {
    sleep(Duration::from_millis(10 + id as u64 * 2)).await;
    id * 2
}

/// Medium operation - moderate duration
#[async_inspect::trace]
async fn medium_operation(id: u32) -> u32 {
    sleep(Duration::from_millis(50 + id as u64 * 5)).await;
    id * 3
}

/// Slow operation - potential bottleneck
#[async_inspect::trace]
async fn slow_operation(id: u32) -> u32 {
    sleep(Duration::from_millis(150 + id as u64 * 10)).await;
    id * 4
}

/// Inefficient operation - lots of blocking time
#[async_inspect::trace]
async fn inefficient_operation(id: u32) -> u32 {
    // Multiple small awaits (high blocked time ratio)
    for _ in 0..10 {
        sleep(Duration::from_millis(15)).await;
    }
    id
}

/// Busy operation - many polls
#[async_inspect::trace]
async fn busy_operation(id: u32) -> u32 {
    let mut result = id;
    for i in 0..5 {
        // Simulate work with short sleeps
        sleep(Duration::from_millis(5)).await;
        result += i;
    }
    result
}

/// Hot path - executed frequently
#[async_inspect::trace]
async fn hot_path_function(value: u32) -> u32 {
    sleep(Duration::from_millis(20)).await;
    value + 1
}

/// Coordinator that triggers various workloads
async fn performance_test_workload() {
    println!("🚀 Starting performance test workload...\n");

    // Scenario 1: Fast operations (should not be bottlenecks)
    println!("📊 Running fast operations...");
    let fast_tasks: Vec<_> = (1..=10)
        .map(|id| spawn_tracked(format!("fast_{}", id), fast_operation(id)))
        .collect();

    for task in fast_tasks {
        let _ = task.await;
    }

    // Scenario 2: Medium operations
    println!("📊 Running medium operations...");
    let medium_tasks: Vec<_> = (1..=8)
        .map(|id| spawn_tracked(format!("medium_{}", id), medium_operation(id)))
        .collect();

    for task in medium_tasks {
        let _ = task.await;
    }

    // Scenario 3: Slow operations (potential bottlenecks)
    println!("📊 Running slow operations (potential bottlenecks)...");
    let slow_tasks: Vec<_> = (1..=5)
        .map(|id| spawn_tracked(format!("slow_{}", id), slow_operation(id)))
        .collect();

    for task in slow_tasks {
        let _ = task.await;
    }

    // Scenario 4: Inefficient operations (high blocked time)
    println!("📊 Running inefficient operations...");
    let inefficient_tasks: Vec<_> = (1..=6)
        .map(|id| spawn_tracked(format!("inefficient_{}", id), inefficient_operation(id)))
        .collect();

    for task in inefficient_tasks {
        let _ = task.await;
    }

    // Scenario 5: Busy operations (many polls)
    println!("📊 Running busy operations...");
    let busy_tasks: Vec<_> = (1..=5)
        .map(|id| spawn_tracked(format!("busy_{}", id), busy_operation(id)))
        .collect();

    for task in busy_tasks {
        let _ = task.await;
    }

    // Scenario 6: Hot path (executed many times)
    println!("📊 Executing hot path multiple times...");
    for i in 1..=20 {
        spawn_tracked(format!("hotpath_{}", i), hot_path_function(i))
            .await
            .unwrap();
    }

    println!("\n✅ Workload complete!\n");
}

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  async-inspect - Performance Analysis Example             ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("🎯 This example demonstrates performance profiling:");
    println!("   • Task duration tracking");
    println!("   • Hot path detection");
    println!("   • Statistical analysis (p50, p95, p99)");
    println!("   • Bottleneck identification");
    println!("   • Efficiency analysis");
    println!();

    // Run the performance test workload
    performance_test_workload().await;

    // Give some time for all events to be recorded
    sleep(Duration::from_millis(100)).await;

    // Build profiler from collected data
    let profiler = Inspector::global().build_profiler();

    // Generate performance report
    let reporter = PerformanceReporter::new(&profiler);
    reporter.print_report();
    reporter.print_recommendations();

    // Also generate basic summary report
    let basic_reporter = Reporter::global();
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ Gantt Timeline                                             │");
    println!("└────────────────────────────────────────────────────────────┘");
    basic_reporter.print_gantt_timeline();

    // Generate HTML report
    println!("\n┌────────────────────────────────────────────────────────────┐");
    println!("│ Generating HTML Report                                     │");
    println!("└────────────────────────────────────────────────────────────┘");

    let html_reporter = HtmlReporter::global();
    let html_path = "performance_analysis_report.html";

    match html_reporter.save_to_file(html_path) {
        Ok(_) => {
            println!("\n✅ HTML report saved to: {}", html_path);
            println!(
                "   Open: file://{}/{}",
                std::env::current_dir().unwrap().display(),
                html_path
            );
        }
        Err(e) => {
            println!("\n❌ Failed to save HTML report: {}", e);
        }
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  Performance Analysis Complete                             ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("💡 Key Insights:");
    println!("   • Check the bottleneck section for slow tasks");
    println!("   • Hot paths show frequently executed code");
    println!("   • Efficiency analysis reveals tasks with high blocked time");
    println!("   • Use p95/p99 latencies for SLA monitoring");
    println!();
}
