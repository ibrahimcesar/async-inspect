use async_inspect::inspector::Inspector;
use async_inspect::runtime::tokio::{spawn_tracked, InspectExt};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tokio::runtime::Runtime;

fn benchmark_task_creation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_creation");

    // Baseline: raw tokio::spawn
    group.bench_function("tokio_spawn_baseline", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let handle = tokio::spawn(async { black_box(42) });
            handle.await.unwrap()
        });
    });

    // With tracking
    group.bench_function("spawn_tracked", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let handle = spawn_tracked("bench_task", async { black_box(42) });
            handle.await.unwrap()
        });
    });

    group.finish();
}

fn benchmark_inspect_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("inspect_overhead");

    // Baseline: raw await
    group.bench_function("raw_await", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async { black_box(42) });
    });

    // With .inspect()
    group.bench_function("with_inspect", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt)
            .iter(|| async { async { black_box(42) }.inspect("bench_await").await });
    });

    group.finish();
}

fn benchmark_inspector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("inspector_ops");

    group.bench_function("get_task", |b| {
        let inspector = Inspector::global();
        let rt = Runtime::new().unwrap();

        // Spawn a task to get its ID
        let task_id = rt.block_on(async {
            let handle = spawn_tracked("test_task", async {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            });

            // Get the task ID from inspector
            let tasks = inspector.get_all_tasks();
            tasks.first().map(|t| t.id).unwrap()
        });

        b.iter(|| inspector.get_task(task_id));
    });

    group.bench_function("stats", |b| {
        let inspector = Inspector::global();
        b.iter(|| inspector.stats());
    });

    group.bench_function("get_all_tasks", |b| {
        let inspector = Inspector::global();
        b.iter(|| inspector.get_all_tasks());
    });

    group.finish();
}

fn benchmark_concurrent_tasks(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_tasks");

    for count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("spawn_tracked", count),
            count,
            |b, &count| {
                let rt = Runtime::new().unwrap();
                b.to_async(&rt).iter(|| async move {
                    let handles: Vec<_> = (0..count)
                        .map(|i| {
                            spawn_tracked(format!("task_{}", i), async move { black_box(i * 2) })
                        })
                        .collect();

                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.bench_function("task_info_size", |b| {
        use async_inspect::task::TaskInfo;
        use std::mem::size_of;

        b.iter(|| black_box(size_of::<TaskInfo>()));
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_task_creation_overhead,
    benchmark_inspect_overhead,
    benchmark_inspector_operations,
    benchmark_concurrent_tasks,
    benchmark_memory_footprint
);

criterion_main!(benches);
