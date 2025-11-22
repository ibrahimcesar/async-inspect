//! Deadlock detection and analysis
//!
//! This module provides automatic detection of deadlocks caused by circular
//! dependencies between tasks waiting on resources (mutexes, channels, etc.).

use crate::task::TaskId;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Unique identifier for a resource (lock, channel, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(u64);

impl ResourceId {
    /// Create a new unique resource ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resource#{}", self.0)
    }
}

/// Type of resource that can be waited on
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    /// Mutex lock
    Mutex,
    /// RwLock (read or write)
    RwLock,
    /// Semaphore
    Semaphore,
    /// Channel (send or receive)
    Channel,
    /// Other resource type
    Other(String),
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mutex => write!(f, "Mutex"),
            Self::RwLock => write!(f, "RwLock"),
            Self::Semaphore => write!(f, "Semaphore"),
            Self::Channel => write!(f, "Channel"),
            Self::Other(name) => write!(f, "{}", name),
        }
    }
}

/// Information about a resource
#[derive(Debug, Clone)]
pub struct ResourceInfo {
    /// Unique resource identifier
    pub id: ResourceId,

    /// Type of resource
    pub kind: ResourceKind,

    /// Name or description
    pub name: String,

    /// Task currently holding this resource (if any)
    pub holder: Option<TaskId>,

    /// Tasks waiting for this resource
    pub waiters: Vec<TaskId>,

    /// Memory address (for debugging)
    pub address: Option<usize>,
}

impl ResourceInfo {
    /// Create a new resource info
    pub fn new(kind: ResourceKind, name: String) -> Self {
        Self {
            id: ResourceId::new(),
            kind,
            name,
            holder: None,
            waiters: Vec::new(),
            address: None,
        }
    }

    /// Set the memory address
    pub fn with_address(mut self, address: usize) -> Self {
        self.address = Some(address);
        self
    }

    /// Check if resource is held
    pub fn is_held(&self) -> bool {
        self.holder.is_some()
    }

    /// Check if resource has waiters
    pub fn has_waiters(&self) -> bool {
        !self.waiters.is_empty()
    }
}

impl fmt::Display for ResourceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} '{}' ({})", self.kind, self.name, self.id)?;
        if let Some(addr) = self.address {
            write!(f, " @ 0x{:x}", addr)?;
        }
        Ok(())
    }
}

/// A cycle in the wait-for graph (deadlock)
#[derive(Debug, Clone)]
pub struct DeadlockCycle {
    /// Tasks involved in the cycle
    pub tasks: Vec<TaskId>,

    /// Resources involved in the cycle
    pub resources: Vec<ResourceId>,

    /// Detailed chain: Task -> Resource -> Task -> ...
    pub chain: Vec<WaitEdge>,
}

/// An edge in the wait-for graph
#[derive(Debug, Clone)]
pub struct WaitEdge {
    /// Task waiting
    pub task: TaskId,

    /// Resource being waited for
    pub resource: ResourceId,

    /// Task holding the resource
    pub holder: TaskId,
}

impl DeadlockCycle {
    /// Get a human-readable description of the cycle
    pub fn describe(&self) -> String {
        let mut desc = String::from("Deadlock cycle detected:\n");

        for (i, edge) in self.chain.iter().enumerate() {
            desc.push_str(&format!(
                "  {} Task {} → {} → Task {}\n",
                if i == 0 { "→" } else { " " },
                edge.task,
                edge.resource,
                edge.holder
            ));
        }

        desc.push_str(&format!(
            "\n{} tasks and {} resources involved",
            self.tasks.len(),
            self.resources.len()
        ));

        desc
    }
}

/// Deadlock detector
#[derive(Clone)]
pub struct DeadlockDetector {
    /// Shared state
    state: Arc<RwLock<DetectorState>>,
}

struct DetectorState {
    /// All tracked resources
    resources: HashMap<ResourceId, ResourceInfo>,

    /// Mapping from task to resources it's waiting for
    task_waiting: HashMap<TaskId, ResourceId>,

    /// Whether detection is enabled
    enabled: bool,
}

