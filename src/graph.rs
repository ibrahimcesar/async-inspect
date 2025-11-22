//! Task relationship graph and visualization
//!
//! This module provides comprehensive relationship tracking between async tasks,
//! including spawning, channels, shared resources, data flow, and dependencies.

use crate::task::{TaskId, TaskInfo, TaskState};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::sync::Arc;

/// Types of relationships between tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    /// Parent-child spawn relationship
    Spawned,
    /// Channel send (A sends data to B)
    ChannelSend,
    /// Channel receive (A receives from B)
    ChannelReceive,
    /// Shared resource access (mutex, rwlock, etc.)
    SharedResource,
    /// Data flow (data passed from A to B)
    DataFlow,
    /// Awaits on another task's completion
    AwaitsOn,
    /// Dependency (A depends on B to complete)
    Dependency,
}

impl fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawned => write!(f, "spawned"),
            Self::ChannelSend => write!(f, "sends →"),
            Self::ChannelReceive => write!(f, "← receives"),
            Self::SharedResource => write!(f, "shares resource"),
            Self::DataFlow => write!(f, "data →"),
            Self::AwaitsOn => write!(f, "awaits"),
            Self::Dependency => write!(f, "depends on"),
        }
    }
}

/// A relationship between two tasks
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Source task
    pub from: TaskId,
    /// Target task
    pub to: TaskId,
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// Optional resource name (for shared resources)
    pub resource_name: Option<String>,
    /// Optional data description
    pub data_description: Option<String>,
}

/// Graph of task relationships
#[derive(Debug, Clone)]
pub struct TaskGraph {
    /// All relationships
    relationships: Vec<Relationship>,
    /// Task metadata
    tasks: HashMap<TaskId, TaskInfo>,
    /// Adjacency list for efficient traversal
    adjacency: HashMap<TaskId, Vec<(TaskId, RelationshipType)>>,
    /// Reverse adjacency for finding dependents
    reverse_adjacency: HashMap<TaskId, Vec<(TaskId, RelationshipType)>>,
}

impl TaskGraph {
    /// Create a new task graph
    pub fn new() -> Self {
        Self {
            relationships: Vec::new(),
            tasks: HashMap::new(),
            adjacency: HashMap::new(),
            reverse_adjacency: HashMap::new(),
        }
    }

    /// Add a task to the graph
    pub fn add_task(&mut self, task: TaskInfo) {
        self.tasks.insert(task.id, task);
    }

    /// Add a relationship between tasks
    pub fn add_relationship(&mut self, relationship: Relationship) {
        // Update adjacency lists
        self.adjacency
            .entry(relationship.from)
            .or_insert_with(Vec::new)
            .push((relationship.to, relationship.relationship_type));

        self.reverse_adjacency
            .entry(relationship.to)
            .or_insert_with(Vec::new)
            .push((relationship.from, relationship.relationship_type));

        self.relationships.push(relationship);
    }

    /// Get all relationships of a specific type
    pub fn get_relationships_by_type(&self, rel_type: RelationshipType) -> Vec<&Relationship> {
        self.relationships
            .iter()
            .filter(|r| r.relationship_type == rel_type)
            .collect()
    }

    /// Get all tasks that a given task has a relationship with
    pub fn get_related_tasks(&self, task_id: TaskId) -> Vec<(TaskId, RelationshipType)> {
        self.adjacency.get(&task_id).cloned().unwrap_or_default()
    }

