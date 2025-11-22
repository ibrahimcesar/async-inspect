# Change Log

All notable changes to the async-inspect VS Code extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-22

### Added

**Core Features:**
- Real-time async task monitoring with automatic refresh
- Deadlock detection with circular dependency analysis
- Performance profiling and bottleneck identification
- Interactive timeline visualization
- Task dependency graph visualization
- CodeLens integration showing inline performance metrics

**UI Components:**
- 3 sidebar panels:
  - Tasks View: Tree display of all async tasks with color-coded states
  - Statistics View: Real-time metrics (running, blocked, completed, failed)
  - Deadlocks View: Detected deadlocks with detailed descriptions
- 2 webview panels:
  - Timeline Panel: Visual timeline of task execution over time
  - Graph Panel: Interactive task relationship and dependency graph

**Commands (9):**
- `Start Monitoring`: Begin tracking async tasks
- `Stop Monitoring`: Stop task tracking
- `Export Session`: Export task data to JSON/CSV
- `Clear Data`: Clear task history
- `Show Timeline`: Open timeline visualization
- `Show Graph`: Open dependency graph
- `Refresh Tasks`: Manually refresh task list
- `Jump to Task`: Navigate to task source code
- `Show Help`: Display extension documentation

**Configuration (8 settings):**
- `async-inspect.enabled`: Enable/disable extension
- `async-inspect.autoStart`: Auto-start monitoring on workspace open
- `async-inspect.showInlineStats`: Show CodeLens annotations
- `async-inspect.deadlockAlerts`: Enable deadlock notifications
- `async-inspect.performanceThreshold`: Performance warning threshold (ms)
- `async-inspect.refreshInterval`: Auto-refresh interval (ms)
- `async-inspect.cliPath`: Custom path to async-inspect CLI
- `async-inspect.features`: Enabled features array

**Integration:**
- Automatic activation on Rust files or Cargo.toml detection
- Spawns async-inspect CLI process
- Parses JSON output from CLI
- Maintains real-time task state
- Supports jump-to-definition for tasks

**Developer Experience:**
- Color-coded task states (running, blocked, completed, failed)
- Hover tooltips with detailed task information
- Performance warnings for slow operations
- Deadlock notifications with suggestions
- Keyboard shortcuts for common actions
- Status icons in activity bar

### Technical Details

- Built with TypeScript 5.x
- VS Code Engine: ^1.85.0
- Latest ESLint 9.x and TypeScript ESLint 8.x
- Compiled JavaScript output in `out/` directory
- Package size: ~28KB

### Requirements

- Visual Studio Code 1.85.0 or higher
- Rust toolchain (rustc, cargo)
- async-inspect CLI: `cargo install async-inspect`

### Known Issues

- Graph visualization requires running application
- Timeline shows recent history only (configurable via settings)
- CodeLens updates may have slight delay (controlled by refresh interval)

## [Unreleased]

### Planned for 0.2.0

**Enhanced Visualization:**
- Zoom and pan controls for graph view
- Export timeline/graph as PNG
- Minimap for large task graphs
- Timeline playback controls (pause, step, rewind)

**Search and Filtering:**
- Task search by name or ID
- Filter by state (running, blocked, completed, failed)
- Filter by duration range
- Regex search support

**Analysis Features:**
- Session comparison (diff two monitoring sessions)
- Historical trend analysis
- Performance regression detection
- Task grouping by module/crate

**Integration Improvements:**
- Rust Analyzer integration
- Problem panel integration for deadlocks
- Test explorer integration
- Debug adapter protocol support

**UI Enhancements:**
- Syntax highlighting in webviews
- Dark/light theme support optimization
- Customizable task colors
- Collapsible task groups in tree view

**Export Options:**
- Export to PNG/SVG
- Export to Markdown report
- Export to HTML report
- Flamegraph generation

### Planned for Future Releases

**Advanced Features:**
- Language server protocol integration
- Inline decorations (colored gutters)
- Hover tooltips with full task details
- Task statistics in status bar
- Custom task annotations
- Breakpoint-style task markers

**Performance:**
- Lazy loading for large task lists
- Virtual scrolling in tree views
- Worker thread for parsing JSON
- Incremental updates instead of full refresh

**Configuration:**
- Per-project settings
- Task filtering profiles
- Custom CodeLens templates
- Hotkey customization

---

## Version History

- **0.1.0** (2025-01-22) - Initial release

## Feedback

Found a bug or have a feature request? Please file an issue on [GitHub](https://github.com/ibrahimcesar/async-inspect/issues).

## License

MIT - See [LICENSE](LICENSE) for details
