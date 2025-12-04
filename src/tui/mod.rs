//! Terminal User Interface (TUI) for real-time async monitoring
//!
//! This module provides an interactive terminal dashboard for monitoring
//! async tasks in real-time, similar to htop for processes.

use crate::inspector::Inspector;
use crate::task::{TaskInfo, TaskState};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

/// Sort mode for task list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Sort by task ID
    Id,
    /// Sort by task name
    Name,
    /// Sort by duration (slowest first)
    Duration,
    /// Sort by state
    State,
    /// Sort by poll count
    PollCount,
}

/// Filter mode for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// Show all tasks
    All,
    /// Show only running tasks
    Running,
    /// Show only completed tasks
    Completed,
    /// Show only failed tasks
    Failed,
    /// Show only blocked tasks
    Blocked,
}

/// View mode for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Task list view
    TaskList,
    /// Dependency graph view
    DependencyGraph,
}

/// TUI application state
pub struct TuiApp {
    /// Inspector instance
    inspector: Inspector,

    /// Current view mode
    view_mode: ViewMode,

    /// Current sort mode
    sort_mode: SortMode,

    /// Current filter mode
    filter_mode: FilterMode,

    /// Selected task index
    selected: usize,

    /// Whether to show help
    show_help: bool,

    /// Search query
    search_query: String,

    /// Whether search is active
    search_active: bool,

    /// Last update time
    last_update: Instant,

    /// Update interval
    update_interval: Duration,
}

impl TuiApp {
    /// Create a new TUI application
    #[must_use]
    pub fn new(inspector: Inspector) -> Self {
        Self {
            inspector,
            view_mode: ViewMode::TaskList,
            sort_mode: SortMode::Duration,
            filter_mode: FilterMode::All,
            selected: 0,
            show_help: false,
            search_query: String::new(),
            search_active: false,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
        }
    }

    /// Set update interval
    pub fn set_update_interval(&mut self, interval: Duration) {
        self.update_interval = interval;
    }

    /// Get filtered and sorted tasks
    fn get_tasks(&self) -> Vec<TaskInfo> {
        let mut tasks = self.inspector.get_all_tasks();

        // Apply search filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            tasks.retain(|task| {
                task.name.to_lowercase().contains(&query)
                    || format!("{}", task.id.as_u64()).contains(&query)
            });
        }

        // Apply state filter
        tasks.retain(|task| match self.filter_mode {
            FilterMode::All => true,
            FilterMode::Running => matches!(task.state, TaskState::Running),
            FilterMode::Completed => matches!(task.state, TaskState::Completed),
            FilterMode::Failed => matches!(task.state, TaskState::Failed),
            FilterMode::Blocked => matches!(task.state, TaskState::Blocked { .. }),
        });

        // Apply sort
        match self.sort_mode {
            SortMode::Id => tasks.sort_by_key(|t| t.id.as_u64()),
            SortMode::Name => tasks.sort_by(|a, b| a.name.cmp(&b.name)),
            SortMode::Duration => tasks.sort_by(|a, b| b.age().cmp(&a.age())),
            SortMode::State => {
                tasks.sort_by(|a, b| format!("{:?}", a.state).cmp(&format!("{:?}", b.state)));
            }
            SortMode::PollCount => tasks.sort_by(|a, b| b.poll_count.cmp(&a.poll_count)),
        }

        tasks
    }

    /// Move selection up
    fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    fn select_next(&mut self, max: usize) {
        if self.selected < max.saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Cycle to next sort mode
    fn next_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Id => SortMode::Name,
            SortMode::Name => SortMode::Duration,
            SortMode::Duration => SortMode::State,
            SortMode::State => SortMode::PollCount,
            SortMode::PollCount => SortMode::Id,
        };
        self.selected = 0;
    }

    /// Cycle to next filter mode
    fn next_filter_mode(&mut self) {
        self.filter_mode = match self.filter_mode {
            FilterMode::All => FilterMode::Running,
            FilterMode::Running => FilterMode::Completed,
            FilterMode::Completed => FilterMode::Failed,
            FilterMode::Failed => FilterMode::Blocked,
            FilterMode::Blocked => FilterMode::All,
        };
        self.selected = 0;
    }

    /// Toggle help display
    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Toggle view mode
    fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::TaskList => ViewMode::DependencyGraph,
            ViewMode::DependencyGraph => ViewMode::TaskList,
        };
        self.selected = 0;
    }

    /// Activate search mode
    fn activate_search(&mut self) {
        self.search_active = true;
    }

    /// Deactivate search mode
    fn deactivate_search(&mut self) {
        self.search_active = false;
    }

    /// Clear search query
    fn clear_search(&mut self) {
        self.search_query.clear();
        self.selected = 0;
    }

    /// Add character to search query
    fn add_to_search(&mut self, c: char) {
        self.search_query.push(c);
        self.selected = 0;
    }

    /// Remove last character from search query
    fn backspace_search(&mut self) {
        self.search_query.pop();
        self.selected = 0;
    }

    /// Export data to file
    fn export_data(&mut self) -> io::Result<()> {
        use crate::export::{ChromeTraceExporter, CsvExporter, JsonExporter};
        use std::fs;

        // Create export directory
        let export_dir = "tui_exports";
        fs::create_dir_all(export_dir)?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Export to multiple formats
        JsonExporter::export_to_file(
            &self.inspector,
            &format!("{}/export_{}.json", export_dir, timestamp),
        )?;

        CsvExporter::export_tasks_to_file(
            &self.inspector,
            &format!("{}/tasks_{}.csv", export_dir, timestamp),
        )?;

        CsvExporter::export_events_to_file(
            &self.inspector,
            &format!("{}/events_{}.csv", export_dir, timestamp),
        )?;

        ChromeTraceExporter::export_to_file(
            &self.inspector,
            &format!("{}/trace_{}.json", export_dir, timestamp),
        )?;

        Ok(())
    }
}

