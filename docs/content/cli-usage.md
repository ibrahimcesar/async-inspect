# CLI Usage

Complete reference for the async-inspect command-line interface.

## Overview

```bash
async-inspect [OPTIONS] [COMMAND]
```

## Commands

### `monitor` - Launch TUI

Start interactive terminal monitoring:

```bash
async-inspect monitor [OPTIONS]
```

**Options:**
- `--interval <MS>` - Update interval in milliseconds (default: 500)
- `--port <PORT>` - WebSocket port for remote connections (default: 9001)

**Example:**
```bash
async-inspect monitor --interval 1000
```

**Features:**
- Real-time task list
- Live statistics
- Task state transitions
- Keyboard shortcuts (press `?` for help)

### `export` - Export Data

Export task data to file:

```bash
async-inspect export [OPTIONS]
```

**Options:**
- `--format <FORMAT>` - Output format: json, csv (default: json)
- `--output <FILE>` - Output file path
- `--with-events` - Include event timeline (default: tasks only)

**Examples:**
```bash
# Export to JSON
async-inspect export --format json --output trace.json

# Export to CSV with events
async-inspect export --format csv --with-events --output tasks.csv

# Export to stdout
async-inspect export --format json
```

### `stats` - Show Statistics

Display current statistics:

```bash
async-inspect stats [OPTIONS]
```

**Options:**
- `--detailed` - Show per-task breakdown
- `--json` - Output as JSON

**Example:**
```bash
async-inspect stats --detailed
```

**Output:**
```
📊 Statistics:
  Total Tasks:     156
  Running:          12
  Blocked:           8
  Completed:       134
  Failed:            2

⏱️  Performance:
  Avg Duration:   234ms
  P95 Duration:   567ms
  P99 Duration:  1234ms

🔥 Hottest Tasks:
  1. fetch_posts    (45 calls, avg 567ms)
  2. db_query       (123 calls, avg 234ms)
  3. check_auth     (156 calls, avg 12ms)
```

### `config` - Configure Settings

Set production configuration:

```bash
async-inspect config <MODE>
```

**Modes:**
- `production` - Optimized for production (1% sampling, low overhead)
- `development` - Full instrumentation
- `debug` - Maximum verbosity
- `custom` - Custom configuration (prompts for values)

**Examples:**
```bash
# Set production mode
async-inspect config production

# Set development mode
async-inspect config development

# Custom configuration
async-inspect config custom
```

**Production Settings:**
```
Sampling Rate:  0.01 (1%)
Max Tasks:      1000
Max Events:     10000
Overhead:       <0.1%
```

### `info` - System Information

Show configuration and feature information:

```bash
async-inspect info
```

**Output:**
```
async-inspect 0.1.0

Configuration:
  Version:  0.1.0
  Features: cli, tokio, prometheus-export
  Build:    release

Runtime:
  OS:       macOS 14.0
  Arch:     aarch64
  Threads:  8

Documentation:
  Docs:     https://docs.rs/async-inspect
  GitHub:   https://github.com/ibrahimcesar/async-inspect
  Examples: Run `async-inspect examples`
```

### `version` - Show Version

Display version information:

```bash
async-inspect version
```

Or use shorthand:

```bash
async-inspect --version
async-inspect -V
```

## Global Options

### `--verbose` / `-v`

Enable verbose output:

```bash
async-inspect -v monitor
async-inspect --verbose stats
```

### `--help` / `-h`

Show help for any command:

```bash
async-inspect --help
async-inspect monitor --help
async-inspect export --help
```

## Configuration File

Create `~/.async-inspect/config.toml` for persistent settings:

```toml
[monitoring]
interval_ms = 500
max_tasks = 1000
sampling_rate = 1.0

[export]
default_format = "json"
include_events = true

[performance]
max_overhead_percent = 5
enable_profiling = true
```

## Environment Variables

### `ASYNC_INSPECT_CONFIG`

Path to custom config file:

```bash
export ASYNC_INSPECT_CONFIG=/path/to/config.toml
async-inspect monitor
```

### `ASYNC_INSPECT_LOG`

Set log level:

```bash
export ASYNC_INSPECT_LOG=debug
async-inspect monitor
```

Levels: `error`, `warn`, `info`, `debug`, `trace`

### `ASYNC_INSPECT_PORT`

Default monitoring port:

```bash
export ASYNC_INSPECT_PORT=9999
async-inspect monitor
```

## TUI Keyboard Shortcuts

When running `async-inspect monitor`:

| Key | Action |
|-----|--------|
| `?` | Show help |
| `q` | Quit |
| `r` | Refresh |
| `c` | Clear history |
| `e` | Export to file |
| `t` | Toggle timeline view |
| `g` | Toggle graph view |
| `s` | Toggle stats view |
| `f` | Filter tasks |
| `/` | Search |
| `↑`/`↓` | Navigate tasks |
| `Enter` | View task details |
| `Space` | Pause/resume |

## Output Formats

### JSON Export

```json
{
  "tasks": [
    {
      "id": 42,
      "name": "fetch_user",
      "state": "Completed",
      "duration_ms": 234.5,
      "poll_count": 12,
      "location": "src/api.rs:156"
    }
  ],
  "events": [
    {
      "task_id": 42,
      "timestamp_ms": 1234567890,
      "kind": "StateChanged",
      "details": {
        "old_state": "Running",
        "new_state": "Completed"
      }
    }
  ],
  "metadata": {
    "version": "0.1.0",
    "timestamp": "2024-11-20T10:30:00Z",
    "total_tasks": 156
  }
}
```

### CSV Export

**tasks.csv:**
```csv
id,name,state,duration_ms,poll_count,location
42,fetch_user,Completed,234.5,12,src/api.rs:156
43,db_query,Completed,123.4,8,src/db.rs:89
```

**events.csv:**
```csv
task_id,timestamp_ms,kind,details
42,1234567890,StateChanged,"Running -> Completed"
43,1234567891,AwaitStarted,"db_query"
```

## Advanced Usage

### Remote Monitoring

Monitor a remote application:

```bash
# On remote server
async-inspect monitor --port 9001

# On local machine
async-inspect connect localhost:9001
```

### Continuous Export

Export every 10 seconds:

```bash
while true; do
  async-inspect export --output "trace-$(date +%s).json"
  sleep 10
done
```

### Integration with CI/CD

```bash
# In GitHub Actions
- name: Run tests with async-inspect
  run: |
    async-inspect monitor &
    MONITOR_PID=$!
    cargo test
    kill $MONITOR_PID
    async-inspect export --output test-trace.json

- name: Upload trace
  uses: actions/upload-artifact@v3
  with:
    name: async-trace
    path: test-trace.json
```

## Troubleshooting

### Monitor Not Starting

```bash
# Check if port is in use
lsof -i :9001

# Use different port
async-inspect monitor --port 9002
```

### Export File Permissions

```bash
# Ensure write permissions
chmod 644 output.json
async-inspect export --output output.json
```

### High Overhead

```bash
# Reduce sampling
async-inspect config production

# Or set custom rate
ASYNC_INSPECT_SAMPLING=0.01 async-inspect monitor
```

---

[← Quick Start](./quickstart) | [Examples →](./examples)
