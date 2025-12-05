//! Example demonstrating the web dashboard
//!
//! This example starts a web dashboard on localhost:8080 and simulates various async tasks.
//! Open http://localhost:8080 in your browser to view the real-time dashboard.
//!
//! Run with: cargo run --example dashboard_demo --features dashboard

use async_inspect::dashboard::Dashboard;
use async_inspect::runtime::tokio::{spawn_tracked, InspectExt};
use std::time::Duration;

/// Simulates fetching data from an API
async fn fetch_api_data(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("API data for ID: {}", id)
}

/// Simulates processing data
async fn process_data(data: String, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("Processed: {}", data)
}

/// Simulates writing to database
async fn write_to_db(data: String, delay_ms: u64) -> bool {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    println!("✅ Written to DB: {}", data);
    true
}

/// Worker task that processes multiple requests
async fn worker_task(worker_id: u32, num_requests: u32) {
    for request_id in 1..=num_requests {
        let global_id = worker_id * 1000 + request_id;

        // Fetch data
        let data = fetch_api_data(global_id, (50 + (request_id % 3) * 50) as u64)
            .inspect(format!("fetch_api_{}", global_id))
            .await;

        // Process data
        let processed = process_data(data, (30 + (request_id % 2) * 30) as u64)
            .inspect(format!("process_{}", global_id))
            .await;

        // Write to database
        write_to_db(processed, (20 + (request_id % 4) * 20) as u64)
            .inspect(format!("write_db_{}", global_id))
            .await;

        println!(
            "Worker {} completed request {}/{}",
            worker_id, request_id, num_requests
        );
    }

    println!("✅ Worker {} finished!", worker_id);
}

/// Long-running background task
async fn background_monitoring() {
    let mut counter = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(2)).await;
        counter += 1;
        println!("📊 Background monitor tick: {}", counter);

        if counter >= 30 {
            break;
        }
    }
}

/// Periodic cleanup task
async fn cleanup_task() {
    let mut iteration = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;
        iteration += 1;
        println!("🧹 Cleanup iteration: {}", iteration);

        if iteration >= 20 {
            break;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 async-inspect Dashboard Demo");
    println!("================================\n");

    // Start the dashboard on port 8080
    println!("Starting dashboard on http://localhost:8080");
    let dashboard = Dashboard::new(8080);
    let dashboard_handle = dashboard.start().await?;

    println!("✅ Dashboard is running!");
    println!("📱 Open http://localhost:8080 in your browser\n");
    println!("The demo will run for about 60 seconds, spawning various async tasks.");
    println!("Watch the dashboard to see real-time task monitoring!\n");

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Spawn background tasks
    spawn_tracked("background_monitoring", background_monitoring());
    spawn_tracked("cleanup_task", cleanup_task());

    // Spawn multiple workers
    let mut worker_handles = vec![];

    println!("Spawning 5 worker tasks...");
    for worker_id in 1..=5 {
        let handle = spawn_tracked(
            format!("worker_{}", worker_id),
            worker_task(worker_id, 8), // Each worker processes 8 requests
        );
        worker_handles.push(handle);
    }

    // Wait a bit then spawn more workers
    tokio::time::sleep(Duration::from_secs(10)).await;

    println!("\nSpawning 3 more workers...");
    for worker_id in 6..=8 {
        let handle = spawn_tracked(
            format!("worker_{}", worker_id),
            worker_task(worker_id, 6), // These process 6 requests
        );
        worker_handles.push(handle);
    }

    // Wait for all workers to complete
    for handle in worker_handles {
        handle.await?;
    }

    println!("\n✅ All workers completed!");
    println!("\nDashboard will continue running for 10 more seconds...");
    println!("You can still explore the completed tasks in the browser.\n");

    tokio::time::sleep(Duration::from_secs(10)).await;

    println!("Demo complete! Shutting down...");

    // The dashboard will shut down when dropped
    drop(dashboard_handle);

    Ok(())
}