    /// Get all tasks that have a relationship to a given task
    pub fn get_dependent_tasks(&self, task_id: TaskId) -> Vec<(TaskId, RelationshipType)> {
        self.reverse_adjacency
            .get(&task_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get a task by ID
    pub fn get_task(&self, task_id: &TaskId) -> Option<&TaskInfo> {
        self.tasks.get(task_id)
    }

    /// Find the critical path (longest dependency chain)
    pub fn find_critical_path(&self) -> Vec<TaskId> {
        let mut longest_path = Vec::new();
        let mut visited = HashSet::new();

        for task_id in self.tasks.keys() {
            let path = self.find_longest_path(*task_id, &mut visited);
            if path.len() > longest_path.len() {
                longest_path = path;
            }
        }

        longest_path
    }

    /// Find longest path from a given task
    fn find_longest_path(&self, task_id: TaskId, visited: &mut HashSet<TaskId>) -> Vec<TaskId> {
        if visited.contains(&task_id) {
            return vec![];
        }

        visited.insert(task_id);
        let mut longest = vec![task_id];

        if let Some(related) = self.adjacency.get(&task_id) {
            for (next_id, rel_type) in related {
                // Only follow dependency and data flow relationships for critical path
                if matches!(
                    rel_type,
                    RelationshipType::Dependency
                        | RelationshipType::DataFlow
                        | RelationshipType::AwaitsOn
                ) {
                    let mut path = self.find_longest_path(*next_id, visited);
                    if path.len() + 1 > longest.len() {
                        path.insert(0, task_id);
                        longest = path;
                    }
                }
            }
        }

        visited.remove(&task_id);
        longest
    }

    /// Find all transitive dependencies of a task
    pub fn find_transitive_dependencies(&self, task_id: TaskId) -> HashSet<TaskId> {
        let mut dependencies = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(task_id);

        while let Some(current) = queue.pop_front() {
            if let Some(related) = self.adjacency.get(&current) {
                for (next_id, rel_type) in related {
                    if matches!(
                        rel_type,
                        RelationshipType::Dependency | RelationshipType::AwaitsOn
                    ) {
                        if dependencies.insert(*next_id) {
                            queue.push_back(*next_id);
                        }
                    }
                }
            }
        }

        dependencies
    }

    /// Find all tasks sharing a resource
    pub fn find_tasks_sharing_resource(&self, resource_name: &str) -> Vec<TaskId> {
        let mut tasks = HashSet::new();

        for rel in &self.relationships {
            if rel.relationship_type == RelationshipType::SharedResource {
                if let Some(ref name) = rel.resource_name {
                    if name == resource_name {
                        tasks.insert(rel.from);
                        tasks.insert(rel.to);
                    }
                }
            }
        }

        tasks.into_iter().collect()
    }

    /// Find channel communication pairs
    pub fn find_channel_pairs(&self) -> Vec<(TaskId, TaskId)> {
        let mut pairs = Vec::new();

        for send in self.get_relationships_by_type(RelationshipType::ChannelSend) {
            for recv in self.get_relationships_by_type(RelationshipType::ChannelReceive) {
                // Match send/receive on same channel
                if send.resource_name == recv.resource_name && send.to == recv.from {
                    pairs.push((send.from, recv.to));
                }
            }
        }

        pairs
    }

    /// Detect potential deadlocks based on resource sharing
    pub fn detect_potential_deadlocks(&self) -> Vec<Vec<TaskId>> {
        let mut deadlock_cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task_id in self.tasks.keys() {
            if !visited.contains(task_id) {
                if let Some(cycle) = self.find_cycle(*task_id, &mut visited, &mut rec_stack) {
                    deadlock_cycles.push(cycle);
                }
            }
        }

        deadlock_cycles
    }

    /// Find cycles in the graph (potential deadlocks)
    fn find_cycle(
        &self,
        task_id: TaskId,
        visited: &mut HashSet<TaskId>,
        rec_stack: &mut HashSet<TaskId>,
    ) -> Option<Vec<TaskId>> {
        visited.insert(task_id);
        rec_stack.insert(task_id);

        if let Some(related) = self.adjacency.get(&task_id) {
            for (next_id, rel_type) in related {
                // Only consider blocking relationships
                if matches!(
                    rel_type,
                    RelationshipType::SharedResource | RelationshipType::AwaitsOn
                ) {
                    if !visited.contains(next_id) {
                        if let Some(cycle) = self.find_cycle(*next_id, visited, rec_stack) {
                            return Some(cycle);
                        }
                    } else if rec_stack.contains(next_id) {
                        // Found a cycle
                        return Some(vec![task_id, *next_id]);
                    }
                }
            }
        }

        rec_stack.remove(&task_id);
        None
    }

    /// Generate DOT format for graphviz visualization
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph TaskGraph {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Add nodes
        for (task_id, task) in &self.tasks {
            let color = match task.state {
                TaskState::Pending => "lightgray",
                TaskState::Running => "lightblue",
                TaskState::Blocked { .. } => "yellow",
                TaskState::Completed => "lightgreen",
                TaskState::Failed => "lightcoral",
            };

            dot.push_str(&format!(
                "  t{} [label=\"{}\n{:?}\", fillcolor={}, style=\"filled,rounded\"];\n",
                task_id.as_u64(),
                task.name,
                task.state,
                color
            ));
        }

        dot.push_str("\n");

        // Add edges with different styles for different relationship types
        for rel in &self.relationships {
            let (style, color, label) = match rel.relationship_type {
                RelationshipType::Spawned => ("solid", "black", "spawned"),
                RelationshipType::ChannelSend => ("dashed", "blue", "→ channel"),
                RelationshipType::ChannelReceive => ("dashed", "blue", "channel →"),
                RelationshipType::SharedResource => ("dotted", "red", "shares"),
                RelationshipType::DataFlow => ("bold", "green", "data →"),
                RelationshipType::AwaitsOn => ("solid", "purple", "awaits"),
                RelationshipType::Dependency => ("solid", "orange", "depends"),
            };

            let mut edge_label = label.to_string();
            if let Some(ref resource) = rel.resource_name {
                edge_label = format!("{}\n{}", label, resource);
            }

            dot.push_str(&format!(
                "  t{} -> t{} [label=\"{}\", style={}, color={}];\n",
                rel.from.as_u64(),
                rel.to.as_u64(),
                edge_label,
                style,
                color
            ));
        }

        // Highlight critical path
        let critical_path = self.find_critical_path();
        if critical_path.len() > 1 {
            dot.push_str("\n  // Critical path\n");
            for window in critical_path.windows(2) {
                dot.push_str(&format!(
                    "  t{} -> t{} [color=red, penwidth=3.0, constraint=false];\n",
                    window[0].as_u64(),
                    window[1].as_u64()
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Generate a text-based visualization
    pub fn to_text(&self) -> String {
        let mut output = String::new();
        output.push_str("Task Relationship Graph\n");
        output.push_str("=======================\n\n");

        // Group by relationship type
        for rel_type in &[
            RelationshipType::Spawned,
            RelationshipType::ChannelSend,
            RelationshipType::ChannelReceive,
            RelationshipType::SharedResource,
            RelationshipType::DataFlow,
            RelationshipType::AwaitsOn,
            RelationshipType::Dependency,
        ] {
            let rels = self.get_relationships_by_type(*rel_type);
            if !rels.is_empty() {
                output.push_str(&format!("\n{} Relationships:\n", rel_type));
                for rel in rels {
                    let from_name = self
                        .tasks
                        .get(&rel.from)
                        .map(|t| t.name.as_str())
                        .unwrap_or("?");
                    let to_name = self
                        .tasks
                        .get(&rel.to)
                        .map(|t| t.name.as_str())
                        .unwrap_or("?");

                    output.push_str(&format!("  {} {} {}", from_name, rel_type, to_name));

                    if let Some(ref resource) = rel.resource_name {
                        output.push_str(&format!(" ({})", resource));
                    }
                    output.push('\n');
                }
            }
        }

        // Critical path
        let critical_path = self.find_critical_path();
        if !critical_path.is_empty() {
            output.push_str("\nCritical Path:\n");
            for task_id in &critical_path {
                if let Some(task) = self.tasks.get(task_id) {
                    output.push_str(&format!("  → {} ({:?})\n", task.name, task.state));
                }
            }
        }

        // Resource sharing
        let mut resources: HashMap<String, Vec<TaskId>> = HashMap::new();
        for rel in &self.relationships {
            if rel.relationship_type == RelationshipType::SharedResource {
                if let Some(ref name) = rel.resource_name {
                    resources
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(rel.from);
                    resources
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .push(rel.to);
                }
            }
        }

        if !resources.is_empty() {
            output.push_str("\nShared Resources:\n");
            for (resource, task_ids) in resources {
                let unique_tasks: HashSet<_> = task_ids.into_iter().collect();
                output.push_str(&format!(
                    "  {} (accessed by {} tasks):\n",
                    resource,
                    unique_tasks.len()
                ));
                for task_id in unique_tasks {
                    if let Some(task) = self.tasks.get(&task_id) {
                        output.push_str(&format!("    - {}\n", task.name));
                    }
                }
            }
        }

        output
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Global task graph instance
static GRAPH: once_cell::sync::Lazy<Arc<RwLock<TaskGraph>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(TaskGraph::new())));

/// Get the global task graph
pub fn global_graph() -> Arc<RwLock<TaskGraph>> {
    Arc::clone(&GRAPH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::TaskId;
    use std::time::{Duration, Instant};

    #[test]
    fn test_critical_path() {
        let mut graph = TaskGraph::new();

        let t1 = TaskId::from_u64(1);
        let t2 = TaskId::from_u64(2);
        let t3 = TaskId::from_u64(3);

        // Add tasks to the graph
        use crate::task::{TaskInfo, TaskState};
        let now = Instant::now();
        graph.add_task(TaskInfo {
            id: t1,
            name: "task1".to_string(),
            state: TaskState::Running,
            created_at: now,
            last_updated: now,
            parent: None,
            location: None,
            poll_count: 0,
            total_run_time: Duration::ZERO,
        });
        graph.add_task(TaskInfo {
            id: t2,
            name: "task2".to_string(),
            state: TaskState::Running,
            created_at: now,
            last_updated: now,
            parent: None,
            location: None,
            poll_count: 0,
            total_run_time: Duration::ZERO,
        });
        graph.add_task(TaskInfo {
            id: t3,
            name: "task3".to_string(),
            state: TaskState::Running,
            created_at: now,
            last_updated: now,
            parent: None,
            location: None,
            poll_count: 0,
            total_run_time: Duration::ZERO,
        });

        graph.add_relationship(Relationship {
            from: t1,
            to: t2,
            relationship_type: RelationshipType::Dependency,
            resource_name: None,
            data_description: None,
        });

        graph.add_relationship(Relationship {
            from: t2,
            to: t3,
            relationship_type: RelationshipType::Dependency,
            resource_name: None,
            data_description: None,
        });

        let path = graph.find_critical_path();
        assert!(path.contains(&t1));
        assert!(path.contains(&t2));
        assert!(path.contains(&t3));
    }

    #[test]
    fn test_shared_resources() {
        let mut graph = TaskGraph::new();

        let t1 = TaskId::from_u64(1);
        let t2 = TaskId::from_u64(2);

        graph.add_relationship(Relationship {
            from: t1,
            to: t2,
            relationship_type: RelationshipType::SharedResource,
            resource_name: Some("mutex_1".to_string()),
            data_description: None,
        });

        let tasks = graph.find_tasks_sharing_resource("mutex_1");
        assert_eq!(tasks.len(), 2);
    }
}
