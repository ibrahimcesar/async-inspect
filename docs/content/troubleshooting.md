# Troubleshooting

Common issues and solutions when using async-inspect.

## Installation Issues

### `cargo install async-inspect` fails

**Problem**: Build fails with compiler errors.

**Solution**:

1. Update Rust to latest stable:
   ```bash
   rustup update stable
   ```

2. Ensure minimum Rust version (1.70+):
   ```bash
   rustc --version
   ```

3. If issues persist, try clean install:
   ```bash
   cargo install async-inspect --force
   ```

### Feature compilation errors

**Problem**: Specific features fail to compile.

**Solution**:

Check feature dependencies:
```bash
# Install with specific features
cargo install async-inspect --features cli,tokio

# Or use --all-features
cargo install async-inspect --all-features
```

## Runtime Issues

### No tasks appear in monitoring

**Problem**: Running `async-inspect monitor` shows no tasks.

**Checklist**:

1. ✅ **Add tracing annotations**:
   ```rust
   #[async_inspect::trace]  // ← Add this!
   async fn my_function() { }
   ```

2. ✅ **Initialize inspector**:
   ```rust
   use async_inspect::Inspector;

   #[tokio::main]
   async fn main() {
       let inspector = Inspector::new(Default::default());
       // Your code
   }
   ```

3. ✅ **Actually run async code**:
   ```rust
   // This creates future but doesn't execute!
   my_function();

   // This executes
   my_function().await;
   ```

4. ✅ **Check sampling rate**:
   ```rust
   Config {
       sampling_rate: 1.0,  // 100% in development
       ..Default::default()
   }
   ```

### Tasks not updating

**Problem**: Task list is stale or frozen.

**Solutions**:

1. **Check refresh interval**:
   ```bash
   async-inspect monitor --refresh 100  # Update every 100ms
   ```

2. **Verify inspector is running**:
   ```rust
   println!("Task count: {}", inspector.task_count());
   ```

3. **Check for panics** in traced functions

### CLI hangs or freezes

**Problem**: CLI becomes unresponsive.

**Solutions**:

1. **Kill and restart**:
   ```bash
   pkill async-inspect
   async-inspect monitor
   ```

2. **Check for deadlocks** in your code:
   ```bash
   async-inspect analyze --deadlocks
   ```

3. **Reduce refresh rate**:
   ```bash
   async-inspect monitor --refresh 1000  # Every 1 second
   ```

## TUI Issues

### Colors not displaying

**Problem**: TUI shows garbled text or no colors.

**Solutions**:

1. **Check terminal support**:
   ```bash
   echo $TERM  # Should be xterm-256color or similar
   ```

2. **Force color mode**:
   ```bash
   COLORTERM=truecolor async-inspect tui
   ```

3. **Try alternative terminal**:
   - iTerm2 (macOS)
   - Windows Terminal (Windows)
   - Alacritty (cross-platform)

### TUI crashes on resize

**Problem**: Terminal crashes when resized.

**Solution**:

This is a known issue. Workaround:
```bash
# Restart TUI
async-inspect tui
```

Fix coming in next release.

### Keyboard shortcuts not working

**Problem**: TUI doesn't respond to key presses.

**Checklist**:

