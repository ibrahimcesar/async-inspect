//! Deadlock detection example
//!
//! This example demonstrates how async-inspect can detect deadlocks
//! caused by circular dependencies between tasks and resources.

use async_inspect::deadlock::{DeadlockDetector, ResourceInfo, ResourceKind};
use async_inspect::task::TaskId;
use colored::Colorize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Simulated deadlock scenario: Classic dining philosophers problem (simplified)
async fn deadlock_scenario_1() {
    println!(
        "{}",
        "=== Scenario 1: Classic Mutex Deadlock ==="
            .bright_yellow()
            .bold()
    );
    println!();

    let detector = DeadlockDetector::new();

    // Create two shared resources (mutexes)
    let mutex_a = Arc::new(Mutex::new(0));
    let mutex_b = Arc::new(Mutex::new(0));

    // Register resources
    let res_a = ResourceInfo::new(ResourceKind::Mutex, "mutex_a".to_string())
        .with_address(Arc::as_ptr(&mutex_a) as usize);
    let res_b = ResourceInfo::new(ResourceKind::Mutex, "mutex_b".to_string())
        .with_address(Arc::as_ptr(&mutex_b) as usize);

    let res_a_id = res_a.id;
    let res_b_id = res_b.id;

    let _ = detector.register_resource(res_a);
    let _ = detector.register_resource(res_b);

    // Spawn task 1: locks A then tries to lock B
    let task1_id = TaskId::new();
    let detector_clone1 = detector.clone();
    let mutex_a_clone1 = Arc::clone(&mutex_a);
    let mutex_b_clone1 = Arc::clone(&mutex_b);

    let task1 = tokio::spawn(async move {
        println!("Task 1: Acquiring mutex_a...");
        let _guard_a = mutex_a_clone1.lock().await;
        detector_clone1.acquire(task1_id, res_a_id);
        println!("Task 1: Acquired mutex_a");

        // Small delay to ensure both tasks acquire their first lock
        tokio::time::sleep(Duration::from_millis(50)).await;

        println!("Task 1: Waiting for mutex_b...");
        detector_clone1.wait_for(task1_id, res_b_id);

        // This will block - Task 2 holds mutex_b
        let _guard_b = mutex_b_clone1.lock().await;
        println!("Task 1: Acquired mutex_b (should never print)");
    });

    // Spawn task 2: locks B then tries to lock A
    let task2_id = TaskId::new();
    let detector_clone2 = detector.clone();
    let mutex_a_clone2 = Arc::clone(&mutex_a);
    let mutex_b_clone2 = Arc::clone(&mutex_b);

    let task2 = tokio::spawn(async move {
        println!("Task 2: Acquiring mutex_b...");
        let _guard_b = mutex_b_clone2.lock().await;
        detector_clone2.acquire(task2_id, res_b_id);
        println!("Task 2: Acquired mutex_b");

        // Small delay
        tokio::time::sleep(Duration::from_millis(50)).await;

        println!("Task 2: Waiting for mutex_a...");
        detector_clone2.wait_for(task2_id, res_a_id);

        // This will block - Task 1 holds mutex_a
        let _guard_a = mutex_a_clone2.lock().await;
        println!("Task 2: Acquired mutex_a (should never print)");
    });

    // Give tasks time to deadlock
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Detect deadlocks
    println!("\n{}\n", "[*] Checking for deadlocks...".bright_cyan());
    let deadlocks = detector.detect_deadlocks();

    if deadlocks.is_empty() {
        println!("{}", "[OK] No deadlocks detected".on_green().white().bold());
    } else {
        for (i, cycle) in deadlocks.iter().enumerate() {
            println!(
                "{}",
                format!("[FAIL] Deadlock #{} detected!", i + 1)
                    .on_red()
                    .white()
                    .bold()
            );
            println!("{}", cycle.describe());
            println!();

            // Show involved resources
            println!("Resources involved:");
            for &res_id in &cycle.resources {
                if let Some(resource) = detector.get_resource(res_id) {
                    println!("  - {}", resource);
                }
            }
            println!();
        }

        println!("{}", "[*] Suggestions:".on_yellow().white().bold());
        println!(
            "  {} {}",
            "•".bright_yellow(),
            "Acquire locks in consistent order (always A before B)".bright_white()
        );
        println!(
            "  {} {}",
            "•".bright_yellow(),
            "Use try_lock() with timeout".bright_white()
        );
        println!(
            "  {} {}",
            "•".bright_yellow(),
            "Consider lock-free data structures".bright_white()
        );
    }

    // Abort tasks (they're deadlocked)
    task1.abort();
    task2.abort();

    println!();
}

