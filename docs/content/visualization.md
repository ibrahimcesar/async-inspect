# Visualization & Export Formats

async-inspect provides multiple ways to visualize and export your async execution data. This guide covers all available formats and how to use them effectively.

## Overview

| Format | Best For | Tools | Interactivity |
|--------|----------|-------|---------------|
| **JSON** | Programmatic analysis, data pipelines | jq, pandas, custom tools | ❌ |
| **CSV** | Spreadsheet analysis, statistics | Excel, Google Sheets | ❌ |
| **Chrome Trace** | Timeline visualization, performance | chrome://tracing, Perfetto | ✅ |
| **Flamegraph** | Performance hotspots, call stacks | Speedscope, inferno | ✅ |
| **HTML Report** | Shareable reports, documentation | Web browser | ✅ |

## JSON Export

### Basic Usage

```rust
use async_inspect::export::JsonExporter;
use async_inspect::prelude::*;

let inspector = Inspector::global();

// Export to file
JsonExporter::export_to_file(&inspector, "data.json")?;

// Or get as string
let json = JsonExporter::export_to_string(&inspector)?;
println!("{}", json);
```

### JSON Structure

The exported JSON contains three main sections:

```json
{
  "tasks": [
    {
      "id": 1,
      "name": "fetch_user_data",
      "state": "Completed",
      "created_at_ms": 12345,
      "duration_ms": 234.5,
      "poll_count": 3,
      "run_time_ms": 45.2,
      "parent_id": null
    }
  ],
  "events": [
    {
      "event_id": 0,
      "task_id": 1,
      "timestamp_ms": 12345,
      "kind": "TaskSpawned",
      "details": "name=fetch_user_data, parent=None, location=Some(...)"
    }
  ],
  "metadata": {
    "version": "0.0.1",
    "timestamp": "2025-01-23T10:30:00Z",
    "total_tasks": 10,
    "total_events": 45,
    "duration_ms": 567.8
  }
}
```

### Analysis with jq

```bash
# Pretty print
jq . data.json

# List all task names
jq '.tasks[].name' data.json

# Find slowest task
jq '.tasks | sort_by(.duration_ms) | reverse | .[0]' data.json

# Calculate average duration
jq '.tasks | map(.duration_ms) | add / length' data.json

# Filter tasks by name pattern
jq '.tasks[] | select(.name | contains("fetch"))' data.json

# Count events by kind
jq '.events | group_by(.kind) | map({kind: .[0].kind, count: length})' data.json
```

### Analysis with Python

```python
import pandas as pd
import json

# Load data
with open('data.json') as f:
    data = json.load(f)

# Create DataFrames
tasks = pd.DataFrame(data['tasks'])
events = pd.DataFrame(data['events'])

# Statistical summary
print(tasks.describe())

# Find bottlenecks (tasks with low efficiency)
tasks['efficiency'] = tasks['run_time_ms'] / tasks['duration_ms']
bottlenecks = tasks[tasks['efficiency'] < 0.3]
print(bottlenecks[['name', 'duration_ms', 'efficiency']])

# Event timeline analysis
events['timestamp_s'] = events['timestamp_ms'] / 1000
event_counts = events.groupby('kind').size()
print(event_counts)

# Plot duration distribution
tasks['duration_ms'].hist(bins=20)
plt.xlabel('Duration (ms)')
plt.ylabel('Count')
plt.title('Task Duration Distribution')
plt.show()
```

## CSV Export

### Basic Usage

```rust
use async_inspect::export::CsvExporter;
use async_inspect::prelude::*;

let inspector = Inspector::global();

// Export tasks
CsvExporter::export_tasks_to_file(&inspector, "tasks.csv")?;

// Export events
CsvExporter::export_events_to_file(&inspector, "events.csv")?;
```

### CSV Format

**tasks.csv** columns:
- `id`: Unique task identifier
- `name`: Task name (function name)
- `state`: Current state (Running, Blocked, Completed, etc.)
- `created_at_ms`: Creation timestamp
- `duration_ms`: Total duration
- `poll_count`: Number of times polled
- `run_time_ms`: Actual running time (not blocked)
- `parent_id`: Parent task ID (if spawned from another task)