impl DeadlockDetector {
    /// Create a new deadlock detector
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(DetectorState {
                resources: HashMap::new(),
                task_waiting: HashMap::new(),
                enabled: true,
            })),
        }
    }

    /// Enable deadlock detection
    pub fn enable(&self) {
        self.state.write().enabled = true;
    }

    /// Disable deadlock detection
    pub fn disable(&self) {
        self.state.write().enabled = false;
    }

    /// Check if detection is enabled
    pub fn is_enabled(&self) -> bool {
        self.state.read().enabled
    }

    /// Register a new resource
    pub fn register_resource(&self, info: ResourceInfo) -> ResourceId {
        if !self.is_enabled() {
            return info.id;
        }

        let resource_id = info.id;
        self.state.write().resources.insert(resource_id, info);
        resource_id
    }

    /// Record a task acquiring a resource
    pub fn acquire(&self, task_id: TaskId, resource_id: ResourceId) {
        if !self.is_enabled() {
            return;
        }

        let mut state = self.state.write();

        // Remove from waiting
        state.task_waiting.remove(&task_id);

        // Set as holder
        if let Some(resource) = state.resources.get_mut(&resource_id) {
            resource.holder = Some(task_id);
            resource.waiters.retain(|&t| t != task_id);
        }
    }

    /// Record a task releasing a resource
    pub fn release(&self, task_id: TaskId, resource_id: ResourceId) {
        if !self.is_enabled() {
            return;
        }

        let mut state = self.state.write();

        if let Some(resource) = state.resources.get_mut(&resource_id) {
            if resource.holder == Some(task_id) {
                resource.holder = None;
            }
        }
    }

    /// Record a task waiting for a resource
    pub fn wait_for(&self, task_id: TaskId, resource_id: ResourceId) {
        if !self.is_enabled() {
            return;
        }

        let mut state = self.state.write();

        // Record waiting
        state.task_waiting.insert(task_id, resource_id);

        // Add to waiters list
        if let Some(resource) = state.resources.get_mut(&resource_id) {
            if !resource.waiters.contains(&task_id) {
                resource.waiters.push(task_id);
            }
        }
    }

    /// Detect deadlocks using cycle detection
    pub fn detect_deadlocks(&self) -> Vec<DeadlockCycle> {
        let state = self.state.read();

        // Build wait-for graph: Task -> Task via Resource
        let mut graph: HashMap<TaskId, Vec<TaskId>> = HashMap::new();
        let mut task_to_resource: HashMap<TaskId, ResourceId> = HashMap::new();

        for (&waiting_task, &resource_id) in &state.task_waiting {
            if let Some(resource) = state.resources.get(&resource_id) {
                if let Some(holder_task) = resource.holder {
                    graph.entry(waiting_task).or_default().push(holder_task);
                    task_to_resource.insert(waiting_task, resource_id);
                }
            }
        }

        // Find cycles using DFS
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &task in graph.keys() {
            if !visited.contains(&task) {
                if let Some(cycle) = self.find_cycle_dfs(
                    task,
                    &graph,
                    &task_to_resource,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                ) {
                    cycles.push(cycle);
                }
            }
        }

        cycles
    }

    /// DFS-based cycle detection
    fn find_cycle_dfs(
        &self,
        task: TaskId,
        graph: &HashMap<TaskId, Vec<TaskId>>,
        task_to_resource: &HashMap<TaskId, ResourceId>,
        visited: &mut HashSet<TaskId>,
        rec_stack: &mut HashSet<TaskId>,
        path: &mut Vec<TaskId>,
    ) -> Option<DeadlockCycle> {
        visited.insert(task);
        rec_stack.insert(task);
        path.push(task);

        if let Some(neighbors) = graph.get(&task) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if let Some(cycle) = self.find_cycle_dfs(
                        neighbor,
                        graph,
                        task_to_resource,
                        visited,
                        rec_stack,
                        path,
                    ) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(&neighbor) {
                    // Found a cycle!
                    return Some(self.build_cycle(neighbor, path, task_to_resource));
                }
            }
        }

        rec_stack.remove(&task);
        path.pop();
        None
    }

    /// Build a deadlock cycle from the path
    fn build_cycle(
        &self,
        start_task: TaskId,
        path: &[TaskId],
        task_to_resource: &HashMap<TaskId, ResourceId>,
    ) -> DeadlockCycle {
        // Find where the cycle starts
        let cycle_start = path.iter().position(|&t| t == start_task).unwrap_or(0);
        let cycle_tasks: Vec<TaskId> = path[cycle_start..].to_vec();

        let mut resources = Vec::new();
        let mut chain = Vec::new();

        for i in 0..cycle_tasks.len() {
            let waiting_task = cycle_tasks[i];
            let holder_task = cycle_tasks[(i + 1) % cycle_tasks.len()];

            if let Some(&resource_id) = task_to_resource.get(&waiting_task) {
                resources.push(resource_id);
                chain.push(WaitEdge {
                    task: waiting_task,
                    resource: resource_id,
                    holder: holder_task,
                });
            }
        }

        DeadlockCycle {
            tasks: cycle_tasks,
            resources,
            chain,
        }
    }

    /// Get all resources
    pub fn get_resources(&self) -> Vec<ResourceInfo> {
        self.state.read().resources.values().cloned().collect()
    }

    /// Get a specific resource
    pub fn get_resource(&self, id: ResourceId) -> Option<ResourceInfo> {
        self.state.read().resources.get(&id).cloned()
    }

    /// Clear all tracking data
    pub fn clear(&self) {
        let mut state = self.state.write();
        state.resources.clear();
        state.task_waiting.clear();
    }
}

