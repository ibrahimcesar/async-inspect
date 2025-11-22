//! Relationship Graph Example
//!
//! This example demonstrates the enhanced relationship graph functionality,
//! showing different types of task interactions and dependencies.
//!
//! Run with: cargo run --example relationship_graph

use async_inspect::graph::*;
use async_inspect::task::{TaskId, TaskInfo, TaskState};
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  async-inspect - Relationship Graph Example               ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Create a sample task graph
    let mut graph = TaskGraph::new();

    // Create some sample tasks
    let coordinator = create_task(1, "coordinator", TaskState::Completed);
    let worker1 = create_task(2, "worker_1", TaskState::Completed);
    let worker2 = create_task(3, "worker_2", TaskState::Running);
    let worker3 = create_task(
        4,
        "worker_3",
        TaskState::Blocked {
            await_point: "waiting_for_lock".to_string(),
        },
    );
    let aggregator = create_task(5, "aggregator", TaskState::Running);
    let db_pool = create_task(6, "db_connection_pool", TaskState::Running);
    let cache = create_task(7, "cache_manager", TaskState::Running);

    graph.add_task(coordinator.clone());
    graph.add_task(worker1.clone());
    graph.add_task(worker2.clone());
    graph.add_task(worker3.clone());
    graph.add_task(aggregator.clone());
    graph.add_task(db_pool.clone());
    graph.add_task(cache.clone());

    // Add spawn relationships
    graph.add_relationship(Relationship {
        from: coordinator.id,
        to: worker1.id,
        relationship_type: RelationshipType::Spawned,
        resource_name: None,
        data_description: None,
    });

    graph.add_relationship(Relationship {
        from: coordinator.id,
        to: worker2.id,
        relationship_type: RelationshipType::Spawned,
        resource_name: None,
        data_description: None,
    });

    graph.add_relationship(Relationship {
        from: coordinator.id,
        to: worker3.id,
        relationship_type: RelationshipType::Spawned,
        resource_name: None,
        data_description: None,
    });

    // Add channel communication
    graph.add_relationship(Relationship {
        from: worker1.id,
        to: aggregator.id,
        relationship_type: RelationshipType::ChannelSend,
        resource_name: Some("results_channel".to_string()),
        data_description: Some("QueryResult".to_string()),
    });

    graph.add_relationship(Relationship {
        from: aggregator.id,
        to: worker1.id,
        relationship_type: RelationshipType::ChannelReceive,
        resource_name: Some("results_channel".to_string()),
        data_description: Some("QueryResult".to_string()),
    });

    graph.add_relationship(Relationship {
        from: worker2.id,
        to: aggregator.id,
        relationship_type: RelationshipType::ChannelSend,
        resource_name: Some("results_channel".to_string()),
        data_description: Some("QueryResult".to_string()),
    });

    // Add shared resource access
    graph.add_relationship(Relationship {
        from: worker1.id,
        to: db_pool.id,
        relationship_type: RelationshipType::SharedResource,
        resource_name: Some("connection_mutex".to_string()),
        data_description: None,
    });

    graph.add_relationship(Relationship {
        from: worker2.id,
        to: db_pool.id,
        relationship_type: RelationshipType::SharedResource,
        resource_name: Some("connection_mutex".to_string()),
        data_description: None,
    });

    graph.add_relationship(Relationship {
        from: worker3.id,
        to: db_pool.id,
        relationship_type: RelationshipType::SharedResource,
        resource_name: Some("connection_mutex".to_string()),
        data_description: None,
    });

    // Add data flow
    graph.add_relationship(Relationship {
        from: worker1.id,
        to: cache.id,
        relationship_type: RelationshipType::DataFlow,
        resource_name: None,
        data_description: Some("CacheEntry { key, value }".to_string()),
    });

    // Add dependencies
    graph.add_relationship(Relationship {
        from: aggregator.id,
        to: worker1.id,
        relationship_type: RelationshipType::Dependency,
        resource_name: None,
        data_description: None,
    });

    graph.add_relationship(Relationship {
        from: aggregator.id,
        to: worker2.id,
        relationship_type: RelationshipType::Dependency,
        resource_name: None,
        data_description: None,
    });

    // Add await relationship
    graph.add_relationship(Relationship {
        from: coordinator.id,
        to: aggregator.id,
        relationship_type: RelationshipType::AwaitsOn,
        resource_name: None,
        data_description: None,
    });

    // Display analysis
    println!("📊 Task Graph Analysis\n");
    println!("{}", "=".repeat(60));
    println!();

    // Show text visualization
    println!("{}", graph.to_text());
    println!();

    // Critical path analysis
    println!("{}", "=".repeat(60));
    println!("\n🎯 Critical Path Analysis\n");
    let critical_path = graph.find_critical_path();
    println!("Critical path length: {} tasks", critical_path.len());
    println!("This is the longest dependency chain in your application.\n");

    // Transitive dependencies
    println!("{}", "=".repeat(60));
    println!("\n🔗 Transitive Dependencies\n");
    let deps = graph.find_transitive_dependencies(aggregator.id);
    println!("Task 'aggregator' depends on {} other tasks:", deps.len());
    for dep_id in deps {
        if let Some(task) = graph.get_task(&dep_id) {
            println!("  → {}", task.name);
        }
    }
    println!();

    // Shared resources
    println!("{}", "=".repeat(60));
    println!("\n🔒 Shared Resource Analysis\n");
    let sharing_tasks = graph.find_tasks_sharing_resource("connection_mutex");
    println!("Tasks sharing 'connection_mutex': {}", sharing_tasks.len());
    for task_id in sharing_tasks {
        if let Some(task) = graph.get_task(&task_id) {
            println!("  • {} ({:?})", task.name, task.state);
        }
    }
    println!("\n⚠️  Potential contention point - consider using a connection pool!");
    println!();

    // Channel communication
    println!("{}", "=".repeat(60));
    println!("\n📡 Channel Communication Pairs\n");
    let pairs = graph.find_channel_pairs();
    println!("Found {} channel communication pairs:", pairs.len());
    for (from_id, to_id) in pairs {
        if let (Some(from_task), Some(to_task)) = (graph.get_task(&from_id), graph.get_task(&to_id))
        {
            println!("  {} → {}", from_task.name, to_task.name);
        }
    }
    println!();

    // Deadlock detection
    println!("{}", "=".repeat(60));
    println!("\n💀 Deadlock Detection\n");
    let deadlocks = graph.detect_potential_deadlocks();
    if deadlocks.is_empty() {
        println!("✅ No potential deadlocks detected!");
    } else {
        println!("⚠️  Found {} potential deadlock cycle(s):", deadlocks.len());
        for (i, cycle) in deadlocks.iter().enumerate() {
            println!("\nCycle {}:", i + 1);
            for task_id in cycle {
                if let Some(task) = graph.get_task(task_id) {
                    println!("  → {}", task.name);
                }
            }
        }
    }
    println!();

    // Relationship types breakdown
    println!("{}", "=".repeat(60));
    println!("\n📈 Relationship Types Breakdown\n");

    let spawned = graph
        .get_relationships_by_type(RelationshipType::Spawned)
        .len();
    let channels = graph
        .get_relationships_by_type(RelationshipType::ChannelSend)
        .len();
    let shared = graph
        .get_relationships_by_type(RelationshipType::SharedResource)
        .len();
    let data_flow = graph
        .get_relationships_by_type(RelationshipType::DataFlow)
        .len();
    let dependencies = graph
        .get_relationships_by_type(RelationshipType::Dependency)
        .len();
    let awaits = graph
        .get_relationships_by_type(RelationshipType::AwaitsOn)
        .len();

    println!("  Spawned:          {}", spawned);
    println!("  Channel Send:     {}", channels);
    println!("  Shared Resources: {}", shared);
    println!("  Data Flow:        {}", data_flow);
    println!("  Dependencies:     {}", dependencies);
    println!("  Awaits:           {}", awaits);
    println!(
        "  Total:            {}",
        spawned + channels + shared + data_flow + dependencies + awaits
    );
    println!();

    // DOT export
    println!("{}", "=".repeat(60));
    println!("\n📊 GraphViz DOT Format\n");
    println!("To visualize this graph:");
    println!("  1. Save the output below to 'graph.dot'");
    println!("  2. Run: dot -Tpng graph.dot -o graph.png");
    println!("  3. Open graph.png\n");
    println!("{}", graph.to_dot());

    println!("\n✅ Analysis complete!\n");
}

/// Helper to create a sample task
fn create_task(id: u64, name: &str, state: TaskState) -> TaskInfo {
    let now = Instant::now();
    TaskInfo {
        id: TaskId::from_u64(id),
        name: name.to_string(),
        state,
        created_at: now,
        last_updated: now,
        poll_count: 0,
        total_run_time: std::time::Duration::from_millis(0),
        parent: None,
        location: None,
    }
}
