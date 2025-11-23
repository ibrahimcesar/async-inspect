# Interactive TUI Monitor

async-inspect includes a powerful Terminal User Interface (TUI) for real-time monitoring of async tasks, similar to `htop` for processes.

## Features

The TUI provides:

- **Real-time task monitoring** - See tasks as they spawn, run, and complete
- **Multiple view modes** - Switch between list and dependency graph views
- **Advanced filtering** - Filter by state and search by name/ID
- **Interactive search** - Type-to-search functionality
- **Mouse support** - Scroll wheel navigation
- **Data export** - Export data directly from the TUI
- **Customizable sorting** - Sort by ID, name, duration, state, or poll count

## Running the TUI

### From Code

```rust
use async_inspect::prelude::*;
use async_inspect::tui::run_tui;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Your async code here...

    // Launch the TUI
    run_tui(Inspector::global().clone())?;

    Ok(())
}
```

### Example

Run the example to see the TUI in action:

```bash
cargo run --example tui_monitor --features cli
```

## Keyboard Shortcuts

### Navigation & Control

| Key | Action |
|-----|--------|
| `q` | Quit the application |
| `h` or `?` | Toggle help screen |
| `↑`/`↓` | Navigate task list |
| `r` | Reset selection to top |

### View & Display

| Key | Action |
|-----|--------|
| `v` | Toggle view mode (List ↔ Dependency Graph) |
| `s` | Cycle sort mode (ID → Name → Duration → State → Polls) |
| `f` | Cycle filter mode (All → Running → Completed → Failed → Blocked) |

### Search & Export

| Key | Action |
|-----|--------|
| `/` | Activate search mode |
| `c` | Clear search query |
| `ESC` | Exit search mode (while searching) |
| `e` | Export data (JSON, CSV, Chrome Trace) |

### Mouse

| Action | Result |
|--------|--------|
| Scroll wheel up | Navigate up |
| Scroll wheel down | Navigate down |

## View Modes

### Task List View

The default view shows a sortable table of all tasks:

```
┌────────────────────────────────────────────────────────────────┐
│ Tasks (Sort: Duration | Filter: All) - 42 shown               │
├────────────────────────────────────────────────────────────────┤
│ ID       │ Name               │ State    │ Duration  │ Polls   │
├────────────────────────────────────────────────────────────────┤
│ #1       │ fetch_user_data    │ RUNNING  │ 234.5ms  │ 3      │
│ #2       │ process_request    │ BLOCKED  │ 156.2ms  │ 2      │
│ #3       │ cache_write        │ DONE     │ 45.3ms   │ 1      │
└────────────────────────────────────────────────────────────────┘
```

**Features:**
- Color-coded states (Blue=Running, Yellow=Blocked, Green=Done, Red=Failed)
- Sortable by any column
- Shows poll count and runtime
- Highlighted selection

### Dependency Graph View

Press `v` to switch to the dependency graph view, which shows parent-child task relationships:

```
┌────────────────────────────────────────────────────────────────┐
│ Dependency Graph - 42 tasks                                    │
├────────────────────────────────────────────────────────────────┤
│ ID       │ Task Tree                  │ State    │ Duration    │
├────────────────────────────────────────────────────────────────┤
│ #1       │ main_task                  │ DONE     │ 500.2ms    │
│ #2       │   └─ fetch_user_data       │ RUNNING  │ 234.5ms    │
│ #3       │     └─ http_get            │ DONE     │ 200.1ms    │
│ #4       │   └─ process_data          │ DONE     │ 100.3ms    │
└────────────────────────────────────────────────────────────────┘
```

**Features:**
- Hierarchical tree structure
- Shows parent-child relationships
- Indentation shows nesting level
- Same color coding as list view

## Search Functionality

Press `/` to activate search mode:

```
┌────────────────────────────────────────────────────────────────┐
│ Search                                                         │
├────────────────────────────────────────────────────────────────┤
│ Search: fetch█                                                 │
└────────────────────────────────────────────────────────────────┘
```