/// Run the TUI application
pub fn run_tui(inspector: Inspector) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = TuiApp::new(inspector);

    // Run main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Main application loop
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut TuiApp,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Handle input with timeout
        if event::poll(app.update_interval)? {
            match event::read()? {
                Event::Key(key) => {
                    // Handle search mode separately
                    if app.search_active {
                        match key.code {
                            KeyCode::Esc => {
                                app.deactivate_search();
                                app.clear_search();
                            }
                            KeyCode::Enter => app.deactivate_search(),
                            KeyCode::Backspace => app.backspace_search(),
                            KeyCode::Char(c) => app.add_to_search(c),
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('h' | '?') => app.toggle_help(),
                            KeyCode::Char('s') => app.next_sort_mode(),
                            KeyCode::Char('f') => app.next_filter_mode(),
                            KeyCode::Char('v') => app.toggle_view_mode(),
                            KeyCode::Char('/') => app.activate_search(),
                            KeyCode::Char('c') => app.clear_search(),
                            KeyCode::Char('e') => {
                                if let Err(e) = app.export_data() {
                                    // Store error for display (we'll add a status bar later)
                                    eprintln!("Export failed: {}", e);
                                }
                            }
                            KeyCode::Up => app.select_previous(),
                            KeyCode::Down => {
                                let tasks = app.get_tasks();
                                app.select_next(tasks.len());
                            }
                            KeyCode::Char('r') => app.selected = 0, // Reset selection
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    handle_mouse_event(app, mouse);
                }
                _ => {}
            }
        }

        app.last_update = Instant::now();
    }
}

/// Handle mouse events
fn handle_mouse_event(app: &mut TuiApp, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            let tasks = app.get_tasks();
            app.select_next(tasks.len());
        }
        MouseEventKind::ScrollUp => {
            app.select_previous();
        }
        MouseEventKind::Down(_button) => {
            // Click support: calculate which row was clicked
            // This is a simplified version; precise calculation would need to track widget positions
            // For now, we support scroll wheel which is most useful
        }
        _ => {}
    }
}

/// Draw the UI
fn ui(f: &mut Frame, app: &mut TuiApp) {
    if app.show_help {
        draw_help(f);
        return;
    }

    // Create main layout
    let mut constraints = vec![
        Constraint::Length(3), // Header
        Constraint::Length(7), // Stats
    ];

    // Add search bar if active or has query
    if app.search_active || !app.search_query.is_empty() {
        constraints.push(Constraint::Length(3)); // Search bar
    }

    constraints.push(Constraint::Min(10)); // Main content
    constraints.push(Constraint::Length(3)); // Footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.size());

    let mut idx = 0;
    draw_header(f, chunks[idx], app);
    idx += 1;

    draw_stats(f, chunks[idx], app);
    idx += 1;

    if app.search_active || !app.search_query.is_empty() {
        draw_search_bar(f, chunks[idx], app);
        idx += 1;
    }

    // Render based on view mode
    match app.view_mode {
        ViewMode::TaskList => draw_tasks(f, chunks[idx], app),
        ViewMode::DependencyGraph => draw_dependency_graph(f, chunks[idx], app),
    }
    idx += 1;

    draw_footer(f, chunks[idx], app);
}