impl Default for DeadlockDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resource = ResourceInfo::new(ResourceKind::Mutex, "test_mutex".to_string());
        assert_eq!(resource.kind, ResourceKind::Mutex);
        assert_eq!(resource.name, "test_mutex");
        assert!(!resource.is_held());
        assert!(!resource.has_waiters());
    }

    #[test]
    fn test_detector_registration() {
        let detector = DeadlockDetector::new();
        let resource = ResourceInfo::new(ResourceKind::Mutex, "test".to_string());
        let resource_id = resource.id;

        detector.register_resource(resource);

        let retrieved = detector.get_resource(resource_id).unwrap();
        assert_eq!(retrieved.name, "test");
    }

    #[test]
    fn test_simple_deadlock_detection() {
        let detector = DeadlockDetector::new();

        // Create two resources
        let res1 = ResourceInfo::new(ResourceKind::Mutex, "mutex_a".to_string());
        let res2 = ResourceInfo::new(ResourceKind::Mutex, "mutex_b".to_string());
        let res1_id = res1.id;
        let res2_id = res2.id;

        detector.register_resource(res1);
        detector.register_resource(res2);

        // Create two tasks
        let task1 = TaskId::new();
        let task2 = TaskId::new();

        // Task1 holds res1, waits for res2
        detector.acquire(task1, res1_id);
        detector.wait_for(task1, res2_id);

        // Task2 holds res2, waits for res1
        detector.acquire(task2, res2_id);
        detector.wait_for(task2, res1_id);

        // Detect deadlock
        let deadlocks = detector.detect_deadlocks();
        assert_eq!(deadlocks.len(), 1);

        let cycle = &deadlocks[0];
        assert_eq!(cycle.tasks.len(), 2);
        assert!(cycle.tasks.contains(&task1));
        assert!(cycle.tasks.contains(&task2));
    }

    #[test]
    fn test_no_deadlock() {
        let detector = DeadlockDetector::new();

        let res = ResourceInfo::new(ResourceKind::Mutex, "mutex".to_string());
        let res_id = res.id;
        detector.register_resource(res);

        let task1 = TaskId::new();
        let task2 = TaskId::new();

        // Task1 acquires and releases
        detector.acquire(task1, res_id);
        detector.release(task1, res_id);

        // Task2 acquires
        detector.acquire(task2, res_id);

        // No deadlock
        let deadlocks = detector.detect_deadlocks();
        assert_eq!(deadlocks.len(), 0);
    }
}