- Type to filter tasks in real-time
- Searches task names and IDs
- Press `Enter` to deactivate search (keeps filter)
- Press `ESC` to cancel and clear search
- Press `c` to clear search query

## Filtering & Sorting

### Filter Modes

Press `f` to cycle through filter modes:

1. **All** - Show all tasks
2. **Running** - Show only currently running tasks
3. **Completed** - Show only completed tasks
4. **Failed** - Show only failed tasks
5. **Blocked** - Show only blocked tasks

### Sort Modes

Press `s` to cycle through sort modes:

1. **ID** - Sort by task ID (ascending)
2. **Name** - Sort alphabetically by task name
3. **Duration** - Sort by total duration (slowest first)
4. **State** - Sort by task state
5. **PollCount** - Sort by poll count (most active first)

## Export Functionality

Press `e` to export current data to `tui_exports/` directory.

Exports include:
- **JSON** - Full task and event data
- **CSV** - Tasks and events in separate files
- **Chrome Trace** - Compatible with chrome://tracing

Files are timestamped, e.g.:
```
tui_exports/
├── export_1706024400.json
├── tasks_1706024400.csv
├── events_1706024400.csv
└── trace_1706024400.json
```

After exporting, you can:
```bash
# View in chrome://tracing
# 1. Open Chrome
# 2. Navigate to chrome://tracing
# 3. Click "Load" and select trace_*.json

# Analyze with jq
jq '.tasks | map(.duration_ms) | add / length' tui_exports/export_*.json

# Import to pandas
import pandas as pd
df = pd.read_csv('tui_exports/tasks_*.csv')
```

## Statistics Panel

The TUI displays real-time statistics at the top:

```
┌────────────────────────────────────────────────────────────────┐
│ Statistics                                                     │
├────────────────────────────────────────────────────────────────┤
│ Total: 142   Running: 8   Blocked: 2                          │
│ Completed: 130   Failed: 2   Events: 456                      │
│ Duration: 12.34s                                               │
└────────────────────────────────────────────────────────────────┘
```

## Help Screen

Press `h` or `?` to view the full help screen with all keyboard shortcuts and features.

## Performance

The TUI is designed for minimal overhead:

- **Update rate**: 100ms (configurable)
- **Memory**: ~1MB overhead
- **CPU**: <1% on typical workloads
- **Supported tasks**: Tested with 10,000+ concurrent tasks

## Customization

You can customize the update interval:

```rust
use async_inspect::tui::TuiApp;
use std::time::Duration;

let mut app = TuiApp::new(inspector);
app.set_update_interval(Duration::from_millis(50)); // Faster updates
```

## Tips & Tricks

1. **Performance Debugging**
   - Sort by Duration to find slow tasks
   - Sort by PollCount to find busy tasks
   - Use Blocked filter to find waiting tasks

2. **Hierarchy Analysis**
   - Switch to Dependency Graph view
   - Look for deep nesting (potential issue)
   - Identify orphaned tasks

3. **Search Patterns**
   - Search by ID: `#42`
   - Search by prefix: `fetch`
   - Search by pattern: `worker_`

4. **Export for Analysis**
   - Export before closing TUI
   - Use Chrome Trace for timeline view
   - Use CSV for statistical analysis

## Troubleshooting

### TUI Not Starting

**Error**: `Device not configured`

This error occurs when running in non-interactive mode (e.g., without a TTY). The TUI requires a real terminal to function.

**Solution**: Run in an interactive terminal, not via SSH without TTY or in background mode.

### Mouse Not Working

**Problem**: Mouse scroll doesn't navigate

**Solution**: Ensure your terminal supports mouse events. Most modern terminals (iTerm2, Windows Terminal, Alacritty) support this by default.

### Display Issues

**Problem**: Garbled or misaligned display

**Solution**:
- Ensure terminal supports UTF-8
- Resize terminal to at least 80x24
- Use a monospace font

## Next Steps

- [CLI Usage](./cli-usage.md) - Command-line interface
- [Visualization](./visualization.md) - Export formats and visualization
- [Examples](./examples.md) - More examples
