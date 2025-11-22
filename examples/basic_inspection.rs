//! Basic async inspection example
//!
//! This example demonstrates how to use async-inspect to track and debug
//! async operations.

use async_inspect::prelude::*;
use std::time::Duration;

/// Simulated user data
#[derive(Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

/// Simulated async function that fetches user profile
async fn fetch_profile(user_id: u64) -> User {
    let _guard = TaskGuard::new(format!("fetch_profile({})", user_id));

    inspect_point!("starting_profile_fetch");

    // Simulate network delay
    tokio::time::sleep(Duration::from_millis(100)).await;

    inspect_point!("profile_fetched");

    User {
        id: user_id,
        name: format!("User {}", user_id),
        email: format!("user{}@example.com", user_id),
    }
}

/// Simulated async function that fetches user posts
async fn fetch_posts(user_id: u64) -> Vec<String> {
    let _guard = TaskGuard::new(format!("fetch_posts({})", user_id));

    inspect_point!("starting_posts_fetch");

    // Simulate longer network delay
    tokio::time::sleep(Duration::from_millis(150)).await;

    inspect_point!("posts_fetched", format!("Got {} posts", 3));

    vec![
        format!("Post 1 by user {}", user_id),
        format!("Post 2 by user {}", user_id),
        format!("Post 3 by user {}", user_id),
    ]
}

/// Main async function that coordinates fetching user data
async fn fetch_user_data(user_id: u64) {
    let _guard = TaskGuard::new(format!("fetch_user_data({})", user_id));

    inspect_point!("start");

    // Fetch profile
    let user = fetch_profile(user_id).await;
    inspect_point!("profile_complete", format!("Got user: {}", user.name));

    // Fetch posts
    let posts = fetch_posts(user_id).await;
    inspect_point!("posts_complete", format!("Got {} posts", posts.len()));

    println!("\nUser: {} ({})", user.name, user.email);
    println!("Posts:");
    for (i, post) in posts.iter().enumerate() {
        println!("  {}. {}", i + 1, post);
    }

    inspect_point!("done");
}

/// Example with parallel tasks
async fn parallel_example() {
    println!("\n=== Parallel Tasks Example ===\n");

    // Spawn multiple tasks in parallel
    let tasks = vec![
        tokio::spawn(fetch_user_data(1)),
        tokio::spawn(fetch_user_data(2)),
        tokio::spawn(fetch_user_data(3)),
    ];

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}

/// Example with sequential tasks
async fn sequential_example() {
    println!("\n=== Sequential Tasks Example ===\n");

    fetch_user_data(10).await;
    fetch_user_data(20).await;
    fetch_user_data(30).await;
}

#[tokio::main]
async fn main() {
    println!("🔍 async-inspect - Basic Inspection Example");
    println!("===========================================\n");

    // Reset the inspector
    Inspector::global().reset();

    // Run sequential example
    sequential_example().await;

    // Small delay to separate examples
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Run parallel example
    parallel_example().await;

    // Give tasks a moment to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\n=== Inspection Results ===\n");

    // Create a reporter and print results
    let reporter = Reporter::global();

    // Print summary
    reporter.print_summary();

    println!();

    // Print timeline
    reporter.print_timeline();

    println!();

    // Print compact summary
    reporter.print_compact_summary();

    println!();

    // Generate text report
    let report = reporter.generate_report();
    println!("\n=== Text Report ===\n");
    println!("{}", report);
}
