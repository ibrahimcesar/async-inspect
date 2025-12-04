//! Automatic Lock Tracking Example
//!
//! Demonstrates how async-inspect's tracked synchronization primitives
//! automatically record contention metrics without any manual instrumentation.

use async_inspect::sync::{Mutex, RwLock, Semaphore};
use colored::Colorize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!(
        "\n{}\n",
        " [async-inspect] Automatic Lock Tracking Demo "
            .white()
            .bold()
            .on_purple()
    );

    // Demo 1: Mutex contention tracking
    demo_mutex().await;

    // Demo 2: RwLock read/write tracking
    demo_rwlock().await;

    // Demo 3: Semaphore permit tracking
    demo_semaphore().await;

    println!(
        "\n{}\n",
        " [async-inspect] Demo Complete! "
            .white()
            .bold()
            .on_purple()
    );
}

async fn demo_mutex() {
    println!("\n{} Mutex Contention Tracking\n", "[*]".cyan().bold());

    let mutex = Arc::new(Mutex::new(0u64, "counter"));
    let mut handles = vec![];

    // Spawn 10 tasks that will contend for the mutex
    for i in 0..10 {
        let m = mutex.clone();
        handles.push(tokio::spawn(async move {
            let mut guard = m.lock().await;
            // Simulate some work while holding the lock
            sleep(Duration::from_millis(5)).await;
            *guard += 1;
            println!("    Task {} incremented counter to {}", i, *guard);
        }));
    }

    // Wait for all tasks
    for h in handles {
        h.await.unwrap();
    }

    // Check contention metrics - no manual instrumentation needed!
    let metrics = mutex.metrics();
    println!("\n{} Mutex Metrics:", "[STATS]".green().bold());
    println!("    Acquisitions:    {}", metrics.acquisitions);
    println!("    Contentions:     {}", metrics.contentions);
    println!(
        "    Contention rate: {:.1}%",
        metrics.contention_rate() * 100.0
    );
    println!("    Max wait time:   {:?}", metrics.max_wait_time);
    println!("    Avg wait time:   {:?}", metrics.avg_wait_time);

    let final_value = mutex.lock().await;
    println!(
        "\n{} Final counter value: {}",
        "[OK]".green().bold(),
        *final_value
    );
}

async fn demo_rwlock() {
    println!("\n{} RwLock Read/Write Tracking\n", "[*]".cyan().bold());

    let lock = Arc::new(RwLock::new(vec!["initial"], "shared_data"));
    let mut handles = vec![];

    // Spawn multiple readers
    for i in 0..5 {
        let l = lock.clone();
        handles.push(tokio::spawn(async move {
            let guard = l.read().await;
            println!("    Reader {} sees {} items", i, guard.len());
            sleep(Duration::from_millis(2)).await;
        }));
    }

    // Spawn a writer
    {
        let l = lock.clone();
        handles.push(tokio::spawn(async move {
            // Wait briefly to ensure readers start first
            sleep(Duration::from_millis(1)).await;
            let mut guard = l.write().await;
            guard.push("added by writer");
            println!("    Writer added item");
        }));
    }

    // Spawn more readers after writer
    for i in 5..8 {
        let l = lock.clone();
        handles.push(tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            let guard = l.read().await;
            println!("    Reader {} sees {} items", i, guard.len());
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // Get separate read and write metrics
    let (read_metrics, write_metrics) = lock.metrics();
    println!("\n{} RwLock Metrics:", "[STATS]".green().bold());
    println!("  Read operations:");
    println!("    Acquisitions:    {}", read_metrics.acquisitions);
    println!("    Contentions:     {}", read_metrics.contentions);
    println!(
        "    Contention rate: {:.1}%",
        read_metrics.contention_rate() * 100.0
    );
    println!("  Write operations:");
    println!("    Acquisitions:    {}", write_metrics.acquisitions);
    println!("    Contentions:     {}", write_metrics.contentions);
    println!(
        "    Contention rate: {:.1}%",
        write_metrics.contention_rate() * 100.0
    );
}

async fn demo_semaphore() {
    println!("\n{} Semaphore Permit Tracking\n", "[*]".cyan().bold());

    // Create a semaphore that limits concurrent operations to 3
    let semaphore = Arc::new(Semaphore::new(3, "connection_pool"));
    let mut handles = vec![];

    println!(
        "    Semaphore allows {} concurrent operations",
        semaphore.initial_permits()
    );

    // Spawn 10 tasks that will contend for 3 permits
    for i in 0..10 {
        let sem = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            println!(
                "    Task {} acquired permit ({} available)",
                i,
                sem.available_permits()
            );
            sleep(Duration::from_millis(10)).await;
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let metrics = semaphore.metrics();
    println!("\n{} Semaphore Metrics:", "[STATS]".green().bold());
    println!("    Acquisitions:    {}", metrics.acquisitions);
    println!("    Contentions:     {}", metrics.contentions);
    println!(
        "    Contention rate: {:.1}%",
        metrics.contention_rate() * 100.0
    );
    println!("    Max wait time:   {:?}", metrics.max_wait_time);
    println!("    Avg wait time:   {:?}", metrics.avg_wait_time);
    println!(
        "\n{} All {} permits returned",
        "[OK]".green().bold(),
        semaphore.available_permits()
    );
}
