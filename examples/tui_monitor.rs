//! TUI monitoring example
//!
//! This example demonstrates the interactive terminal interface for
//! real-time async task monitoring.
//!
//! Run with: cargo run --example tui_monitor

use async_inspect::prelude::*;
use async_inspect::runtime::tokio::spawn_tracked;
use async_inspect::tui::run_tui;
use std::time::Duration;
use tokio::time::sleep;

/// Background worker that runs continuously
#[async_inspect::trace]
async fn background_worker(id: u32, iterations: u32) {
    for i in 1..=iterations {
        // Simulate work
        sleep(Duration::from_millis(100 + (id * 50) as u64)).await;

        if i % 5 == 0 {
            // Occasionally do more work
            sleep(Duration::from_millis(200)).await;
        }
    }
}

/// Periodic task that runs on a schedule
#[async_inspect::trace]
async fn periodic_task(_name: String, interval_ms: u64, count: u32) {
    for _ in 0..count {
        sleep(Duration::from_millis(interval_ms)).await;
    }
}

/// Task that simulates occasional failures
#[async_inspect::trace]
async fn flaky_task(id: u32) -> std::result::Result<(), String> {
    sleep(Duration::from_millis(150)).await;

    if id % 7 == 0 {
        Err(format!("Task {} failed", id))
    } else {
        Ok(())
    }
}

/// Long-running computation
#[async_inspect::trace]
async fn compute_intensive(iterations: u32) {
    for _ in 0..iterations {
        // Simulate CPU-bound work
        sleep(Duration::from_millis(50)).await;
    }
}

/// Quick task
#[async_inspect::trace]
async fn quick_task(_id: u32) {
    sleep(Duration::from_millis(20)).await;
}

/// Spawn background workload
async fn spawn_background_workload() {
    // Spawn various types of tasks
    for i in 1..=5 {
        spawn_tracked(format!("worker_{}", i), background_worker(i, 20));
    }

    for i in 1..=3 {
        spawn_tracked(
            format!("periodic_{}", i),
            periodic_task(format!("task_{}", i), 200 + i as u64 * 50, 10),
        );
    }

    // Spawn some quick tasks continuously
    tokio::spawn(async {
        for i in 1..=100 {
            spawn_tracked(format!("quick_{}", i), quick_task(i));
            sleep(Duration::from_millis(500)).await;
        }
    });

    // Spawn flaky tasks
    tokio::spawn(async {
        for i in 1..=50 {
            let task_id = i;
            spawn_tracked(format!("flaky_{}", i), async move {
                let _ = flaky_task(task_id).await;
            });
            sleep(Duration::from_millis(800)).await;
        }
    });

    // Spawn compute-intensive tasks
    for i in 1..=3 {
        spawn_tracked(format!("compute_{}", i), compute_intensive(30));
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  async-inspect - TUI Monitor Example                      ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("🎯 Starting background workload...");
    println!();
    println!("Keyboard shortcuts:");
    println!("  [q]     Quit");
    println!("  [s]     Cycle sort mode");
    println!("  [f]     Cycle filter mode");
    println!("  [↑↓]    Navigate tasks");
    println!("  [h/?]   Show help");
    println!();
    println!("Starting TUI in 2 seconds...");

    // Spawn background workload
    spawn_background_workload().await;

    // Give some time for tasks to start
    sleep(Duration::from_millis(2000)).await;

    // Run the TUI
    run_tui(Inspector::global().clone())?;

    println!("\n✅ TUI closed. Generating final report...\n");

    // Generate final summary
    let reporter = Reporter::global();
    reporter.print_summary();

    // Generate performance report
    let profiler = Inspector::global().build_profiler();
    let perf_reporter = async_inspect::profile::PerformanceReporter::new(&profiler);
    println!("\n");
    perf_reporter.print_report();

    Ok(())
}