/// Draw header
fn draw_header(f: &mut Frame, area: Rect, _app: &TuiApp) {
    let title = vec![Line::from(vec![
        Span::styled(
            "async-inspect",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - Real-time Async Task Monitor"),
    ])];

    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::ALL).title("Dashboard"))
        .style(Style::default());

    f.render_widget(header, area);
}

/// Draw statistics panel
fn draw_stats(f: &mut Frame, area: Rect, app: &TuiApp) {
    let stats = app.inspector.stats();

    let stats_text = vec![
        Line::from(vec![
            Span::styled("Total: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.total_tasks),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Running: ", Style::default().fg(Color::Blue)),
            Span::styled(
                format!("{}", stats.running_tasks),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Blocked: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{}", stats.blocked_tasks),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Completed: ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{}", stats.completed_tasks),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Failed: ", Style::default().fg(Color::Red)),
            Span::styled(
                format!("{}", stats.failed_tasks),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("Events: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", stats.total_events),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:.2}s", stats.timeline_duration.as_secs_f64()),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];

    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Statistics"))
        .style(Style::default());

    f.render_widget(stats_widget, area);
}

/// Draw task list
fn draw_tasks(f: &mut Frame, area: Rect, app: &TuiApp) {
    let tasks = app.get_tasks();

    let rows: Vec<Row> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let state_color = match task.state {
                TaskState::Pending => Color::Gray,
                TaskState::Running => Color::Blue,
                TaskState::Blocked { .. } => Color::Yellow,
                TaskState::Completed => Color::Green,
                TaskState::Failed => Color::Red,
            };

            let state_str = match &task.state {
                TaskState::Pending => "PENDING",
                TaskState::Running => "RUNNING",
                TaskState::Blocked { .. } => "BLOCKED",
                TaskState::Completed => "DONE",
                TaskState::Failed => "FAILED",
            };

            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                format!("#{}", task.id.as_u64()),
                format!("{:.20}", task.name),
                state_str.to_string(),
                format!("{:.2}ms", task.age().as_secs_f64() * 1000.0),
                format!("{}", task.poll_count),
                format!("{:.2}ms", task.total_run_time.as_secs_f64() * 1000.0),
            ])
            .style(style)
            .fg(state_color)
        })
        .collect();

    let title = format!(
        "Tasks (Sort: {:?} | Filter: {:?}) - {} shown",
        app.sort_mode,
        app.filter_mode,
        tasks.len()
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),  // ID
            Constraint::Min(20),    // Name
            Constraint::Length(10), // State
            Constraint::Length(12), // Duration
            Constraint::Length(8),  // Polls
            Constraint::Length(12), // Run Time
        ],
    )
    .header(
        Row::new(vec!["ID", "Name", "State", "Duration", "Polls", "Run Time"])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(table, area);
}

/// Draw search bar
fn draw_search_bar(f: &mut Frame, area: Rect, app: &TuiApp) {
    let search_text = if app.search_active {
        format!("Search: {}█", app.search_query)
    } else {
        format!("Search: {} (Press / to edit, c to clear)", app.search_query)
    };

    let search = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(if app.search_active {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default()
                }),
        )
        .style(Style::default().fg(if app.search_active {
            Color::Green
        } else {
            Color::White
        }));

    f.render_widget(search, area);
}

