# async-inspect Asciinema Demos

This directory contains terminal recordings demonstrating async-inspect features using [asciinema](https://asciinema.org/).

## Recordings

### 1. Basic Task Tracking
**File**: `01-basic-tracking.cast`
**Duration**: ~5 seconds
**Demo**: Basic async task inspection showing how async-inspect tracks task state
**Example**: `basic_inspection.rs`

```bash
asciinema play 01-basic-tracking.cast
```

### 2. Deadlock Detection
**File**: `02-deadlock-detection.cast`
**Duration**: ~5 seconds
**Demo**: Demonstrates async-inspect's ability to detect deadlocks in async code
**Example**: `deadlock_detection.rs`

```bash
asciinema play 02-deadlock-detection.cast
```

### 3. Performance Profiling
**File**: `03-performance-profiling.cast`
**Duration**: ~5 seconds
**Demo**: Shows performance profiling capabilities with timing and statistics
**Example**: `performance_profiling.rs`

```bash
asciinema play 03-performance-profiling.cast
```

### 4. CLI Help
**File**: `04-cli-help.cast`
**Duration**: ~2 seconds
**Demo**: Displays the async-inspect CLI tool help output showing available commands
**Command**: `cargo run --bin async-inspect -- --help`

```bash
asciinema play 04-cli-help.cast
```

### 5. Task Visualization
**File**: `05-visualization.cast`
**Duration**: ~5 seconds
**Demo**: Task hierarchy and relationship visualization
**Example**: `visualization.rs`

```bash
asciinema play 05-visualization.cast
```

## Playing Recordings

### Locally

To play any recording:

```bash
asciinema play demos/asciinema/01-basic-tracking.cast
```

### Upload to asciinema.org

To share recordings online:

```bash
asciinema upload demos/asciinema/01-basic-tracking.cast
```

This will give you a URL like `https://asciinema.org/a/xxxxx` that you can share.

### Embed in Documentation

To embed in HTML documentation:

```html
<!DOCTYPE html>
<html>
<head>
  <link rel="stylesheet" type="text/css" href="https://cdn.jsdelivr.net/npm/asciinema-player@3.7.0/dist/bundle/asciinema-player.min.css" />
</head>
<body>
  <div id="demo"></div>
  <script src="https://cdn.jsdelivr.net/npm/asciinema-player@3.7.0/dist/bundle/asciinema-player.min.js"></script>
  <script>
    AsciinemaPlayer.create('/path/to/01-basic-tracking.cast', document.getElementById('demo'));
  </script>
</body>
</html>
```

### Embed in Markdown (GitHub)

For GitHub READMEs or markdown files, convert to animated GIF:

```bash
# Install agg (asciinema gif generator)
brew install agg

# Convert to GIF
agg 01-basic-tracking.cast 01-basic-tracking.gif

# Then use in markdown
![Basic Tracking Demo](demos/asciinema/01-basic-tracking.gif)
```

## Recording New Demos

To record new demos or update existing ones:

```bash
./scripts/record-demos.sh
```

This will automatically record all demos with proper titles and metadata.

### Manual Recording

To record a specific demo manually:

```bash
asciinema rec demos/asciinema/my-demo.cast \
  --title "My Demo Title" \
  --command "cargo run --example my_example"
```

## Technical Details

- **Format**: Asciinema cast v3
- **Terminal**: 80x24 (default)
- **Recorded in**: Headless mode (no TTY)
- **Timeout**: 5 seconds per demo (automated recordings)

## Use Cases

These recordings are useful for:

1. **Documentation**: Embed in docs to show real usage
2. **README**: Include in GitHub README for visual appeal
3. **Blog Posts**: Share in blog posts about async-inspect
4. **Presentations**: Use in conference talks or demos
5. **Social Media**: Share on Twitter/X, Reddit, etc.

## Notes

- Recordings are in headless mode since they're automated
- Each demo is limited to 5 seconds to keep file sizes small
- The CLI help demo captures the full help output
- All demos use the `--overwrite` flag for easy re-recording

## Related Files

- **Recording Script**: `../../scripts/record-demos.sh`
- **Examples Directory**: `../../examples/`
- **Documentation**: `../../docs/`

---

**Generated**: 2025-11-23
**Tool**: asciinema 3.x
**License**: Same as async-inspect (MIT)
