//! Reporting and output formatting
//!
//! This module provides utilities for displaying inspection results.

use crate::inspector::{Inspector, InspectorStats};
use crate::task::{TaskInfo, TaskState};
use crate::timeline::Event;
use std::fmt::Write as FmtWrite;

pub mod html;

/// Reporter for inspection results
pub struct Reporter {
    inspector: Inspector,
}

impl Reporter {
    /// Create a new reporter
    pub fn new(inspector: Inspector) -> Self {
        Self { inspector }
    }

    /// Create a reporter using the global inspector
    pub fn global() -> Self {
        Self::new(Inspector::global().clone())
    }

    /// Print a summary of all tasks
    pub fn print_summary(&self) {
        let stats = self.inspector.stats();
        let tasks = self.inspector.get_all_tasks();

        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ async-inspect - Task Summary                                │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│                                                             │");

        self.print_stats(&stats);

        println!("│                                                             │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ Tasks                                                       │");
        println!("├─────────────────────────────────────────────────────────────┤");

        if tasks.is_empty() {
            println!("│ No tasks tracked                                            │");
        } else {
            for task in &tasks {
                self.print_task_line(task);
            }
        }

        println!("└─────────────────────────────────────────────────────────────┘");
    }

    /// Print statistics
    fn print_stats(&self, stats: &InspectorStats) {
        println!(
            "│ Total Tasks:     {:>3}                                      │",
            stats.total_tasks
        );
        println!(
            "│ Active:          {:>3} (Running: {}, Blocked: {})           │",
            stats.running_tasks + stats.blocked_tasks,
            stats.running_tasks,
            stats.blocked_tasks
        );
        println!(
            "│ Completed:       {:>3}                                      │",
            stats.completed_tasks
        );
        println!(
            "│ Failed:          {:>3}                                      │",
            stats.failed_tasks
        );
        println!(
            "│ Total Events:    {:>3}                                      │",
            stats.total_events
        );
        println!(
            "│ Duration:        {:.2}s                                   │",
            stats.timeline_duration.as_secs_f64()
        );
    }

    /// Print a single task line
    fn print_task_line(&self, task: &TaskInfo) {
        let state_icon = match task.state {
            TaskState::Pending => "⏸️ ",
            TaskState::Running => "🏃",
            TaskState::Blocked { .. } => "⏳",
            TaskState::Completed => "✅",
            TaskState::Failed => "❌",
        };

        let status = format!("{} {} {}", task.id, state_icon, task.name);
        println!("│ {:<59} │", status);

        // Show additional info for blocked tasks
        if let TaskState::Blocked { ref await_point } = task.state {
            let detail = format!(
                "    └─> Waiting: {} ({:.2}s)",
                await_point,
                task.time_since_update().as_secs_f64()
            );
            println!("│ {:<59} │", detail);
        }
    }

    /// Print detailed information about a specific task
    pub fn print_task_details(&self, task_id: crate::task::TaskId) {
        let Some(task) = self.inspector.get_task(task_id) else {
            println!("Task {} not found", task_id);
            return;
        };

        println!("┌─────────────────────────────────────────────────────────────┐");
        println!(
            "│ Task Details: {}                                           │",
            task.id
        );
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│                                                             │");
        println!("│ Name:            {:<44}│", task.name);
        println!("│ State:           {:<44}│", task.state.to_string());
        println!(
            "│ Age:             {:.2}s{:<38}│",
            task.age().as_secs_f64(),
            ""
        );
        println!("│ Poll Count:      {:<44}│", task.poll_count);
        println!(
            "│ Total Runtime:   {:.2}s{:<38}│",
            task.total_run_time.as_secs_f64(),
            ""
        );

        if let Some(parent) = task.parent {
            println!("│ Parent:          {:<44}│", parent.to_string());
        }

        if let Some(location) = &task.location {
            println!("│ Location:        {:<44}│", location);
        }

        println!("│                                                             │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ Events                                                      │");
        println!("├─────────────────────────────────────────────────────────────┤");

        let events = self.inspector.get_task_events(task_id);
        if events.is_empty() {
            println!("│ No events recorded                                          │");
        } else {
            for event in events.iter().take(20) {
                let event_str = format!("{}", event.kind);
                println!("│ {:<59} │", event_str);
            }

            if events.len() > 20 {
                println!(
                    "│ ... and {} more events                                    │",
                    events.len() - 20
                );
            }
        }

        println!("└─────────────────────────────────────────────────────────────┘");
    }