**events.csv** columns:
- `event_id`: Event identifier
- `task_id`: Associated task ID
- `timestamp_ms`: Event timestamp
- `kind`: Event type (TaskSpawned, PollStarted, AwaitEnded, etc.)
- `details`: Additional event-specific information

### Analysis with Spreadsheets

1. Open `tasks.csv` in Excel/Google Sheets
2. Create pivot tables for analysis:
   - Average duration by task name
   - Poll count distribution
   - Tasks by state
3. Add calculated columns:
   - `efficiency = run_time_ms / duration_ms`
   - `avg_poll_time = duration_ms / poll_count`
4. Create visualizations:
   - Duration histogram
   - Timeline chart
   - State distribution pie chart

## Chrome Trace Event Format

### Basic Usage

```rust
use async_inspect::export::ChromeTraceExporter;
use async_inspect::prelude::*;

let inspector = Inspector::global();

ChromeTraceExporter::export_to_file(&inspector, "trace.json")?;
```

### Viewing in Chrome DevTools

1. Open Chrome or Chromium
2. Navigate to `chrome://tracing`
3. Click "Load" button
4. Select `trace.json`
5. Explore the interactive timeline!

**Features:**
- Zoom in/out with mouse wheel
- Pan with WASD or click-and-drag
- Click events for details
- Search for specific tasks
- Measure time between events

### Viewing in Perfetto UI (Recommended)

Perfetto provides more advanced analysis features:

1. Go to [https://ui.perfetto.dev/](https://ui.perfetto.dev/)
2. Click "Open trace file"
3. Select `trace.json`

**Advanced Features:**
- **Thread view**: See tasks organized by thread
- **SQL queries**: Query trace data with SQL
  ```sql
  SELECT name, dur/1e6 as duration_ms
  FROM slice
  WHERE name LIKE '%fetch%'
  ORDER BY dur DESC
  LIMIT 10;
  ```
- **Metrics**: Built-in performance metrics
- **Custom tracks**: Create custom visualizations
- **Statistical summaries**: Percentiles, histograms

### Understanding the Timeline

The Chrome Trace format shows:

- **Complete events (X)**: Tasks and operations with duration
  - Blue bars: Task lifetime
  - Green bars: Poll operations (active work)
  - Yellow bars: Await operations (blocked/waiting)
- **Instant events (i)**: Point-in-time events
  - Task spawning
  - State changes
  - Inspection points
- **Metadata (M)**: Thread names and process info

**Reading the timeline:**
- Tasks appear as horizontal bars
- Longer bars = longer duration
- Stacked bars = nested operations
- Gaps = waiting/idle time

## Flamegraph Export

### Basic Usage

```rust
use async_inspect::export::{FlamegraphExporter, FlamegraphBuilder};
use async_inspect::prelude::*;

let inspector = Inspector::global();

// Simple export
FlamegraphExporter::export_to_file(&inspector, "flamegraph.txt")?;

// Customized export
FlamegraphBuilder::new()
    .include_polls(false)       // Hide poll events
    .include_awaits(true)        // Show await points
    .min_duration_ms(10)         // Filter operations < 10ms
    .export_to_file(&inspector, "flamegraph_filtered.txt")?;
```

### Viewing with Speedscope (Online)

1. Go to [https://www.speedscope.app/](https://www.speedscope.app/)
2. Drop `flamegraph.txt` onto the page
3. Explore the flamegraph!

**View modes:**
- **Time Order**: Shows execution over time
- **Left Heavy**: Optimizes for reading call stacks
- **Sandwich**: Shows both callers and callees

### Generating SVG with inferno

```bash
# Install inferno
cargo install inferno

# Generate SVG
cat flamegraph.txt | inferno-flamegraph > flamegraph.svg

# Open in browser
open flamegraph.svg  # macOS
xdg-open flamegraph.svg  # Linux
start flamegraph.svg  # Windows
```

### Generating SVG with Rust (Optional)

Enable the `flamegraph` feature:

```toml
[dependencies]
async-inspect = { version = "0.0.1", features = ["flamegraph"] }
```

```rust
#[cfg(feature = "flamegraph")]
{
    use async_inspect::export::FlamegraphExporter;
    FlamegraphExporter::generate_svg(&inspector, "flamegraph.svg")?;
}
```

### Reading Flamegraphs

**Anatomy of a flamegraph:**
- **Width**: Time spent (wider = more time)
- **Height**: Call stack depth (higher = deeper nesting)
- **Color**: Usually just for visual distinction

**How to use:**
1. **Find hotspots**: Look for wide plateaus
2. **Identify bottlenecks**: Wide sections indicate time-consuming operations
3. **Trace call paths**: Follow from bottom to top
4. **Compare tasks**: Different root nodes represent different tasks

**Common patterns:**
- **Wide base, narrow top**: Good - work is distributed
- **Narrow base, wide top**: Bottleneck in nested call
- **Uniform width**: Evenly distributed work
- **Many narrow spikes**: Fragmented work (may need batching)

## HTML Reports

### Basic Usage

```rust
use async_inspect::reporting::HtmlReporter;

let reporter = HtmlReporter::global();
reporter.save_to_file("report.html")?;
```

### Features

The HTML report includes:
- **Summary statistics**: Task counts, durations, success/failure rates
- **Interactive timeline**: Gantt-style visualization with hover details
- **Task list**: Sortable table of all tasks
- **Event log**: Complete event history
- **State machine view**: Visual representation of task states

### Sharing Reports

HTML reports are self-contained and can be:
- Emailed to team members
- Attached to bug reports
- Archived for historical analysis
- Opened without any installation

## Comprehensive Example

See [`examples/export_formats.rs`](https://github.com/ibrahimcesar/async-inspect/blob/main/examples/export_formats.rs) for a complete working example:

```bash
cargo run --example export_formats
```

This generates:
```
async_inspect_exports/
├── data.json                    # JSON export
├── tasks.csv                    # Task metrics
├── events.csv                   # Event timeline
├── trace.json                   # Chrome Trace Event Format
├── flamegraph.txt               # Flamegraph (folded stacks)
└── flamegraph_filtered.txt      # Filtered flamegraph
```

## Best Practices

### Choose the Right Format

- **Debugging a hang**: Chrome Trace Event Format (Perfetto)
- **Performance optimization**: Flamegraph
- **Statistical analysis**: CSV + pandas
- **Custom tooling**: JSON
- **Sharing with non-technical users**: HTML report

### Filtering Data

For large traces, filter before exporting:

```rust
// Only export long-running tasks
let long_tasks: Vec<_> = inspector.get_all_tasks()
    .into_iter()
    .filter(|t| t.age().as_secs() > 1)
    .collect();

// Create custom export data...
```

### Combining Formats

Use multiple formats for comprehensive analysis:

```rust
// Quick overview
HtmlReporter::global().save_to_file("report.html")?;

// Detailed timeline
ChromeTraceExporter::export_to_file(&inspector, "trace.json")?;

// Performance analysis
FlamegraphExporter::export_to_file(&inspector, "flame.txt")?;

// Data analysis
CsvExporter::export_tasks_to_file(&inspector, "tasks.csv")?;
```

## Troubleshooting

### Large Trace Files

If trace files are too large:

```rust
// Use filtered flamegraph
FlamegraphBuilder::new()
    .min_duration_ms(50)  // Only operations >= 50ms
    .export_to_file(&inspector, "flame.txt")?;

// Sample events for Chrome Trace
// (custom filtering before export)
```

### Chrome Trace Won't Load

- Check file size (Chrome has limits ~500MB)
- Try Perfetto UI instead (handles larger files)
- Use `jq` to validate JSON structure:
  ```bash
  jq . trace.json > /dev/null
  ```

### Flamegraph Looks Empty

- Check if you have any completed tasks
- Reduce `min_duration_ms` filter
- Ensure `include_awaits` is `true` for blocked operations

## Next Steps

- [Quickstart Guide](./quickstart.md) - Get started with async-inspect
- [Examples](./examples.md) - More usage examples
- [CLI Usage](./cli-usage.md) - Command-line interface
- [API Documentation](https://docs.rs/async-inspect) - Complete API reference
