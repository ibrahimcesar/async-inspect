//! Example demonstrating advanced performance profiling features
//!
//! This example shows:
//! - Lock contention metrics
//! - Performance snapshots and comparison
//! - Regression detection
//!
//! Run with: cargo run --example performance_profiling --features tokio

use async_inspect::deadlock::{DeadlockDetector, ResourceId, ResourceInfo, ResourceKind};
use async_inspect::profile::{
    comparison::PerformanceComparison, LockContentionMetrics, Profiler, TaskMetrics,
};
use async_inspect::task::TaskId;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

async fn simulate_contended_lock(
    lock: Arc<Mutex<u64>>,
    profiler: Arc<tokio::sync::Mutex<Profiler>>,
    resource_id: ResourceId,
    task_id: usize,
) {
    for i in 0..5 {
        let start = std::time::Instant::now();

        // Simulate waiting for lock
        let _guard = lock.lock().await;
        let wait_time = start.elapsed();

        // Record lock metrics
        {
            let mut prof = profiler.lock().await;
            prof.record_lock_wait(
                resource_id,
                "shared_counter".to_string(),
                wait_time,
                TaskId::new(),
            );
            prof.record_lock_acquisition(resource_id, "shared_counter".to_string());
        }

        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;

        println!("Task {} iteration {}: waited {:?}", task_id, i, wait_time);
    }
}

async fn run_workload(profiler: Arc<tokio::sync::Mutex<Profiler>>) {
    let shared_lock = Arc::new(Mutex::new(0u64));
    let resource_id = ResourceId::new();

    // Spawn multiple tasks that contend for the same lock
    let mut tasks = vec![];

    for i in 0..5 {
        let lock = Arc::clone(&shared_lock);
        let prof = Arc::clone(&profiler);
        let task = tokio::spawn(async move {
            simulate_contended_lock(lock, prof, resource_id, i).await;
        });
        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        task.await.unwrap();
    }

    // Simulate some task metrics
    {
        let mut prof = profiler.lock().await;
        for i in 0..10 {
            let mut metrics = TaskMetrics::new(TaskId::new(), format!("task_{}", i));
            metrics.total_duration = Duration::from_millis(100 + i * 10);
            metrics.running_time = Duration::from_millis(80 + i * 8);
            metrics.blocked_time = Duration::from_millis(20 + i * 2);
            metrics.poll_count = 5 + i;
            metrics.completed = true;
            prof.record_task(metrics);
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Performance Profiling Example");
    println!("============================\n");

    // Create profiler
    let profiler = Arc::new(tokio::sync::Mutex::new(Profiler::new()));

    println!("Running baseline workload...");
    run_workload(Arc::clone(&profiler)).await;

    // Create baseline snapshot
    let baseline_snapshot = {
        let prof = profiler.lock().await;
        prof.create_snapshot("baseline".to_string())
    };

    println!("\nBaseline snapshot created");
    println!("  Total tasks: {}", baseline_snapshot.total_tasks);
    println!(
        "  Avg task duration: {:?}",
        baseline_snapshot.avg_task_duration
    );
    println!("  Lock metrics: {}", baseline_snapshot.lock_metrics.len());

    // Save baseline snapshot
    baseline_snapshot
        .save_to_file("baseline_snapshot.json")
        .unwrap();
    println!("  Saved to baseline_snapshot.json");

    // Print lock contention metrics
    println!("\n=== Lock Contention Metrics ===");
    {
        let prof = profiler.lock().await;
        for lock_metric in prof.all_lock_metrics() {
            println!("\nLock: {}", lock_metric.name);
            println!("  Acquisitions: {}", lock_metric.successful_acquisitions);
            println!("  Contentions: {}", lock_metric.contention_count);
            println!("  Contention rate: {:.1}%", lock_metric.contention_rate * 100.0);
            println!("  Avg wait time: {:?}", lock_metric.avg_wait_time);
            println!("  Max wait time: {:?}", lock_metric.max_wait_time);
            println!("  Waiting tasks: {}", lock_metric.waiting_tasks.len());
        }

        // Identify highly contended locks
        let highly_contended = prof.identify_highly_contended_locks();
        if !highly_contended.is_empty() {
            println!("\n⚠️  Highly Contended Locks ({}):", highly_contended.len());
            for lock in highly_contended {
                println!(
                    "  - {} ({:.1}% contention rate)",
                    lock.name,
                    lock.contention_rate * 100.0
                );
            }
        }
    }

    println!("\n=== Task Performance ===");
    {
        let prof = profiler.lock().await;

        // Show bottlenecks
        let bottlenecks = prof.identify_bottlenecks();
        if !bottlenecks.is_empty() {
            println!("\nBottleneck tasks:");
            for task in bottlenecks {
                println!(
                    "  - {} ({:?}, efficiency: {:.1}%)",
                    task.name,
                    task.total_duration,
                    task.efficiency() * 100.0
                );
            }
        }

        // Show slowest tasks
        println!("\nSlowest tasks:");
        for task in prof.slowest_tasks(3) {
            println!("  - {} ({:?})", task.name, task.total_duration);
        }

        // Show busiest tasks
        println!("\nBusiest tasks (most polls):");
        for task in prof.busiest_tasks(3) {
            println!("  - {} ({} polls)", task.name, task.poll_count);
        }
    }

    // Simulate a second run with slightly different performance
    println!("\n\nRunning modified workload (simulating performance change)...");
    let profiler2 = Arc::new(tokio::sync::Mutex::new(Profiler::new()));
    run_workload(Arc::clone(&profiler2)).await;

    // Add slightly degraded performance
    {
        let mut prof = profiler2.lock().await;
        for i in 0..10 {
            let mut metrics = TaskMetrics::new(TaskId::new(), format!("task_{}", i));
            // 15% slower
            metrics.total_duration = Duration::from_millis((100 + i * 10) * 115 / 100);
            metrics.running_time = Duration::from_millis((80 + i * 8) * 115 / 100);
            metrics.blocked_time = Duration::from_millis((20 + i * 2) * 115 / 100);
            metrics.poll_count = 5 + i;
            metrics.completed = true;
            prof.record_task(metrics);
        }
    }

    let current_snapshot = {
        let prof = profiler2.lock().await;
        prof.create_snapshot("current".to_string())
    };

    // Compare snapshots
    println!("\n=== Performance Comparison ===");
    let comparison = PerformanceComparison::compare(baseline_snapshot, current_snapshot);

    println!("{}", comparison.summary());

    // Check for regressions
    if comparison.has_regressions() {
        println!("\n❌ REGRESSIONS DETECTED!");
        println!("\nRegression details:");
        for finding in comparison.get_regressions() {
            println!("  [{:?}] {}", finding.severity, finding.description);
            println!("      Impact: {:.1}%", finding.impact_percent);
        }
    } else {
        println!("\n✅ No regressions detected");
    }

    // Show improvements
    let improvements = comparison.get_improvements();
    if !improvements.is_empty() {
        println!("\n✨ Improvements:");
        for finding in improvements {
            println!("  - {}", finding.description);
            println!("    Impact: +{:.1}%", finding.impact_percent);
        }
    }

    println!("\n=== Example Complete ===");
    println!("Demonstrated features:");
    println!("  ✅ Lock contention tracking");
    println!("  ✅ Performance snapshots");
    println!("  ✅ Snapshot serialization");
    println!("  ✅ Performance comparison");
    println!("  ✅ Regression detection");
    println!("  ✅ Bottleneck identification");
}
