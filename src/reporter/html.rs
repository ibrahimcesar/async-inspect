//! HTML visualization output
//!
//! Generates interactive HTML reports with timeline visualization,
//! state machine graphs, and task inspection panels.

use crate::inspector::Inspector;
use crate::task::{TaskInfo, TaskState};
use std::fmt::Write as FmtWrite;

/// HTML report generator
pub struct HtmlReporter {
    inspector: Inspector,
}

impl HtmlReporter {
    /// Create a new HTML reporter
    pub fn new(inspector: Inspector) -> Self {
        Self { inspector }
    }

    /// Create a reporter using the global inspector
    pub fn global() -> Self {
        Self::new(Inspector::global().clone())
    }

    /// Generate a complete HTML report
    pub fn generate_html(&self) -> String {
        let mut html = String::new();

        // HTML structure
        writeln!(html, "<!DOCTYPE html>").unwrap();
        writeln!(html, "<html lang=\"en\">").unwrap();
        writeln!(html, "<head>").unwrap();
        writeln!(html, "    <meta charset=\"UTF-8\">").unwrap();
        writeln!(
            html,
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
        )
        .unwrap();
        writeln!(html, "    <title>async-inspect Report</title>").unwrap();

        // Embedded CSS
        html.push_str(&self.generate_css());

        writeln!(html, "</head>").unwrap();
        writeln!(html, "<body>").unwrap();

        // Header
        html.push_str(&self.generate_header());

        // Main content
        writeln!(html, "    <div class=\"container\">").unwrap();

        // Statistics panel
        html.push_str(&self.generate_stats_panel());

        // Timeline visualization
        html.push_str(&self.generate_timeline_viz());

        // State machine graph
        html.push_str(&self.generate_state_machine_graph());

        // Task list with details
        html.push_str(&self.generate_task_list());

        writeln!(html, "    </div>").unwrap();

        // Embedded JavaScript
        html.push_str(&self.generate_javascript());

        writeln!(html, "</body>").unwrap();
        writeln!(html, "</html>").unwrap();

        html
    }

    /// Generate CSS styles
    fn generate_css(&self) -> String {
        r#"
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            overflow: hidden;
        }

