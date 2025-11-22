//! Tokio runtime integration example
//!
//! This example demonstrates automatic task tracking when using
//! the Tokio integration features.

use async_inspect::prelude::*;
use async_inspect::runtime::tokio::{spawn_tracked, InspectExt};
use std::time::Duration;

/// Example async function - no manual instrumentation needed!
async fn fetch_data(id: u64) -> String {
    tokio::time::sleep(Duration::from_millis(50)).await;
    format!("Data for ID {}", id)
}

/// Another example function
async fn process_data(data: String) -> String {
    tokio::time::sleep(Duration::from_millis(30)).await;
    data.to_uppercase()
}

/// Complex async operation
async fn complex_operation(id: u64) -> String {
    // Using .inspect() extension method - automatic tracking!
    let data = fetch_data(id).inspect(format!("fetch_data({})", id)).await;

    // Another tracked operation
    let processed = process_data(data)
        .inspect(format!("process_data({})", id))
        .await;

    processed
}

#[tokio::main]
async fn main() {
    println!("🔍 async-inspect - Tokio Integration Example");
    println!("==============================================\n");

    // Reset the inspector
    Inspector::global().reset();

    println!("=== Example 1: spawn_tracked() ===\n");

    // Spawn tasks with automatic tracking using spawn_tracked
    let handles: Vec<_> = (1..=3)
        .map(|i| {
            spawn_tracked(format!("background_task_{}", i), async move {
                tokio::time::sleep(Duration::from_millis(i * 20)).await;
                println!("Task {} completed", i);
                i * 100
            })
        })
        .collect();

    // Wait for all spawned tasks
    for handle in handles {
        let result = handle.await.unwrap();
        println!("Got result: {}", result);
    }

    println!("\n=== Example 2: .inspect() extension ===\n");

    // Use .inspect() on any future
    let result1 = fetch_data(10).inspect("fetch_data(10)").await;
    println!("Result 1: {}", result1);

    let result2 = process_data(result1).inspect("process_data").await;
    println!("Result 2: {}", result2);

    println!("\n=== Example 3: Complex operations ===\n");

    // Run complex operations that use .inspect() internally
    let ops: Vec<_> = (20..=22).map(|i| complex_operation(i)).collect();

    for op in ops {
        let result = op.await;
        println!("Complex result: {}", result);
    }

    println!("\n=== Example 4: Parallel spawned tasks ===\n");

    // Spawn multiple tasks in parallel
    let parallel_tasks: Vec<_> = (30..=34)
        .map(|i| {
            spawn_tracked(format!("parallel_{}", i), async move {
                complex_operation(i).await
            })
        })
        .collect();

    // Collect results
    let mut results = Vec::new();
    for task in parallel_tasks {
        results.push(task.await.unwrap());
    }

    println!("Parallel results: {:?}", results);

    // Give everything time to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\n=== Inspection Results ===\n");

    // Generate reports
    let reporter = Reporter::global();

    // Summary
    reporter.print_summary();

    println!();

    // Compact summary
    reporter.print_compact_summary();

    println!("\n=== Task Details ===\n");

    // Show details for a few tasks
    let tasks = Inspector::global().get_all_tasks();
    for task in tasks.iter().take(5) {
        println!("  {} - {} ({})", task.id, task.name, task.state);
    }

    println!("\n=== Timeline (first 20 events) ===\n");

    let events = Inspector::global().get_events();
    for event in events.iter().take(20) {
        println!("  {}", event);
    }

    if events.len() > 20 {
        println!("  ... and {} more events", events.len() - 20);
    }
}