/// Scenario 2: No deadlock (proper lock ordering)
async fn no_deadlock_scenario() {
    println!(
        "{}",
        "=== Scenario 2: Proper Lock Ordering (No Deadlock) ==="
            .bright_yellow()
            .bold()
    );
    println!();

    let detector = DeadlockDetector::new();

    let mutex_a = Arc::new(Mutex::new(0));
    let mutex_b = Arc::new(Mutex::new(0));

    let res_a = ResourceInfo::new(ResourceKind::Mutex, "mutex_a".to_string());
    let res_b = ResourceInfo::new(ResourceKind::Mutex, "mutex_b".to_string());

    let res_a_id = res_a.id;
    let res_b_id = res_b.id;

    let _ = detector.register_resource(res_a);
    let _ = detector.register_resource(res_b);

    // Both tasks acquire locks in the SAME order: A then B
    let tasks: Vec<_> = (1..=3)
        .map(|i| {
            let detector_clone = detector.clone();
            let mutex_a_clone = Arc::clone(&mutex_a);
            let mutex_b_clone = Arc::clone(&mutex_b);

            tokio::spawn(async move {
                let task_id = TaskId::new();

                println!("Task {}: Acquiring mutex_a...", i);
                let _guard_a = mutex_a_clone.lock().await;
                detector_clone.acquire(task_id, res_a_id);
                println!("Task {}: Acquired mutex_a", i);

                tokio::time::sleep(Duration::from_millis(10)).await;

                println!("Task {}: Acquiring mutex_b...", i);
                let _guard_b = mutex_b_clone.lock().await;
                detector_clone.acquire(task_id, res_b_id);
                println!("Task {}: Acquired mutex_b", i);

                tokio::time::sleep(Duration::from_millis(10)).await;

                // Release (automatically via Drop)
                detector_clone.release(task_id, res_b_id);
                detector_clone.release(task_id, res_a_id);

                println!("Task {}: Released all locks", i);
            })
        })
        .collect();

    // Wait for completion
    for task in tasks {
        let _ = task.await;
    }

    // Check for deadlocks
    println!("\n{}\n", "[*] Checking for deadlocks...".bright_cyan());
    let deadlocks = detector.detect_deadlocks();

    if deadlocks.is_empty() {
        println!(
            "{}",
            "[OK] No deadlocks detected - all tasks completed successfully!"
                .on_green()
                .white()
                .bold()
        );
    } else {
        println!(
            "{}",
            "[FAIL] Unexpected deadlock detected!"
                .on_red()
                .white()
                .bold()
        );
    }

    println!();
}

#[tokio::main]
async fn main() {
    println!("{}", "[async-inspect]".on_purple().white().bold());
    println!("{}", "Deadlock Detection Example".bright_blue());
    println!("===============================================\n");

    // Scenario 1: Demonstrate deadlock detection
    deadlock_scenario_1().await;

    // Scenario 2: Show proper lock ordering
    no_deadlock_scenario().await;

    println!("{}", "=== Summary ===".bright_yellow().bold());
    println!();
    println!(
        "{} {}",
        "[async-inspect]".on_purple().white().bold(),
        "successfully:".bright_white().bold()
    );
    println!(
        "  {} {}",
        "[OK]".on_green().white().bold(),
        "Detected circular dependencies".bright_white()
    );
    println!(
        "  {} {}",
        "[OK]".on_green().white().bold(),
        "Identified involved tasks and resources".bright_white()
    );
    println!(
        "  {} {}",
        "[OK]".on_green().white().bold(),
        "Generated actionable suggestions".bright_white()
    );
    println!();
    println!(
        "{}",
        "Deadlock detection helps you find and fix".bright_cyan()
    );
    println!(
        "{}",
        "circular wait conditions before they reach production!".bright_cyan()
    );
}