        header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }

        header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
        }

        header p {
            font-size: 1.2em;
            opacity: 0.9;
        }

        .stats-panel {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            padding: 30px;
            background: #f8f9fa;
            border-bottom: 1px solid #e0e0e0;
        }

        .stat-card {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            transition: transform 0.2s;
        }

        .stat-card:hover {
            transform: translateY(-5px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        }

        .stat-card .label {
            color: #666;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 5px;
        }

        .stat-card .value {
            font-size: 2em;
            font-weight: bold;
            color: #667eea;
        }

        .timeline-viz {
            padding: 30px;
            border-bottom: 1px solid #e0e0e0;
        }

        .timeline-viz h2 {
            margin-bottom: 20px;
            color: #333;
        }

        .timeline-container {
            position: relative;
            height: 400px;
            background: #f8f9fa;
            border-radius: 8px;
            overflow-x: auto;
            overflow-y: auto;
            border: 1px solid #e0e0e0;
        }

        .timeline-svg {
            width: 100%;
            min-width: 800px;
            height: 100%;
        }

        .task-row {
            cursor: pointer;
            transition: opacity 0.2s;
        }

        .task-row:hover {
            opacity: 0.8;
        }

        .task-bar {
            stroke-width: 2;
            stroke: white;
        }

        .task-bar.completed {
            fill: #4caf50;
        }

        .task-bar.running {
            fill: #2196f3;
        }

        .task-bar.blocked {
            fill: #ff9800;
        }

        .task-bar.failed {
            fill: #f44336;
        }

        .task-bar.pending {
            fill: #9e9e9e;
        }

        .task-list {
            padding: 30px;
        }

        .task-list h2 {
            margin-bottom: 20px;
            color: #333;
        }

        .task-item {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 15px;
            cursor: pointer;
            transition: all 0.2s;
            border-left: 4px solid #667eea;
        }

        .task-item:hover {
            background: #e9ecef;
            transform: translateX(5px);
        }

        .task-item.expanded {
            background: white;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .task-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .task-name {
            font-weight: bold;
            font-size: 1.1em;
            color: #333;
        }

        .task-state {
            padding: 5px 15px;
            border-radius: 20px;
            font-size: 0.85em;
            font-weight: bold;
            text-transform: uppercase;
        }

        .state-completed {
            background: #4caf50;
            color: white;
        }

        .state-running {
            background: #2196f3;
            color: white;
        }

        .state-blocked {
            background: #ff9800;
            color: white;
        }

        .state-failed {
            background: #f44336;
            color: white;
        }

        .state-pending {
            background: #9e9e9e;
            color: white;
        }

        .task-details {
            margin-top: 15px;
            padding-top: 15px;
            border-top: 1px solid #e0e0e0;
            display: none;
        }

        .task-item.expanded .task-details {
            display: block;
        }

        .task-meta {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 15px;
            margin-bottom: 15px;
        }

        .meta-item {
            font-size: 0.9em;
        }

        .meta-label {
            color: #666;
            font-weight: bold;
            margin-bottom: 3px;
        }

        .meta-value {
            color: #333;
        }

        .events-section {
            margin-top: 15px;
        }

        .events-section h4 {
            margin-bottom: 10px;
            color: #667eea;
        }

        .event-item {
            background: white;
            padding: 10px;
            margin-bottom: 8px;
            border-radius: 4px;
            border-left: 3px solid #667eea;
            font-size: 0.9em;
        }

        .event-time {
            color: #666;
            font-family: 'Courier New', monospace;
        }

        .legend {
            display: flex;
            gap: 20px;
            margin-top: 15px;
            padding: 15px;
            background: white;
            border-radius: 8px;
        }

        .legend-item {
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 0.9em;
        }

        .legend-color {
            width: 20px;
            height: 20px;
            border-radius: 4px;
        }

        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }

        .running-indicator {
            animation: pulse 1.5s infinite;
        }

        /* State Machine Graph */
        .state-machine-graph {
            padding: 30px;
            border-bottom: 1px solid #e0e0e0;
        }

        .state-machine-graph h2 {
            margin-bottom: 20px;
            color: #333;
        }

        .graph-container {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 20px;
            min-height: 400px;
            border: 1px solid #e0e0e0;
            position: relative;
        }

        .state-node {
            cursor: pointer;
            transition: all 0.2s;
        }

        .state-node:hover {
            transform: scale(1.05);
        }

        .state-node circle {
            stroke-width: 2;
            transition: all 0.2s;
        }

        .state-node:hover rect,
        .state-node:hover circle {
            stroke-width: 3;
        }

        .state-node.pending rect,
        .state-node.pending circle {
            fill: #9e9e9e;
            stroke: #757575;
        }

        .state-node.running rect,
        .state-node.running circle {
            fill: #2196f3;
            stroke: #1976d2;
        }

        .state-node.blocked rect,
        .state-node.blocked circle {
            fill: #ff9800;
            stroke: #f57c00;
        }

        .state-node.completed rect,
        .state-node.completed circle {
            fill: #4caf50;
            stroke: #388e3c;
        }

        .state-node.failed rect,
        .state-node.failed circle {
            fill: #f44336;
            stroke: #d32f2f;
        }

        .state-node text {
            fill: white;
            font-size: 12px;
            font-weight: bold;
            text-anchor: middle;
            pointer-events: none;
        }

        .state-transition {
            fill: none;
            stroke: #999;
            stroke-width: 2;
            marker-end: url(#arrowhead);
        }

        .state-transition.animated {
            stroke-dasharray: 5, 5;
            animation: dash 1s linear infinite;
        }

        @keyframes dash {
            to {
                stroke-dashoffset: -10;
            }
        }

        .transition-label {
            font-size: 10px;
            fill: #666;
            text-anchor: middle;
        }

        .graph-legend {
            margin-top: 15px;
            padding: 15px;
            background: white;
            border-radius: 8px;
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
        }
    </style>
"#
        .to_string()
    }

    /// Generate header
    fn generate_header(&self) -> String {
        let stats = self.inspector.stats();
        format!(
            r#"    <header>
        <h1>🔍 async-inspect</h1>
        <p>X-ray vision for async Rust - {} tasks analyzed</p>
    </header>
"#,
            stats.total_tasks
        )
    }

    /// Generate statistics panel
    fn generate_stats_panel(&self) -> String {
        let stats = self.inspector.stats();
        let mut html = String::new();

        writeln!(html, "        <div class=\"stats-panel\">").unwrap();

        self.add_stat_card(&mut html, "Total Tasks", &stats.total_tasks.to_string());
        self.add_stat_card(&mut html, "Running", &stats.running_tasks.to_string());
        self.add_stat_card(&mut html, "Blocked", &stats.blocked_tasks.to_string());
        self.add_stat_card(&mut html, "Completed", &stats.completed_tasks.to_string());
        self.add_stat_card(&mut html, "Failed", &stats.failed_tasks.to_string());
        self.add_stat_card(&mut html, "Total Events", &stats.total_events.to_string());
        self.add_stat_card(
            &mut html,
            "Duration",
            &format!("{:.2}s", stats.timeline_duration.as_secs_f64()),
        );

        writeln!(html, "        </div>").unwrap();

        html
    }

    /// Add a stat card
    fn add_stat_card(&self, html: &mut String, label: &str, value: &str) {
        writeln!(html, "            <div class=\"stat-card\">").unwrap();
        writeln!(html, "                <div class=\"label\">{}</div>", label).unwrap();
        writeln!(html, "                <div class=\"value\">{}</div>", value).unwrap();
        writeln!(html, "            </div>").unwrap();
    }

    /// Generate interactive timeline visualization
    fn generate_timeline_viz(&self) -> String {
        let tasks = self.inspector.get_all_tasks();

        if tasks.is_empty() {
            return String::from(
                "        <div class=\"timeline-viz\"><p>No tasks to visualize</p></div>",
            );
        }

        let mut html = String::new();
        writeln!(html, "        <div class=\"timeline-viz\">").unwrap();
        writeln!(html, "            <h2>Concurrency Timeline</h2>").unwrap();
        writeln!(html, "            <div class=\"timeline-container\">").unwrap();

        // Generate SVG timeline
        html.push_str(&self.generate_svg_timeline(&tasks));

        writeln!(html, "            </div>").unwrap();

        // Legend
        writeln!(html, "            <div class=\"legend\">").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #4caf50;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Completed</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #2196f3;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Running</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #ff9800;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Blocked</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #f44336;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Failed</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #9e9e9e;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Pending</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "            </div>").unwrap();

        writeln!(html, "        </div>").unwrap();

        html
    }

    /// Generate SVG timeline
    fn generate_svg_timeline(&self, tasks: &[TaskInfo]) -> String {
        let mut svg = String::new();

        // Calculate time bounds
        let start_time = tasks
            .iter()
            .map(|t| t.created_at)
            .min()
            .unwrap_or_else(std::time::Instant::now);

        let end_time = tasks
            .iter()
            .map(|t| t.created_at + t.age())
            .max()
            .unwrap_or_else(std::time::Instant::now);

        let total_duration = end_time.duration_since(start_time);
        let total_ms = total_duration.as_millis() as f64;

        // SVG dimensions
        let width = 1200.0;
        let row_height = 40.0;
        let margin_left = 200.0;
        let timeline_width = width - margin_left - 50.0;
        let height = (tasks.len() as f64 * row_height) + 60.0;

        writeln!(svg, "<svg class=\"timeline-svg\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">", width, height).unwrap();

        // Time axis
        self.add_time_axis(&mut svg, margin_left, timeline_width, total_ms);

        // Task rows
        for (i, task) in tasks.iter().enumerate() {
            let y = 50.0 + (i as f64 * row_height);
            self.add_task_row(
                &mut svg,
                task,
                y,
                margin_left,
                timeline_width,
                start_time,
                total_ms,
            );
        }

        writeln!(svg, "</svg>").unwrap();

        svg
    }

    /// Add time axis to SVG
    fn add_time_axis(&self, svg: &mut String, margin_left: f64, width: f64, total_ms: f64) {
        // Time markers
        let num_markers = 10;
        for i in 0..=num_markers {
            let x = margin_left + (i as f64 / num_markers as f64) * width;
            let time_ms = (i as f64 / num_markers as f64) * total_ms;

            writeln!(svg, "  <line x1=\"{}\" y1=\"30\" x2=\"{}\" y2=\"35\" stroke=\"#999\" stroke-width=\"1\" />", x, x).unwrap();
            writeln!(svg, "  <text x=\"{}\" y=\"25\" text-anchor=\"middle\" font-size=\"10\" fill=\"#666\">{}ms</text>", x, time_ms as u64).unwrap();
        }

        // Axis line
        writeln!(
            svg,
            "  <line x1=\"{}\" y1=\"35\" x2=\"{}\" y2=\"35\" stroke=\"#333\" stroke-width=\"2\" />",
            margin_left,
            margin_left + width
        )
        .unwrap();
    }

    /// Add task row to SVG
    fn add_task_row(
        &self,
        svg: &mut String,
        task: &TaskInfo,
        y: f64,
        margin_left: f64,
        timeline_width: f64,
        start_time: std::time::Instant,
        total_ms: f64,
    ) {
        // Task name
        writeln!(svg, "  <text x=\"10\" y=\"{}\" font-size=\"12\" font-weight=\"bold\" fill=\"#333\">{}</text>", y + 5.0, task.name).unwrap();

        // Task bar
        let task_start = task.created_at.duration_since(start_time).as_millis() as f64;
        let task_duration = task.age().as_millis() as f64;

        let x = margin_left + (task_start / total_ms) * timeline_width;
        let bar_width = ((task_duration / total_ms) * timeline_width).max(2.0);

        let state_class = match task.state {
            TaskState::Completed => "completed",
            TaskState::Running => "running",
            TaskState::Blocked { .. } => "blocked",
            TaskState::Failed => "failed",
            TaskState::Pending => "pending",
        };

        writeln!(svg, "  <g class=\"task-row\" data-task-id=\"{}\">", task.id).unwrap();
        writeln!(svg, "    <rect class=\"task-bar {}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"25\" rx=\"3\" />",
            state_class, x, y - 12.0, bar_width).unwrap();
        writeln!(
            svg,
            "    <title>{}: {:.2}ms</title>",
            task.name, task_duration
        )
        .unwrap();
        writeln!(svg, "  </g>").unwrap();
    }

    /// Generate state machine graph visualization
    fn generate_state_machine_graph(&self) -> String {
        let mut html = String::new();
        writeln!(html, "        <div class=\"state-machine-graph\">").unwrap();
        writeln!(html, "            <h2>Task Relationship Graph</h2>").unwrap();
        writeln!(html, "            <p style=\"color: #666; margin-bottom: 15px;\">Hierarchical view of task dependencies and interactions</p>").unwrap();
        writeln!(
            html,
            "            <div class=\"graph-container\" id=\"state-graph\">"
        )
        .unwrap();

        // Generate SVG for state machine
        html.push_str(&self.generate_state_machine_svg());

        writeln!(html, "            </div>").unwrap();

        // Legend
        writeln!(html, "            <div class=\"graph-legend\">").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #9e9e9e;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Pending</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #2196f3;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Running</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #ff9800;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Blocked</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #4caf50;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Completed</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "                <div class=\"legend-item\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"legend-color\" style=\"background: #f44336;\"></div>"
        )
        .unwrap();
        writeln!(html, "                    <span>Failed</span>").unwrap();
        writeln!(html, "                </div>").unwrap();
        writeln!(html, "            </div>").unwrap();

        writeln!(html, "        </div>").unwrap();

        html
    }

    /// Generate SVG for state machine visualization (task relationship graph)
    fn generate_state_machine_svg(&self) -> String {
        use std::collections::{HashMap, HashSet};

        let mut svg = String::new();
        let tasks = self.inspector.get_all_tasks();

        if tasks.is_empty() {
            writeln!(svg, "<svg width=\"800\" height=\"400\"><text x=\"400\" y=\"200\" text-anchor=\"middle\" fill=\"#666\">No tasks to visualize</text></svg>").unwrap();
            return svg;
        }

        let width = 1000.0;
        let height = 600.0;

        writeln!(
            svg,
            "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
            width, height
        )
        .unwrap();

        // Define arrowhead markers for different relationship types
        writeln!(svg, "  <defs>").unwrap();
        writeln!(svg, "    <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"10\" refX=\"9\" refY=\"3\" orient=\"auto\">").unwrap();
        writeln!(
            svg,
            "      <polygon points=\"0 0, 10 3, 0 6\" fill=\"#999\" />"
        )
        .unwrap();
        writeln!(svg, "    </marker>").unwrap();
        writeln!(svg, "    <marker id=\"arrowhead-parent\" markerWidth=\"10\" markerHeight=\"10\" refX=\"9\" refY=\"3\" orient=\"auto\">").unwrap();
        writeln!(
            svg,
            "      <polygon points=\"0 0, 10 3, 0 6\" fill=\"#667eea\" />"
        )
        .unwrap();
        writeln!(svg, "    </marker>").unwrap();
        writeln!(svg, "    <marker id=\"arrowhead-blocked\" markerWidth=\"10\" markerHeight=\"10\" refX=\"9\" refY=\"3\" orient=\"auto\">").unwrap();
        writeln!(
            svg,
            "      <polygon points=\"0 0, 10 3, 0 6\" fill=\"#ff9800\" />"
        )
        .unwrap();
        writeln!(svg, "    </marker>").unwrap();
        writeln!(svg, "  </defs>").unwrap();

        // Build task hierarchy and relationships
        let mut parent_child: Vec<(crate::task::TaskId, crate::task::TaskId)> = Vec::new();
        let mut root_tasks: Vec<&TaskInfo> = Vec::new();

        for task in &tasks {
            if let Some(parent_id) = task.parent {
                parent_child.push((parent_id, task.id));
            } else {
                root_tasks.push(task);
            }
        }

        // Layout tasks in layers (hierarchical layout)
        let mut task_positions: HashMap<crate::task::TaskId, (f64, f64)> = HashMap::new();
        let mut layers: Vec<Vec<crate::task::TaskId>> = Vec::new();

        // Layer 0: root tasks
        if !root_tasks.is_empty() {
            layers.push(root_tasks.iter().map(|t| t.id).collect());
        } else {
            // If no root tasks, treat all as layer 0
            layers.push(tasks.iter().map(|t| t.id).collect());
        }

        // Build subsequent layers from parent-child relationships
        let mut processed: HashSet<crate::task::TaskId> = layers[0].iter().copied().collect();
        loop {
            let last_layer = layers.last().unwrap();
            let mut next_layer = Vec::new();

            for &parent_id in last_layer {
                for &(pid, cid) in &parent_child {
                    if pid == parent_id && !processed.contains(&cid) {
                        next_layer.push(cid);
                        processed.insert(cid);
                    }
                }
            }

            if next_layer.is_empty() {
                break;
            }
            layers.push(next_layer);
        }

        // Position tasks
        let layer_height = 120.0;
        let base_y = 80.0;

        for (layer_idx, layer) in layers.iter().enumerate() {
            let y = base_y + (layer_idx as f64 * layer_height);
            let layer_width = width - 100.0;
            let spacing = if layer.len() > 1 {
                layer_width / (layer.len() - 1) as f64
            } else {
                0.0
            };

            for (i, &task_id) in layer.iter().enumerate() {
                let x = if layer.len() == 1 {
                    width / 2.0
                } else {
                    50.0 + (i as f64 * spacing)
                };
                task_positions.insert(task_id, (x, y));
            }
        }

        // Draw parent-child relationships
        for &(parent_id, child_id) in &parent_child {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (
                task_positions.get(&parent_id),
                task_positions.get(&child_id),
            ) {
                writeln!(svg, "  <line class=\"state-transition\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#667eea\" stroke-width=\"2\" marker-end=\"url(#arrowhead-parent)\" stroke-dasharray=\"5,5\" />",
                    x1, y1 + 35.0, x2, y2 - 35.0).unwrap();

                // Add label
                let mid_x = (x1 + x2) / 2.0;
                let mid_y = (y1 + y2) / 2.0;
                writeln!(svg, "  <text x=\"{}\" y=\"{}\" class=\"transition-label\" fill=\"#667eea\">spawns</text>",
                    mid_x + 10.0, mid_y).unwrap();
            }
        }

        // Draw blocking relationships (tasks waiting on each other)
        // This would come from await points and blocked states
        for task in &tasks {
            if let TaskState::Blocked { ref await_point } = task.state {
                // Find if await_point references another task
                for other_task in &tasks {
                    if other_task.id != task.id && await_point.contains(&other_task.name) {
                        if let (Some(&(x1, y1)), Some(&(x2, y2))) = (
                            task_positions.get(&task.id),
                            task_positions.get(&other_task.id),
                        ) {
                            writeln!(svg, "  <path class=\"state-transition\" d=\"M {} {} Q {} {} {} {}\" stroke=\"#ff9800\" stroke-width=\"2\" marker-end=\"url(#arrowhead-blocked)\" />",
                                x1 + 30.0, y1, x1 + 50.0, (y1 + y2) / 2.0, x2 - 30.0, y2).unwrap();

                            writeln!(svg, "  <text x=\"{}\" y=\"{}\" class=\"transition-label\" fill=\"#ff9800\">waits for</text>",
                                x1 + 60.0, (y1 + y2) / 2.0).unwrap();
                        }
                    }
                }
            }
        }

        // Draw task nodes
        for task in &tasks {
            if let Some(&(x, y)) = task_positions.get(&task.id) {
                let state_class = match task.state {
                    TaskState::Pending => "pending",
                    TaskState::Running => "running",
                    TaskState::Blocked { .. } => "blocked",
                    TaskState::Completed => "completed",
                    TaskState::Failed => "failed",
                };

                // Draw rounded rectangle for task
                writeln!(
                    svg,
                    "  <g class=\"state-node {}\" data-task-id=\"{}\">",
                    state_class, task.id
                )
                .unwrap();
                writeln!(svg, "    <rect x=\"{}\" y=\"{}\" width=\"120\" height=\"70\" rx=\"10\" ry=\"10\" />", x - 60.0, y - 35.0).unwrap();

                // Task name (truncate if needed)
                let display_name = if task.name.len() > 12 {
                    format!("{}...", &task.name[..9])
                } else {
                    task.name.clone()
                };
                writeln!(
                    svg,
                    "    <text x=\"{}\" y=\"{}\" font-size=\"13\">{}</text>",
                    x,
                    y - 5.0,
                    display_name
                )
                .unwrap();

                // Task ID
                writeln!(svg, "    <text x=\"{}\" y=\"{}\" font-size=\"10\" fill=\"white\" opacity=\"0.8\">#{}</text>",
                    x, y + 12.0, task.id.as_u64()).unwrap();

                // State indicator
                let state_text = match task.state {
                    TaskState::Pending => "⏸ Pending",
                    TaskState::Running => "▶ Running",
                    TaskState::Blocked { .. } => "⏳ Blocked",
                    TaskState::Completed => "✓ Done",
                    TaskState::Failed => "✗ Failed",
                };
                writeln!(svg, "    <text x=\"{}\" y=\"{}\" font-size=\"9\" fill=\"white\" opacity=\"0.9\">{}</text>",
                    x, y + 25.0, state_text).unwrap();

                // Tooltip
                writeln!(
                    svg,
                    "    <title>{}\nState: {:?}\nPoll count: {}\nRuntime: {:.2}ms</title>",
                    task.name,
                    task.state,
                    task.poll_count,
                    task.total_run_time.as_millis()
                )
                .unwrap();
                writeln!(svg, "  </g>").unwrap();
            }
        }

        // Add legend
        let legend_y = height - 80.0;
        writeln!(svg, "  <text x=\"20\" y=\"{}\" font-size=\"14\" font-weight=\"bold\" fill=\"#333\">Relationships:</text>", legend_y).unwrap();
        writeln!(svg, "  <line x1=\"20\" y1=\"{}\" x2=\"80\" y2=\"{}\" stroke=\"#667eea\" stroke-width=\"2\" stroke-dasharray=\"5,5\" marker-end=\"url(#arrowhead-parent)\" />",
            legend_y + 15.0, legend_y + 15.0).unwrap();
        writeln!(
            svg,
            "  <text x=\"90\" y=\"{}\" font-size=\"12\" fill=\"#666\">Parent spawns child</text>",
            legend_y + 20.0
        )
        .unwrap();

        writeln!(svg, "  <line x1=\"20\" y1=\"{}\" x2=\"80\" y2=\"{}\" stroke=\"#ff9800\" stroke-width=\"2\" marker-end=\"url(#arrowhead-blocked)\" />",
            legend_y + 35.0, legend_y + 35.0).unwrap();
        writeln!(
            svg,
            "  <text x=\"90\" y=\"{}\" font-size=\"12\" fill=\"#666\">Task waits for</text>",
            legend_y + 40.0
        )
        .unwrap();

        writeln!(svg, "</svg>").unwrap();

        svg
    }

    /// Generate task list with details
    fn generate_task_list(&self) -> String {
        let tasks = self.inspector.get_all_tasks();
        let mut html = String::new();

        writeln!(html, "        <div class=\"task-list\">").unwrap();
        writeln!(html, "            <h2>Task Details</h2>").unwrap();

        for task in &tasks {
            html.push_str(&self.generate_task_item(task));
        }

        writeln!(html, "        </div>").unwrap();

        html
    }

    /// Generate a single task item
    fn generate_task_item(&self, task: &TaskInfo) -> String {
        let mut html = String::new();

        let (state_class, state_text) = match task.state {
            TaskState::Completed => ("completed", "Completed"),
            TaskState::Running => ("running", "Running"),
            TaskState::Blocked { .. } => ("blocked", "Blocked"),
            TaskState::Failed => ("failed", "Failed"),
            TaskState::Pending => ("pending", "Pending"),
        };

        writeln!(
            html,
            "            <div class=\"task-item\" data-task-id=\"{}\">",
            task.id
        )
        .unwrap();
        writeln!(html, "                <div class=\"task-header\">").unwrap();
        writeln!(
            html,
            "                    <div class=\"task-name\">{}</div>",
            task.name
        )
        .unwrap();
        writeln!(
            html,
            "                    <div class=\"task-state state-{}\">{}</div>",
            state_class, state_text
        )
        .unwrap();
        writeln!(html, "                </div>").unwrap();

        // Expandable details
        writeln!(html, "                <div class=\"task-details\">").unwrap();
        writeln!(html, "                    <div class=\"task-meta\">").unwrap();
        writeln!(html, "                        <div class=\"meta-item\">").unwrap();
        writeln!(
            html,
            "                            <div class=\"meta-label\">Task ID</div>"
        )
        .unwrap();
        writeln!(
            html,
            "                            <div class=\"meta-value\">{}</div>",
            task.id
        )
        .unwrap();
        writeln!(html, "                        </div>").unwrap();
        writeln!(html, "                        <div class=\"meta-item\">").unwrap();
        writeln!(
            html,
            "                            <div class=\"meta-label\">Age</div>"
        )
        .unwrap();
        writeln!(
            html,
            "                            <div class=\"meta-value\">{:.2}ms</div>",
            task.age().as_millis()
        )
        .unwrap();
        writeln!(html, "                        </div>").unwrap();
        writeln!(html, "                        <div class=\"meta-item\">").unwrap();
        writeln!(
            html,
            "                            <div class=\"meta-label\">Poll Count</div>"
        )
        .unwrap();
        writeln!(
            html,
            "                            <div class=\"meta-value\">{}</div>",
            task.poll_count
        )
        .unwrap();
        writeln!(html, "                        </div>").unwrap();
        writeln!(html, "                        <div class=\"meta-item\">").unwrap();
        writeln!(
            html,
            "                            <div class=\"meta-label\">Total Runtime</div>"
        )
        .unwrap();
        writeln!(
            html,
            "                            <div class=\"meta-value\">{:.2}ms</div>",
            task.total_run_time.as_millis()
        )
        .unwrap();
        writeln!(html, "                        </div>").unwrap();
        writeln!(html, "                    </div>").unwrap();

        // Events
        let events = self.inspector.get_task_events(task.id);
        if !events.is_empty() {
            writeln!(html, "                    <div class=\"events-section\">").unwrap();
            writeln!(
                html,
                "                        <h4>Events ({} total)</h4>",
                events.len()
            )
            .unwrap();
            for event in events.iter().take(10) {
                writeln!(html, "                        <div class=\"event-item\">").unwrap();
                writeln!(
                    html,
                    "                            <span class=\"event-time\">[{:.3}ms]</span> {}",
                    event.age().as_millis(),
                    event.kind
                )
                .unwrap();
                writeln!(html, "                        </div>").unwrap();
            }
            if events.len() > 10 {
                writeln!(html, "                        <div style=\"margin-top: 10px; color: #666; font-size: 0.85em;\">... and {} more events</div>", events.len() - 10).unwrap();
            }
            writeln!(html, "                    </div>").unwrap();
        }

        writeln!(html, "                </div>").unwrap();
        writeln!(html, "            </div>").unwrap();

        html
    }

    /// Generate JavaScript for interactivity
    fn generate_javascript(&self) -> String {
        String::from(
            r##"
    <script>
        // Task item click to expand/collapse
        document.querySelectorAll('.task-item').forEach(item => {
            item.addEventListener('click', (e) => {
                // Don't toggle if clicking on a link or button
                if (e.target.tagName === 'A' || e.target.tagName === 'BUTTON') {
                    return;
                }
                item.classList.toggle('expanded');
            });
        });

        // SVG task row click to highlight corresponding task item
        document.querySelectorAll('.task-row').forEach(row => {
            row.addEventListener('click', (e) => {
                const taskId = row.getAttribute('data-task-id');
                const taskItem = document.querySelector(`.task-item[data-task-id="${taskId}"]`);

                if (taskItem) {
                    // Scroll into view
                    taskItem.scrollIntoView({ behavior: 'smooth', block: 'center' });

                    // Expand
                    taskItem.classList.add('expanded');

                    // Flash highlight
                    taskItem.style.background = '#fff3cd';
                    setTimeout(() => {
                        taskItem.style.background = '';
                    }, 1000);
                }
            });
        });

        // Add smooth scrolling
        document.querySelectorAll('a[href^="#"]').forEach(anchor => {
            anchor.addEventListener('click', function (e) {
                e.preventDefault();
                const target = document.querySelector(this.getAttribute('href'));
                if (target) {
                    target.scrollIntoView({ behavior: 'smooth' });
                }
            });
        });
    </script>
"##,
        )
    }

    /// Save HTML report to file
    pub fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let html = self.generate_html();
        std::fs::write(path, html)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_generation() {
        let inspector = Inspector::new();
        inspector.register_task("test_task".to_string());

        let reporter = HtmlReporter::new(inspector);
        let html = reporter.generate_html();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("async-inspect"));
        assert!(html.contains("test_task"));
    }

    #[test]
    fn test_save_to_file() {
        let inspector = Inspector::new();
        let reporter = HtmlReporter::new(inspector);

        let temp_file = "/tmp/async_inspect_test.html";
        reporter.save_to_file(temp_file).unwrap();

        let content = std::fs::read_to_string(temp_file).unwrap();
        assert!(content.contains("<!DOCTYPE html>"));

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }
}