1. ✅ Focus is on TUI window
2. ✅ Not using SSH (some keys don't forward)
3. ✅ Try alternative keys:
   - `q` or `Ctrl+C` to quit
   - `↑↓` or `jk` for navigation
   - `r` or `F5` to refresh

## Performance Issues

### High memory usage

**Problem**: async-inspect consuming too much memory.

**Solutions**:

1. **Set memory limits**:
   ```rust
   Config {
       max_tasks: 1_000,      // Reduce from default 10k
       max_events: 10_000,    // Reduce from default 100k
       ..Default::default()
   }
   ```

2. **Enable cleanup**:
   ```rust
   Config {
       cleanup_interval: Duration::from_secs(30),
       task_retention: Duration::from_secs(60),
       ..Default::default()
   }
   ```

3. **Use sampling** in production:
   ```rust
   Config {
       sampling_rate: 0.01,  // Only track 1%
       ..Default::default()
   }
   ```

### High CPU usage

**Problem**: Performance overhead too high.

**Solutions**:

1. **Disable expensive features**:
   ```rust
   Config {
       capture_backtraces: false,
       track_allocations: false,
       ..Default::default()
   }
   ```

2. **Increase intervals**:
   ```bash
   async-inspect monitor --refresh 2000  # 2 seconds
   ```

3. **Use production mode**:
   ```rust
   Config {
       mode: Mode::Production,
       ..Default::default()
   }
   ```

### Application slowdown

**Problem**: App runs slower with tracing enabled.

**Benchmarks**:

| Configuration | Overhead |
|---------------|----------|
| No tracing | `0%` |
| Development mode (100% sampling) | `5-15%` |
| Analysis mode (10% sampling) | `2-5%` |
| Production mode (1% sampling) | `<1%` |

**Solutions**:

1. **Use conditional compilation**:
   ```rust
   #[cfg_attr(feature = "inspect", async_inspect::trace)]
   async fn hot_path() { }
   ```

2. **Selective tracing**:
   ```rust
   // Only trace important functions
   #[async_inspect::trace]
   async fn api_endpoint() { }

   // Don't trace tight loops
   async fn background_worker() {
       // No annotation
   }
   ```

## Export Issues

### JSON export incomplete

**Problem**: Exported JSON missing tasks.

**Causes**:

1. **Tasks cleaned up**: Increase retention
   ```rust
   Config {
       task_retention: Duration::from_secs(3600),  // 1 hour
       ..Default::default()
   }
   ```

2. **Sampling active**: Some tasks not tracked
   ```rust
   Config {
       sampling_rate: 1.0,  // Disable sampling for export
       ..Default::default()
   }
   ```

### CSV export fails

**Problem**: `async-inspect export --csv` errors.

**Solutions**:

1. **Check file permissions**:
   ```bash
   touch test.csv  # Can we write here?
   ```

2. **Use absolute path**:
   ```bash
   async-inspect export --csv /tmp/export.csv
   ```

3. **Check disk space**:
   ```bash
   df -h .
   ```

## Integration Issues

### Prometheus metrics not showing

**Problem**: Metrics endpoint returns empty.

**Checklist**:

1. ✅ **Feature enabled**:
   ```toml
   [dependencies]
   async-inspect = { version = "0.1", features = ["prometheus-export"] }
   ```

2. ✅ **Exporter started**:
   ```rust
   let exporter = PrometheusExporter::new(inspector.clone());
   exporter.start_server("0.0.0.0:9090").await?;
   ```

3. ✅ **Port accessible**:
   ```bash
   curl http://localhost:9090/metrics
   ```

4. ✅ **Tasks exist**:
   ```bash
   # Should show async_inspect_tasks_total
   curl http://localhost:9090/metrics | grep async_inspect
   ```

### OpenTelemetry not exporting

**Problem**: No traces in OpenTelemetry backend.

**Solutions**:

1. **Verify endpoint**:
   ```bash
   curl http://localhost:4317  # Should connect
   ```

2. **Check configuration**:
   ```rust
   OtelExporter::new(
       inspector.clone(),
       "http://localhost:4317",  // Correct endpoint?
   )?;
   ```

3. **Enable debug logging**:
   ```bash
   RUST_LOG=async_inspect=debug cargo run
   ```

### Tracing subscriber conflicts

**Problem**: Multiple subscriber initialization errors.

**Solution**:

Only initialize once:

```rust
use tracing_subscriber::prelude::*;

tracing_subscriber::registry()
    .with(async_inspect::integrations::AsyncInspectLayer::new(inspector.clone()))
    .with(tracing_subscriber::fmt::layer())
    .init();  // ← Only call once!
```

## VS Code Extension Issues

### Extension not loading

**Problem**: async-inspect extension doesn't appear.

**Solutions**:

1. **Check installation**:
   ```bash
   code --list-extensions | grep async-inspect
   ```

2. **Reinstall**:
   ```bash
   code --uninstall-extension async-inspect
   code --install-extension async-inspect
   ```

3. **Check VS Code version**:
   Requires VS Code 1.85.0 or higher

### No tasks in sidebar

**Problem**: Sidebar shows "No tasks".

**Checklist**:

1. ✅ **Monitoring started**: Click "Start Monitoring" button
2. ✅ **Rust workspace open**: Extension only activates in Rust projects
3. ✅ **async-inspect CLI installed**:
   ```bash
   async-inspect --version
   ```
4. ✅ **Code running**: Tasks only appear during execution

### CodeLens not showing

**Problem**: No inline performance stats.

**Solutions**:

1. **Enable in settings**:
   ```json
   {
     "async-inspect.showInlineStats": true
   }
   ```

2. **Check file type**: Only works in `.rs` files

3. **Restart extension**:
   - Cmd+Shift+P → "Reload Window"

## Compilation Errors

### `async_inspect::trace` macro not found

**Problem**: `#[async_inspect::trace]` causes compile error.

**Solution**:

Add dependency:
```toml
[dependencies]
async-inspect = "0.1"
async-inspect-macros = "0.1"  # ← Add this
```

Or use feature:
```toml
[dependencies]
async-inspect = { version = "0.1", features = ["macros"] }
```

### Lifetime errors with traced functions

**Problem**: Borrow checker errors after adding `#[trace]`.

**Example**:
```rust
#[async_inspect::trace]
async fn process(data: &str) -> String {
    data.to_string()
}
// ERROR: lifetime may not live long enough
```

**Solution**:

Add explicit lifetimes:
```rust
#[async_inspect::trace]
async fn process<'a>(data: &'a str) -> String {
    data.to_string()
}
```

### `Send` trait not satisfied

**Problem**: "future cannot be sent between threads safely".

**Cause**: Non-`Send` types held across `.await` points.

**Solution**:

1. **Identify the problem** using async-inspect:
   ```bash
   async-inspect analyze --check-send
   ```

2. **Fix by scoping**:
   ```rust
   // ❌ BAD
   async fn bad() {
       let rc = Rc::new(42);  // Not Send!
       other_async().await;   // ERROR
   }

   // ✅ GOOD
   async fn good() {
       let value = {
           let rc = Rc::new(42);
           *rc  // Drop Rc before .await
       };
       other_async().await;  // OK
   }
   ```

## Debug Logging

Enable verbose logging to diagnose issues:

```bash
# Linux/macOS
RUST_LOG=async_inspect=trace cargo run

# Windows
set RUST_LOG=async_inspect=trace
cargo run
```

Log levels:
- `error`: Critical issues only
- `warn`: Warnings and errors
- `info`: General information
- `debug`: Detailed debugging
- `trace`: Very verbose (everything)

## Getting Help

If your issue isn't covered here:

1. **Check GitHub Issues**: [async-inspect/issues](https://github.com/ibrahimcesar/async-inspect/issues)
2. **Ask in Discussions**: [async-inspect/discussions](https://github.com/ibrahimcesar/async-inspect/discussions)
3. **Provide details**:
   - OS and version
   - Rust version (`rustc --version`)
   - async-inspect version (`async-inspect --version`)
   - Minimal reproduction code
   - Full error message

## Common Error Messages

### "inspector not initialized"

**Cause**: Trying to use tracing before creating Inspector.

**Fix**:
```rust
let inspector = Inspector::new(Default::default());
// Now tracing will work
```

### "failed to spawn process"

**Cause**: CLI not found or not executable.

**Fix**:
```bash
which async-inspect  # Should show path
chmod +x $(which async-inspect)  # Make executable
```

### "address already in use"

**Cause**: Metrics/export port already taken.

**Fix**:
```rust
// Change port
exporter.start_server("0.0.0.0:9091").await?;  // Not 9090
```

## Performance Tuning

For production deployments, see [Production Guide](./production.md).

## Still Having Issues?

Create a detailed bug report:

```bash
# Generate diagnostic report
async-inspect diagnostic > report.txt

# Attach to GitHub issue
```

---

[Back to Documentation](./intro.md)