    /// Print timeline of all events
    pub fn print_timeline(&self) {
        let events = self.inspector.get_events();

        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ async-inspect - Timeline                                    │");
        println!("├─────────────────────────────────────────────────────────────┤");

        if events.is_empty() {
            println!("│ No events recorded                                          │");
        } else {
            for event in events.iter().take(50) {
                self.print_event_line(&event);
            }

            if events.len() > 50 {
                println!("│                                                             │");
                println!(
                    "│ ... and {} more events                                    │",
                    events.len() - 50
                );
            }
        }

        println!("└─────────────────────────────────────────────────────────────┘");
    }

    /// Print a single event line
    fn print_event_line(&self, event: &Event) {
        let time_str = format!("[{:.3}s]", event.age().as_secs_f64());
        let event_str = format!("{} {}: {}", time_str, event.task_id, event.kind);

        // Truncate if too long
        let truncated = if event_str.len() > 59 {
            format!("{}...", &event_str[..56])
        } else {
            event_str
        };

        println!("│ {:<59} │", truncated);
    }

    /// Generate a text report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        let stats = self.inspector.stats();
        let tasks = self.inspector.get_all_tasks();

        writeln!(report, "async-inspect Report").unwrap();
        writeln!(report, "====================").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "Statistics:").unwrap();
        writeln!(report, "  Total Tasks:     {}", stats.total_tasks).unwrap();
        writeln!(report, "  Pending:         {}", stats.pending_tasks).unwrap();
        writeln!(report, "  Running:         {}", stats.running_tasks).unwrap();
        writeln!(report, "  Blocked:         {}", stats.blocked_tasks).unwrap();
        writeln!(report, "  Completed:       {}", stats.completed_tasks).unwrap();
        writeln!(report, "  Failed:          {}", stats.failed_tasks).unwrap();
        writeln!(report, "  Total Events:    {}", stats.total_events).unwrap();
        writeln!(
            report,
            "  Duration:        {:.2}s",
            stats.timeline_duration.as_secs_f64()
        )
        .unwrap();
        writeln!(report).unwrap();

        writeln!(report, "Tasks:").unwrap();
        for task in &tasks {
            writeln!(report, "  {}", task).unwrap();
        }

        report
    }

    /// Print a compact one-line summary
    pub fn print_compact_summary(&self) {
        let stats = self.inspector.stats();
        println!(
            "async-inspect: {} tasks ({} active, {} completed, {} failed) | {} events | {:.2}s",
            stats.total_tasks,
            stats.running_tasks + stats.blocked_tasks,
            stats.completed_tasks,
            stats.failed_tasks,
            stats.total_events,
            stats.timeline_duration.as_secs_f64()
        );
    }

    /// Print a Gantt-style concurrency timeline
    pub fn print_gantt_timeline(&self) {
        let tasks = self.inspector.get_all_tasks();

        if tasks.is_empty() {
            println!("No tasks to display");
            return;
        }

        // Calculate time bounds
        let start_time = tasks
            .iter()
            .map(|t| t.created_at)
            .min()
            .expect("At least one task");

        let end_time = tasks
            .iter()
            .map(|t| t.created_at + t.age())
            .max()
            .expect("At least one task");

        let total_duration = end_time.duration_since(start_time);

        // Timeline configuration
        const TIMELINE_WIDTH: usize = 50;

        println!("┌────────────────────────────────────────────────────────────────────────────┐");
        println!("│ Concurrency Timeline (Gantt View)                                         │");
        println!("├────────────────────────────────────────────────────────────────────────────┤");
        println!("│                                                                            │");

        // Print time scale
        let time_markers = self.generate_time_markers(total_duration, TIMELINE_WIDTH);
        println!("│ Time:  {}│", time_markers);
        println!("│        {}│", self.generate_timeline_ruler(TIMELINE_WIDTH));
        println!("│                                                                            │");

        // Print each task as a timeline bar
        for task in &tasks {
            let task_line =
                self.generate_task_timeline(task, start_time, total_duration, TIMELINE_WIDTH);
            println!("│ {}│", task_line);
        }

        println!("│                                                                            │");
        println!("│ Legend: █ Running  ░ Blocked  ─ Waiting  ✓ Completed  ✗ Failed           │");
        println!("└────────────────────────────────────────────────────────────────────────────┘");
    }

    /// Generate time markers for the timeline
    fn generate_time_markers(&self, total_duration: std::time::Duration, width: usize) -> String {
        let mut markers = String::new();
        let millis = total_duration.as_millis();

        // Show markers at 0%, 25%, 50%, 75%, 100%
        let positions = [0, width / 4, width / 2, 3 * width / 4, width];
        let mut last_end = 0;

        for &pos in &positions {
            let time_ms = (millis as f64 * pos as f64 / width as f64) as u128;
            let marker = format!("{}ms", time_ms);

            // Add spacing
            if pos > last_end {
                let spaces = pos.saturating_sub(last_end);
                markers.push_str(&" ".repeat(spaces));
            }

            markers.push_str(&marker);
            last_end = pos + marker.len();
        }

        // Pad to width
        if markers.len() < width {
            markers.push_str(&" ".repeat(width - markers.len()));
        }

        markers
    }

    /// Generate timeline ruler
    fn generate_timeline_ruler(&self, width: usize) -> String {
        let mut ruler = String::new();
        for i in 0..width {
            if i % 10 == 0 {
                ruler.push('|');
            } else if i % 5 == 0 {
                ruler.push('·');
            } else {
                ruler.push('─');
            }
        }
        ruler
    }

    /// Generate a timeline bar for a single task
    fn generate_task_timeline(
        &self,
        task: &TaskInfo,
        start_time: std::time::Instant,
        total_duration: std::time::Duration,
        width: usize,
    ) -> String {
        let mut line = String::new();

        // Task name (first 12 chars)
        let name = if task.name.len() > 12 {
            format!("{:.9}...", task.name)
        } else {
            format!("{:<12}", task.name)
        };
        line.push_str(&name);
        line.push_str(": ");

        // Calculate task position and length
        let task_start = task.created_at.duration_since(start_time);
        let task_duration = task.age();

        let start_pos = ((task_start.as_millis() as f64 / total_duration.as_millis() as f64)
            * width as f64) as usize;
        let task_len = ((task_duration.as_millis() as f64 / total_duration.as_millis() as f64)
            * width as f64)
            .max(1.0) as usize;

        // Build the timeline bar
        for i in 0..width {
            if i < start_pos {
                line.push(' ');
            } else if i < start_pos + task_len {
                // Determine character based on task state
                let ch = match task.state {
                    TaskState::Running => '█',
                    TaskState::Blocked { .. } => '░',
                    TaskState::Completed => '█',
                    TaskState::Failed => '▓',
                    TaskState::Pending => '─',
                };
                line.push(ch);
            } else {
                line.push(' ');
            }
        }

        // Add state indicator
        let indicator = match task.state {
            TaskState::Completed => " ✓",
            TaskState::Failed => " ✗",
            TaskState::Running => " →",
            TaskState::Blocked { .. } => " ⏸",
            TaskState::Pending => " ○",
        };
        line.push_str(indicator);

        // Pad to consistent width
        while line.len() < 74 {
            line.push(' ');
        }

        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reporter_creation() {
        let inspector = Inspector::new();
        let reporter = Reporter::new(inspector);
        // Just verify it doesn't panic
        reporter.print_compact_summary();
    }

    #[test]
    fn test_generate_report() {
        let inspector = Inspector::new();
        inspector.register_task("test".to_string());

        let reporter = Reporter::new(inspector);
        let report = reporter.generate_report();

        assert!(report.contains("async-inspect Report"));
        assert!(report.contains("Total Tasks:     1"));
    }
}
