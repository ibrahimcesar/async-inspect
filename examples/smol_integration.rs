//! Example demonstrating smol runtime integration
//!
//! This example shows how to use async-inspect with the smol runtime.
//!
//! Run with: cargo run --example smol_integration --features smol-runtime

use async_inspect::runtime::smol::{spawn_tracked, InspectExt};
use async_inspect::prelude::Inspector;
use std::time::Duration;

async fn fetch_data(id: u32) -> String {
    smol::Timer::after(Duration::from_millis(100 * id as u64)).await;
    format!("Data for ID: {}", id)
}

async fn process_data(data: String) -> String {
    smol::Timer::after(Duration::from_millis(50)).await;
    format!("Processed: {}", data)
}

async fn worker_task(worker_id: u32) {
    println!("Worker {} starting", worker_id);

    // Use InspectExt trait to track inline futures
    let data = fetch_data(worker_id)
        .inspect(&format!("fetch_data_{}", worker_id))
        .await;

    println!("Worker {} fetched: {}", worker_id, data);

    let processed = process_data(data)
        .inspect(&format!("process_data_{}", worker_id))
        .await;

    println!("Worker {} completed: {}", worker_id, processed);
}

fn main() {
    println!("smol Runtime Integration Example");
    println!("=================================\n");

    smol::block_on(async {
        // Create multiple tracked tasks using spawn_tracked
        let tasks: Vec<_> = (1..=5)
            .map(|i| spawn_tracked(format!("worker_{}", i), worker_task(i)))
            .collect();

        // Wait for all tasks to complete
        for task in tasks {
            task.await;
        }

        // Give a moment for all events to be recorded
        smol::Timer::after(Duration::from_millis(100)).await;

        // Get the global inspector and print statistics
        let inspector = Inspector::global();
        let stats = inspector.stats();

        println!("\n=== Inspection Results ===");
        println!("Total tasks: {}", stats.total_tasks);
        println!("Running tasks: {}", stats.running_tasks);
        println!("Completed tasks: {}", stats.completed_tasks);

        // Get all tasks and display them
        let tasks = inspector.get_all_tasks();
        println!("\n=== Task Details ===");
        for task in tasks.iter().take(15) {
            println!(
                "Task: {} | ID: {} | Parent: {} | State: {:?}",
                task.name,
                task.id.as_u64(),
                task.parent.map_or("None".to_string(), |p| p.as_u64().to_string()),
                task.state
            );
        }

        println!("\n✅ Example completed successfully!");
        println!("   {} tasks were tracked during execution", stats.total_tasks);
    });
}
