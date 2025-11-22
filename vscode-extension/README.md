# async-inspect VS Code Extension

X-ray vision for async Rust - Debug async code with real-time task inspection directly in VS Code!

## Features

### 🔍 Real-Time Task Monitoring

View all active async tasks in the sidebar with live status updates:

- **Running tasks** (green) - Currently executing
- **Blocked tasks** (yellow) - Waiting for I/O or locks
- **Completed tasks** (blue) - Successfully finished
- **Failed tasks** (red) - Panicked or errored

### 📊 Inline Performance Stats

CodeLens integration shows performance metrics directly in your code:

```rust
// 🔍 45 calls | avg: 234.5ms | max: 567.2ms
async fn fetch_user(id: u64) -> User {
    // ...
}
```

### 📈 Interactive Timeline

Visualize async task execution over time with an interactive timeline view showing:
- Task start and end times
- Duration and overlap
- Parent-child relationships

### 🔗 Task Dependency Graph

See how your tasks relate to each other:
- Spawn relationships
- Channel communication
- Shared resource access
- Data flow

### 💀 Deadlock Detection

Get instant alerts when deadlocks are detected with:
- Involved tasks highlighted
- Dependency cycle visualization
- Jump-to-code functionality

### ⚡ Performance Warnings

Automatic notifications for:
- Slow tasks (configurable threshold)
- High contention points
- Memory leaks
- Polling bottlenecks

## Requirements

- VS Code 1.85.0 or higher
- Rust 1.70 or higher
- async-inspect CLI installed:

```bash
cargo install async-inspect
```

## Getting Started

1. **Install the extension** from VS Code Marketplace
2. **Open a Rust project** with async code
3. **Start monitoring**:
   - Click the async-inspect icon in the activity bar
   - Press `Cmd+Shift+P` → "Async-Inspect: Start Monitoring"
4. **Run your code** and see tasks appear in real-time!

## Commands

Access via Command Palette (`Cmd+Shift+P` / `Ctrl+Shift+P`):

- `Async-Inspect: Start Monitoring` - Begin tracking async tasks
- `Async-Inspect: Stop Monitoring` - Stop tracking
- `Async-Inspect: Show Timeline` - Open timeline visualization
- `Async-Inspect: Show Task Graph` - Display dependency graph
- `Async-Inspect: Analyze Deadlocks` - Check for deadlocks
- `Async-Inspect: Export Session` - Save traces to JSON/CSV
- `Async-Inspect: Clear History` - Reset all data

## Extension Settings

Configure via VS Code settings:

- `async-inspect.enabled` - Enable/disable extension
- `async-inspect.autoStart` - Start monitoring automatically
- `async-inspect.showInlineStats` - Show CodeLens statistics
- `async-inspect.deadlockAlerts` - Enable deadlock notifications
- `async-inspect.performanceThreshold` - Slow task threshold (ms)
- `async-inspect.refreshInterval` - UI update interval (ms)
- `async-inspect.cliPath` - Path to async-inspect binary

## Usage Examples

### Basic Monitoring

```rust
use async_inspect::prelude::*;

#[async_inspect::trace]
async fn fetch_user(id: u64) -> User {
    let profile = fetch_profile(id).await;
    let posts = fetch_posts(id).await;
    User { profile, posts }
}
```

Run your code and the extension will automatically show:
- When `fetch_user` starts
- How long `fetch_profile` takes
- If `fetch_posts` is blocked
- Total execution time

### Debugging Hangs

When a test hangs:

1. Extension shows blocked task in sidebar
2. Click task to jump to code
3. See exactly which `.await` is stuck
4. Check timeline to see task history

### Performance Analysis

CodeLens shows performance metrics:
- See which functions are slow
- Click to view detailed timeline
- Export data for offline analysis

## Tips

1. **Use with tokio-console** - Both tools complement each other
2. **Set appropriate thresholds** - Tune warning levels for your use case
3. **Export sessions** - Save problematic runs for later analysis
4. **Watch the timeline** - Spot patterns and bottlenecks visually

## Troubleshooting

### Extension not working?

1. Check async-inspect is installed: `async-inspect --version`
2. Ensure you're in a Rust workspace
3. Check Output panel (View → Output → Async-Inspect)

### No tasks appearing?

1. Make sure monitoring is started
2. Check your code has `#[async_inspect::trace]` annotations
3. Run your async code (tasks only appear when executed)

### Performance issues?

1. Increase `refreshInterval` setting
2. Disable inline stats if needed
3. Clear history periodically

## Known Issues

- Graph visualization is basic (D3.js integration coming soon)
- Timeline doesn't support zoom/pan yet
- Large task counts (>1000) may slow UI

## Release Notes

### 0.1.0

Initial release:
- Task tree view
- Statistics panel
- Deadlock detection
- CodeLens integration
- Timeline webview
- Graph visualization
- Export functionality

## Contributing

Found a bug? Have a feature request?

- GitHub: [async-inspect/issues](https://github.com/ibrahimcesar/async-inspect/issues)
- Discussions: [async-inspect/discussions](https://github.com/ibrahimcesar/async-inspect/discussions)

## License

MIT

---

**Enjoy debugging async Rust!** 🦀🔍
