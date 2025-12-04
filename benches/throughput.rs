use async_inspect::runtime::tokio::{spawn_tracked, InspectExt};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;
use tokio::runtime::Runtime;

fn benchmark_event_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_throughput");
    group.measurement_time(Duration::from_secs(10));

    group.throughput(Throughput::Elements(1000));
    group.bench_function("1000_tasks", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let handles: Vec<_> = (0..1000)
                .map(|i| spawn_tracked(format!("throughput_{}", i), async move { black_box(i) }))
                .collect();

            for handle in handles {
                handle.await.unwrap();
            }
        });
    });

    group.throughput(Throughput::Elements(10000));
    group.bench_function("10000_tasks", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let handles: Vec<_> = (0..10000)
                .map(|i| spawn_tracked(format!("throughput_{}", i), async move { black_box(i) }))
                .collect();

            for handle in handles {
                handle.await.unwrap();
            }
        });
    });

    group.finish();
}

fn benchmark_await_point_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("await_throughput");
    group.measurement_time(Duration::from_secs(10));

    group.throughput(Throughput::Elements(1000));
    group.bench_function("1000_awaits", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            for i in 0..1000 {
                async { black_box(i) }
                    .inspect(&format!("await_{}", i))
                    .await;
            }
        });
    });

    group.finish();
}

fn benchmark_realistic_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workload");
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("web_server_simulation", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            // Simulate 100 concurrent requests
            let handles: Vec<_> = (0..100)
                .map(|req_id| {
                    spawn_tracked(format!("request_{}", req_id), async move {
                        // Simulate database query
                        tokio::time::sleep(Duration::from_micros(100))
                            .inspect("db_query")
                            .await;

                        // Simulate business logic
                        black_box(req_id * 2);

                        // Simulate cache write
                        tokio::time::sleep(Duration::from_micros(50))
                            .inspect("cache_write")
                            .await;

                        black_box(req_id)
                    })
                })
                .collect();

            for handle in handles {
                handle.await.unwrap();
            }
        });
    });

    group.finish();
}

fn benchmark_background_workers(c: &mut Criterion) {
    let mut group = c.benchmark_group("background_workers");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("worker_pool_10", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);

            // Spawn 10 workers
            let workers: Vec<_> = (0..10)
                .map(|worker_id| {
                    let mut rx = rx.clone();
                    spawn_tracked(format!("worker_{}", worker_id), async move {
                        while let Some(job) = rx.recv().await {
                            tokio::time::sleep(Duration::from_micros(10))
                                .inspect("process_job")
                                .await;
                            black_box(job);
                        }
                    })
                })
                .collect();

            // Send 1000 jobs
            for i in 0..1000 {
                tx.send(i).await.unwrap();
            }

            drop(tx);
            drop(rx);

            for worker in workers {
                worker.await.unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_event_throughput,
    benchmark_await_point_throughput,
    benchmark_realistic_workload,
    benchmark_background_workers
);

criterion_main!(benches);
