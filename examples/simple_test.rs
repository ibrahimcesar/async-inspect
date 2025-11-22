//! Simple test example without spawning

use async_inspect::prelude::*;
use std::time::Duration;

async fn simple_task(id: u64) {
    let _guard = TaskGuard::new(format!("simple_task({})", id));

    inspect_point!("start");

    tokio::time::sleep(Duration::from_millis(50)).await;

    inspect_point!("middle");

    tokio::time::sleep(Duration::from_millis(50)).await;

    inspect_point!("end");
}

#[tokio::main]
async fn main() {
    println!("🔍 async-inspect - Simple Test\n");

    // Reset the inspector
    Inspector::global().reset();

    // Run a few tasks sequentially
    simple_task(1).await;
    simple_task(2).await;
    simple_task(3).await;

    println!("\n=== Results ===\n");

    // Create a reporter and print results
    let reporter = Reporter::global();

    // Print summary
    reporter.print_summary();

    println!();

    //Print timeline
    reporter.print_timeline();

    println!();

    // Print compact summary
    reporter.print_compact_summary();
}