/// Draw dependency graph view
fn draw_dependency_graph(f: &mut Frame, area: Rect, app: &TuiApp) {
    let tasks = app.get_tasks();

    // Build parent-child relationships
    let mut tree_lines = Vec::new();
    let mut root_tasks: Vec<_> = tasks.iter().filter(|t| t.parent.is_none()).collect();
    root_tasks.sort_by_key(|t| t.id.as_u64());

    for root in &root_tasks {
        build_tree_lines(&tasks, root, 0, &mut tree_lines);
    }

    let rows: Vec<Row> = tree_lines
        .iter()
        .enumerate()
        .map(|(i, (indent, task))| {
            let state_color = match task.state {
                TaskState::Pending => Color::Gray,
                TaskState::Running => Color::Blue,
                TaskState::Blocked { .. } => Color::Yellow,
                TaskState::Completed => Color::Green,
                TaskState::Failed => Color::Red,
            };

            let state_str = match &task.state {
                TaskState::Pending => "PENDING",
                TaskState::Running => "RUNNING",
                TaskState::Blocked { .. } => "BLOCKED",
                TaskState::Completed => "DONE",
                TaskState::Failed => "FAILED",
            };

            let tree_prefix = "  ".repeat(*indent);
            let tree_symbol = if *indent > 0 { "└─ " } else { "" };

            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                format!("#{}", task.id.as_u64()),
                format!("{}{}{}", tree_prefix, tree_symbol, task.name),
                state_str.to_string(),
                format!("{:.2}ms", task.age().as_secs_f64() * 1000.0),
            ])
            .style(style)
            .fg(state_color)
        })
        .collect();

    let title = format!("Dependency Graph - {} tasks", tasks.len());

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),  // ID
            Constraint::Min(30),    // Name (with tree)
            Constraint::Length(10), // State
            Constraint::Length(12), // Duration
        ],
    )
    .header(
        Row::new(vec!["ID", "Task Tree", "State", "Duration"])
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(table, area);
}

/// Helper to build tree lines recursively
fn build_tree_lines<'a>(
    all_tasks: &'a [TaskInfo],
    task: &'a TaskInfo,
    indent: usize,
    lines: &mut Vec<(usize, &'a TaskInfo)>,
) {
    lines.push((indent, task));

    // Find children
    let mut children: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.parent.map(|p| p == task.id).unwrap_or(false))
        .collect();
    children.sort_by_key(|t| t.id.as_u64());

    for child in children {
        build_tree_lines(all_tasks, child, indent + 1, lines);
    }
}

/// Draw footer with help hint
fn draw_footer(f: &mut Frame, area: Rect, app: &TuiApp) {
    let view_mode_str = match app.view_mode {
        ViewMode::TaskList => "List",
        ViewMode::DependencyGraph => "Graph",
    };

    let help_text = vec![Line::from(vec![
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit  "),
        Span::styled("[v]", Style::default().fg(Color::Yellow)),
        Span::raw(format!(" View:{}  ", view_mode_str)),
        Span::styled("[/]", Style::default().fg(Color::Yellow)),
        Span::raw(" Search  "),
        Span::styled("[e]", Style::default().fg(Color::Yellow)),
        Span::raw(" Export  "),
        Span::styled("[h/?]", Style::default().fg(Color::Yellow)),
        Span::raw(" Help"),
    ])];

    let footer = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    f.render_widget(footer, area);
}

/// Draw help screen
fn draw_help(f: &mut Frame) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Keyboard Shortcuts",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Navigation & Control:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(Color::Yellow)),
            Span::raw("           Quit the application"),
        ]),
        Line::from(vec![
            Span::styled("  h or ?", Style::default().fg(Color::Yellow)),
            Span::raw("      Toggle this help screen"),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw("         Navigate task list (or use mouse scroll)"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Yellow)),
            Span::raw("           Reset selection to top"),
        ]),
        Line::from(vec![
            Span::styled("  Mouse", Style::default().fg(Color::Yellow)),
            Span::raw("        Scroll wheel to navigate tasks"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  View & Display:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  v", Style::default().fg(Color::Yellow)),
            Span::raw("           Toggle view mode (List ↔ Dependency Graph)"),
        ]),
        Line::from(vec![
            Span::styled("  s", Style::default().fg(Color::Yellow)),
            Span::raw("           Cycle sort mode (ID → Name → Duration → State → Polls)"),
        ]),
        Line::from(vec![
            Span::styled("  f", Style::default().fg(Color::Yellow)),
            Span::raw(
                "           Cycle filter mode (All → Running → Completed → Failed → Blocked)",
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Search & Export:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("  /", Style::default().fg(Color::Yellow)),
            Span::raw("           Activate search mode (type to filter tasks)"),
        ]),
        Line::from(vec![
            Span::styled("  c", Style::default().fg(Color::Yellow)),
            Span::raw("           Clear search query"),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Yellow)),
            Span::raw("         Exit search mode (while searching)"),
        ]),
        Line::from(vec![
            Span::styled("  e", Style::default().fg(Color::Yellow)),
            Span::raw("           Export data (JSON, CSV, Chrome Trace to tui_exports/)"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  View Modes:",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("    • Task List: Standard sortable task list"),
        Line::from("    • Dependency Graph: Hierarchical tree showing parent-child relationships"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press h or ? to return",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default());

    // Center the help box
    let area = centered_rect(60, 80, f.size());
    f.render_widget(help, area);
}

/// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
