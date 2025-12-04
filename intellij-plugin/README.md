# Async Inspect - IntelliJ IDEA Plugin

Real-time async task monitoring and visualization for Rust applications in IntelliJ IDEA and RustRover.

## Features

- **Real-time Task Monitoring**: Watch async tasks spawn, run, and complete in real-time
- **Interactive Timeline**: Visualize task execution on an interactive timeline chart
- **Task Hierarchy**: View parent-child relationships between tasks
- **Performance Metrics**: Track task counts, durations, and states
- **Event Log**: Stream of all task-related events
- **Export Data**: Save monitoring data to JSON for later analysis
- **Seamless Integration**: Works with the Rust plugin and RustRover

## Requirements

- IntelliJ IDEA 2023.3+ or RustRover
- Rust plugin for IntelliJ
- Rust project with `async-inspect` dependency

## Installation

### From JetBrains Marketplace (Coming Soon)

1. Open IntelliJ IDEA/RustRover
2. Go to `Settings` → `Plugins` → `Marketplace`
3. Search for "Async Inspect"
4. Click `Install`
5. Restart the IDE

### Manual Installation

1. Download the latest release from [GitHub Releases](https://github.com/async-inspect/async-inspect/releases)
2. Open IntelliJ IDEA/RustRover
3. Go to `Settings` → `Plugins` → ⚙️ → `Install Plugin from Disk`
4. Select the downloaded `.zip` file
5. Restart the IDE

### Build from Source

```bash
cd intellij-plugin
./gradlew buildPlugin
```

The plugin will be created in `build/distributions/`.

## Setup

### 1. Add async-inspect to Your Rust Project

Add to your `Cargo.toml`:

```toml
[dependencies]
async-inspect = { version = "0.1", features = ["dashboard"] }
tokio = { version = "1", features = ["full"] }
```

### 2. Instrument Your Code

```rust
use async_inspect::runtime::tokio::{spawn_tracked, InspectExt};
use async_inspect::dashboard::Dashboard;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start the dashboard server
    let dashboard = Dashboard::new(8080);
    let _handle = dashboard.start().await?;

    println!("Dashboard running at http://localhost:8080");

    // Use spawn_tracked instead of tokio::spawn
    spawn_tracked("my_task", async {
        println!("Task running!");
    });

    // Or use .inspect() on futures
    async_operation()
        .inspect("async_op")
        .await;

    Ok(())
}

async fn async_operation() {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
```

### 3. Run Your Application

```bash
cargo run
```

The dashboard server will start on `http://localhost:8080`.

## Usage

### Opening the Tool Window

1. Run your Rust application with async-inspect integration
2. In IntelliJ, go to `View` → `Tool Windows` → `Async Inspect`
3. The tool window will appear at the bottom of the IDE

### Connecting to Dashboard

**Option 1: Automatic Connection**
- The plugin will automatically connect to `localhost:8080` when you open the tool window

**Option 2: Manual Connection**
1. Click the "Start Monitoring" button in the toolbar
2. Or use `Tools` → `Async Inspect` → `Start Monitoring`

### Tool Window Components

#### Metrics Panel
Displays live counters:
- **Total Tasks**: Total number of spawned tasks
- **Running Tasks**: Currently executing tasks
- **Completed Tasks**: Successfully finished tasks
- **Failed Tasks**: Tasks that panicked or failed
- **Blocked Tasks**: Tasks waiting on resources

#### Timeline Chart
- Interactive visualization of task execution over time
- Color-coded by state:
  - **Blue**: Running
  - **Green**: Completed
  - **Red**: Failed
  - **Yellow**: Blocked
- Hover over bars to see task details
- Click to focus on a specific task

#### Task Table
- Sortable columns showing all tracked tasks
- Real-time updates as tasks change state
- Columns: ID, Name, State, Parent, Poll Count, Duration

#### Event Log
- Live stream of all task events
- Auto-scrolls to show latest events
- Limited to 1000 most recent events

### Toolbar Actions

- **Start Monitoring** (▶️): Connect to the dashboard
- **Stop Monitoring** (⏹️): Disconnect from the dashboard
- **Clear Tasks** (🗑️): Clear all tracked tasks from the view
- **Export Data** (💾): Save current task data to JSON
- **Settings** (⚙️): Configure connection settings

### Keyboard Shortcuts

- `Ctrl+Shift+A` → "Start Monitoring" to connect quickly
- `Ctrl+Shift+A` → "Stop Monitoring" to disconnect
- `Ctrl+Shift+A` → "Clear Tasks" to clear the view

## Configuration

### Connection Settings

Access via `Tools` → `Async Inspect` → `Settings`

- **Dashboard Host**: Hostname of the dashboard server (default: `localhost`)
- **Dashboard Port**: Port number of the dashboard server (default: `8080`)
- **Auto-connect on project open**: Automatically connect when opening a Rust project

### Custom Port

If your dashboard runs on a different port:

```rust
let dashboard = Dashboard::new(9090); // Custom port
dashboard.start().await?;
```

Then update the plugin settings to use port `9090`.

## Troubleshooting

### Plugin Won't Connect

1. **Check if dashboard is running**:
   ```bash
   curl http://localhost:8080/api/stats
   ```
   Should return JSON with task statistics.

2. **Check firewall**: Ensure port 8080 is not blocked

3. **Check logs**:
   - IntelliJ: `Help` → `Show Log in Finder/Explorer`
   - Look for `async-inspect` related errors

### No Tasks Appearing

1. **Verify instrumentation**: Ensure you're using `spawn_tracked()` or `.inspect()`
2. **Check connection status**: Should show "Connected" in green
3. **Try refreshing**: Use "Clear Tasks" then restart monitoring

### Performance Issues

- The dashboard uses WebSockets for efficiency
- Metrics update every 100ms
- If you have thousands of tasks, consider:
  - Filtering tasks in your application
  - Reducing the monitoring window
  - Using sampling instead of tracking every task

## Examples

See the [examples directory](../examples/) in the main repository:

- [`dashboard_demo.rs`](../examples/dashboard_demo.rs): Comprehensive dashboard demonstration
- [`basic_tracking.rs`](../examples/basic_tracking.rs): Simple task tracking
- [`hierarchy.rs`](../examples/hierarchy.rs): Parent-child task relationships

## Development

### Building the Plugin

```bash
cd intellij-plugin
./gradlew buildPlugin
```

### Running in Development Mode

```bash
./gradlew runIde
```

This will start a new IntelliJ instance with the plugin installed.

### Testing

```bash
./gradlew test
```

## Architecture

The plugin consists of several key components:

- **InspectorService**: Application-level service managing WebSocket connections
- **ProjectInspectorService**: Project-level service for per-project monitoring
- **AsyncInspectToolWindow**: Main UI component with metrics, table, and log
- **TimelinePanel**: Custom Swing component for timeline visualization
- **Actions**: Toolbar and menu actions for user interactions

### Communication Flow

```
Rust App (Dashboard Server)
         ↓ WebSocket
InspectorService (Connection Manager)
         ↓ Events
Tool Window Components (UI Updates)
```

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/async-inspect/async-inspect) for contribution guidelines.

## License

MIT OR Apache-2.0

## Support

- **Issues**: [GitHub Issues](https://github.com/async-inspect/async-inspect/issues)
- **Discussions**: [GitHub Discussions](https://github.com/async-inspect/async-inspect/discussions)
- **Documentation**: [Main Documentation](https://github.com/async-inspect/async-inspect)

## Changelog

### 0.1.0 (Initial Release)

- Real-time task monitoring via WebSocket
- Interactive timeline visualization
- Task table with sorting and filtering
- Event log with auto-scroll
- Export to JSON functionality
- Connection management UI
- Integration with Rust plugin

---

**Made with ❤️ for the Rust async ecosystem**
