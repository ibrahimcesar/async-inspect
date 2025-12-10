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

    /// Search query (supports glob patterns like "fetch_*" and duration filters like ">100")
    search_query: String,

    /// Whether search is active
    search_active: bool,

    /// Minimum duration filter in milliseconds (parsed from ">N" in search)
    min_duration_ms: Option<u64>,

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
            min_duration_ms: None,
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

        // Apply search filter with glob pattern support
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            // Check if it's a glob pattern (contains * or ?)
            let is_glob = query.contains('*') || query.contains('?');

            tasks.retain(|task| {
                let name_lower = task.name.to_lowercase();
                let id_str = format!("{}", task.id.as_u64());

                if is_glob {
                    // Simple glob matching: * matches any characters, ? matches single char
                    glob_match(&query, &name_lower) || glob_match(&query, &id_str)
                } else {
                    // Substring match (original behavior)
                    name_lower.contains(&query) || id_str.contains(&query)
                }
            });
        }

        // Apply duration filter (from >N syntax in search)
        if let Some(min_ms) = self.min_duration_ms {
            let min_duration = Duration::from_millis(min_ms);
            tasks.retain(|task| task.age() >= min_duration);
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

    /// Parse search query for special filters (e.g., ">100" for duration)
    fn parse_search_query(&mut self) {
        self.min_duration_ms = None;

        // Check for duration filter: >N (milliseconds)
        if self.search_query.starts_with('>') {
            if let Ok(ms) = self.search_query[1..].trim().parse::<u64>() {
                self.min_duration_ms = Some(ms);
                // Clear the search query since it's a filter, not a name search
                self.search_query.clear();
            }
        }
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

    /// Deactivate search mode and parse special filters
    fn deactivate_search(&mut self) {
        self.search_active = false;
        self.parse_search_query();
    }

    /// Clear search query and duration filter
    fn clear_search(&mut self) {
        self.search_query.clear();
        self.min_duration_ms = None;
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
            format!("{export_dir}/export_{timestamp}.json"),
        )?;

        CsvExporter::export_tasks_to_file(
            &self.inspector,
            format!("{export_dir}/tasks_{timestamp}.csv"),
        )?;

        CsvExporter::export_events_to_file(
            &self.inspector,
            format!("{export_dir}/events_{timestamp}.csv"),
        )?;

        ChromeTraceExporter::export_to_file(
            &self.inspector,
            format!("{export_dir}/trace_{timestamp}.json"),
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
                    } else if app.show_help {
                        // Any key closes help (as promised in the help screen)
                        app.show_help = false;
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
                                    eprintln!("Export failed: {e}");
                                }
                            }
                            // Navigation: vim-style j/k
                            KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
                            KeyCode::Down | KeyCode::Char('j') => {
                                let tasks = app.get_tasks();
                                app.select_next(tasks.len());
                            }
                            // Go to top/bottom (g and r both go to top)
                            KeyCode::Char('g' | 'r') => app.selected = 0,
                            KeyCode::Char('G') => {
                                let tasks = app.get_tasks();
                                app.selected = tasks.len().saturating_sub(1);
                            }
                            // Quick filter keys (1-5)
                            KeyCode::Char('1') => {
                                app.filter_mode = FilterMode::All;
                                app.selected = 0;
                            }
                            KeyCode::Char('2') => {
                                app.filter_mode = FilterMode::Running;
                                app.selected = 0;
                            }
                            KeyCode::Char('3') => {
                                app.filter_mode = FilterMode::Completed;
                                app.selected = 0;
                            }
                            KeyCode::Char('4') => {
                                app.filter_mode = FilterMode::Failed;
                                app.selected = 0;
                            }
                            KeyCode::Char('5') => {
                                app.filter_mode = FilterMode::Blocked;
                                app.selected = 0;
                            }
                            KeyCode::Esc => app.show_help = false,
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

    // Add search bar if active, has query, or has duration filter
    if app.search_active || !app.search_query.is_empty() || app.min_duration_ms.is_some() {
        constraints.push(Constraint::Length(3)); // Search bar
    }

    constraints.push(Constraint::Min(10)); // Main content
    constraints.push(Constraint::Length(3)); // Footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.area());

    let mut idx = 0;
    draw_header(f, chunks[idx], app);
    idx += 1;

    draw_stats(f, chunks[idx], app);
    idx += 1;

    if app.search_active || !app.search_query.is_empty() || app.min_duration_ms.is_some() {
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
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(table, area);
}

/// Draw search bar
fn draw_search_bar(f: &mut Frame, area: Rect, app: &TuiApp) {
    let mut filter_info = String::new();

    // Show active duration filter
    if let Some(min_ms) = app.min_duration_ms {
        filter_info = format!(" [Duration > {min_ms}ms]");
    }

    let search_text = if app.search_active {
        format!(
            "Search: {}█ (glob: fetch_* | duration: >100)",
            app.search_query
        )
    } else if !app.search_query.is_empty() || app.min_duration_ms.is_some() {
        format!(
            "Filter: {}{} (/ to edit, c to clear)",
            if app.search_query.is_empty() {
                String::new()
            } else {
                format!("\"{}\"", app.search_query)
            },
            filter_info
        )
    } else {
        "Press / to search (supports glob: fetch_* and duration: >100)".to_string()
    };

    let search = Paragraph::new(search_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search & Filter")
                .border_style(if app.search_active {
                    Style::default().fg(Color::Green)
                } else if app.min_duration_ms.is_some() || !app.search_query.is_empty() {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        )
        .style(Style::default().fg(if app.search_active {
            Color::Green
        } else if app.min_duration_ms.is_some() {
            Color::Yellow
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
    .row_highlight_style(Style::default().bg(Color::DarkGray));

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
        .filter(|t| t.parent.is_some_and(|p| p == task.id))
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
        Span::raw(format!(" View:{view_mode_str}  ")),
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

/// Draw help screen (keyboard shortcut reference card)
fn draw_help(f: &mut Frame) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  ⌨️  Keyboard Shortcuts Reference Card",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Navigation",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    ↑/k      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move selection up"),
        ]),
        Line::from(vec![
            Span::styled("    ↓/j      ", Style::default().fg(Color::Yellow)),
            Span::raw("Move selection down"),
        ]),
        Line::from(vec![
            Span::styled("    g        ", Style::default().fg(Color::Yellow)),
            Span::raw("Go to top"),
        ]),
        Line::from(vec![
            Span::styled("    G        ", Style::default().fg(Color::Yellow)),
            Span::raw("Go to bottom"),
        ]),
        Line::from(vec![
            Span::styled("    r        ", Style::default().fg(Color::Yellow)),
            Span::raw("Reset selection to top"),
        ]),
        Line::from(vec![
            Span::styled("    Mouse    ", Style::default().fg(Color::Yellow)),
            Span::raw("Scroll wheel to navigate"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Views & Sorting",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    v        ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle view (List ↔ Graph)"),
        ]),
        Line::from(vec![
            Span::styled("    s        ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle sort: ID→Name→Duration→State→Polls"),
        ]),
        Line::from(vec![
            Span::styled("    f        ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle filter: All→Running→Done→Failed→Blocked"),
        ]),
        Line::from(vec![
            Span::styled("    1-5      ", Style::default().fg(Color::Yellow)),
            Span::raw("Quick filter (1=All 2=Run 3=Done 4=Fail 5=Block)"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Search & Filter",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    /        ", Style::default().fg(Color::Yellow)),
            Span::raw("Start search (supports glob: fetch_*)"),
        ]),
        Line::from(vec![
            Span::styled("    Enter    ", Style::default().fg(Color::Yellow)),
            Span::raw("Confirm search"),
        ]),
        Line::from(vec![
            Span::styled("    Esc      ", Style::default().fg(Color::Yellow)),
            Span::raw("Cancel search / Close help"),
        ]),
        Line::from(vec![
            Span::styled("    c        ", Style::default().fg(Color::Yellow)),
            Span::raw("Clear search query"),
        ]),
        Line::from(vec![
            Span::styled("    >N       ", Style::default().fg(Color::Yellow)),
            Span::raw("Filter duration > N ms (e.g., >100)"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Actions",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    e        ", Style::default().fg(Color::Yellow)),
            Span::raw("Export data to tui_exports/"),
        ]),
        Line::from(vec![
            Span::styled("    d        ", Style::default().fg(Color::Yellow)),
            Span::raw("Show task details (when implemented)"),
        ]),
        Line::from(vec![
            Span::styled("    h/?      ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("    q        ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  ─────────────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  Press any key to close this help",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help - Keyboard Shortcuts ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default());

    // Center the help box
    let area = centered_rect(65, 85, f.area());
    f.render_widget(help, area);
}

/// Simple glob pattern matching
/// Supports * (matches any number of characters) and ? (matches single character)
fn glob_match(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                // Handle consecutive *s
                while pattern_chars.peek() == Some(&'*') {
                    pattern_chars.next();
                }

                // If * is at end, it matches everything
                if pattern_chars.peek().is_none() {
                    return true;
                }

                // Try matching rest of pattern at each position
                let remaining_pattern: String = pattern_chars.collect();
                while text_chars.peek().is_some() {
                    let remaining_text: String = text_chars.clone().collect();
                    if glob_match(&remaining_pattern, &remaining_text) {
                        return true;
                    }
                    text_chars.next();
                }
                // Also try matching with empty string
                let remaining_text: String = text_chars.collect();
                return glob_match(&remaining_pattern, &remaining_text);
            }
            '?' => {
                // ? matches exactly one character
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                // Literal character must match
                match text_chars.next() {
                    Some(t) if t == c => {}
                    _ => return false,
                }
            }
        }
    }

    // Pattern exhausted, text should also be exhausted
    text_chars.peek().is_none()
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
