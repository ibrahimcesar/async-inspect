---
sidebar_position: 8
---

# Task Filtering and Sorting

async-inspect provides powerful filtering and sorting capabilities to help you find exactly the tasks you're looking for in large applications.

## Overview

When debugging async applications, you often need to find specific tasks among thousands. The `TaskFilter` and sorting APIs let you:

- Filter tasks by state (running, blocked, pending, completed, failed)
- Search by name patterns
- Filter by duration/age
- Filter by poll count
- Find child tasks of a specific parent
- Sort results by various criteria

## Using TaskFilter

The `TaskFilter` struct uses a builder pattern for constructing queries:

```rust
use async_inspect::prelude::*;
use std::time::Duration;

let inspector = Inspector::global();

// Find all running tasks with "fetch" in the name
let filter = TaskFilter::new()
    .with_state(TaskState::Running)
    .with_name_pattern("fetch");

let running_fetch_tasks = inspector.get_tasks_filtered(&filter);
```

### Filter Criteria

#### By State

Filter tasks by their current execution state:

```rust
// Find all blocked tasks
let filter = TaskFilter::new()
    .with_state(TaskState::Blocked { await_point: String::new() });

// Find running tasks
let filter = TaskFilter::new()
    .with_state(TaskState::Running);

// Find completed tasks
let filter = TaskFilter::new()
    .with_state(TaskState::Completed);
```

#### By Name Pattern

Search for tasks with names containing a substring (case-insensitive):

```rust
// Find all tasks with "database" in the name
let filter = TaskFilter::new()
    .with_name_pattern("database");

// Find tasks related to HTTP
let filter = TaskFilter::new()
    .with_name_pattern("http");
```

#### By Duration

Find tasks older or newer than a threshold:

```rust
use std::time::Duration;

// Find long-running tasks (older than 5 seconds)
let filter = TaskFilter::new()
    .with_min_duration(Duration::from_secs(5));

// Find recently started tasks (less than 1 second old)
let filter = TaskFilter::new()
    .with_max_duration(Duration::from_secs(1));

// Find tasks in a specific age range
let filter = TaskFilter::new()
    .with_min_duration(Duration::from_secs(2))
    .with_max_duration(Duration::from_secs(10));
```

#### By Poll Count

Filter based on how many times a task has been polled:

```rust
// Find tasks that have been polled many times (potential busy-wait)
let filter = TaskFilter::new()
    .with_min_polls(1000);

// Find tasks that haven't been polled much
let filter = TaskFilter::new()
    .with_max_polls(5);
```

#### By Parent Task

Find child tasks of a specific parent:

```rust
// Get all children of a specific task
let filter = TaskFilter::new()
    .with_parent(parent_task_id);

// Get only root tasks (no parent)
let filter = TaskFilter::new()
    .root_only();
```

### Combining Filters

Filters can be combined for precise queries:

```rust
// Find long-running HTTP tasks that are currently blocked
let filter = TaskFilter::new()
    .with_state(TaskState::Blocked { await_point: String::new() })
    .with_name_pattern("http")
    .with_min_duration(Duration::from_secs(10));

let stuck_http_tasks = inspector.get_tasks_filtered(&filter);
```

## Sorting Results

Use `get_tasks_sorted` to get filtered results in a specific order:

```rust
use async_inspect::task::{TaskSortBy, SortDirection};

// Get blocked tasks sorted by age (oldest first)
let filter = TaskFilter::new()
    .with_state(TaskState::Blocked { await_point: String::new() });

let tasks = inspector.get_tasks_sorted(
    &filter,
    TaskSortBy::Age,
    SortDirection::Descending
);
```

### Sort Options

Available sort criteria:

| Sort By | Description |
|---------|-------------|
| `TaskSortBy::Id` | Sort by task ID (creation order) |
| `TaskSortBy::Name` | Sort alphabetically by name |
| `TaskSortBy::Age` | Sort by task age |
| `TaskSortBy::Polls` | Sort by poll count |
| `TaskSortBy::RunTime` | Sort by total CPU time |
| `TaskSortBy::State` | Sort by state (Running, Blocked, Pending, Completed, Failed) |

Sort directions:

- `SortDirection::Ascending` - smallest/oldest first
- `SortDirection::Descending` - largest/newest first

## Convenience Methods

The `Inspector` provides convenience methods for common queries:

```rust
let inspector = Inspector::global();

// Get all currently running tasks
let running = inspector.get_running_tasks();

// Get all blocked tasks
let blocked = inspector.get_blocked_tasks();

// Get tasks older than a threshold
let long_running = inspector.get_long_running_tasks(Duration::from_secs(30));

// Get top-level tasks (no parent)
let roots = inspector.get_root_tasks();

// Get children of a specific task
let children = inspector.get_child_tasks(parent_id);
```

## Using Filters Directly

You can also use `TaskFilter` directly on collections of tasks:

```rust
let all_tasks: Vec<TaskInfo> = get_tasks_somehow();

let filter = TaskFilter::new()
    .with_state(TaskState::Running)
    .with_min_duration(Duration::from_secs(1));

// Filter references
let filtered: Vec<&TaskInfo> = filter.filter(&all_tasks);

// Filter and clone
let filtered_owned: Vec<TaskInfo> = filter.filter_cloned(all_tasks);
```

## Sorting Tasks Directly

Use the `sort_tasks` function on any mutable slice:

```rust
use async_inspect::task::{sort_tasks, TaskSortBy, SortDirection};

let mut tasks = inspector.get_all_tasks();
sort_tasks(&mut tasks, TaskSortBy::RunTime, SortDirection::Descending);

// Now tasks are sorted by CPU time, heaviest first
for task in tasks.iter().take(10) {
    println!("Heavy task: {} ({:.2}s CPU)",
        task.name,
        task.total_run_time.as_secs_f64()
    );
}
```

## Practical Examples

### Finding Stuck Tasks

```rust
// Tasks blocked for more than 30 seconds are likely stuck
let stuck = inspector.get_tasks_sorted(
    &TaskFilter::new()
        .with_state(TaskState::Blocked { await_point: String::new() })
        .with_min_duration(Duration::from_secs(30)),
    TaskSortBy::Age,
    SortDirection::Descending
);

for task in stuck {
    println!("[STUCK] {} blocked for {:.1}s",
        task.name,
        task.age().as_secs_f64()
    );
}
```

### Finding CPU-Heavy Tasks

```rust
// Tasks with high poll counts relative to age might be busy-waiting
let tasks = inspector.get_tasks_sorted(
    &TaskFilter::new().with_min_polls(100),
    TaskSortBy::Polls,
    SortDirection::Descending
);

for task in tasks.iter().take(5) {
    let polls_per_sec = task.poll_count as f64 / task.age().as_secs_f64();
    println!("{}: {} polls ({:.1}/sec)",
        task.name,
        task.poll_count,
        polls_per_sec
    );
}
```

### Analyzing Task Hierarchies

```rust
// Get root tasks and their immediate children
let roots = inspector.get_root_tasks();

for root in roots {
    let children = inspector.get_child_tasks(root.id);
    println!("{} ({} children)", root.name, children.len());

    for child in children {
        println!("  - {} [{}]", child.name, child.state);
    }
}
```

## Performance Notes

- Filtering is performed on the current snapshot of tasks
- Large filter operations hold a read lock briefly
- For very large task sets, consider filtering early and often
- The `filter` method returns references to avoid cloning when possible
