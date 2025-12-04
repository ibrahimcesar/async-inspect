#!/usr/bin/env bash
# Record asciinema demos for async-inspect
# Requires: asciinema (brew install asciinema)

set -e

DEMO_DIR="demos/asciinema"
mkdir -p "$DEMO_DIR"

echo "🎬 Recording async-inspect demos with asciinema"
echo ""

# Demo 1: Basic Usage
echo "📹 Demo 1: Basic async task tracking"
timeout 5 cargo run --example basic_inspection > /tmp/demo1.txt 2>&1 || true
asciinema rec "$DEMO_DIR/01-basic-tracking.cast" \
  --title "async-inspect: Basic Task Tracking" \
  --command "bash -c 'echo \"Running: cargo run --example basic_inspection\" && timeout 5 cargo run --example basic_inspection || echo \"Demo complete\"'" \
  --overwrite

# Demo 2: Deadlock Detection (non-interactive)
echo "📹 Demo 2: Deadlock detection"
asciinema rec "$DEMO_DIR/02-deadlock-detection.cast" \
  --title "async-inspect: Deadlock Detection" \
  --command "bash -c 'echo \"Running: cargo run --example deadlock_detection\" && timeout 5 cargo run --example deadlock_detection || echo \"Demo complete\"'" \
  --overwrite

# Demo 3: Performance Profiling
echo "📹 Demo 3: Performance profiling"
asciinema rec "$DEMO_DIR/03-performance-profiling.cast" \
  --title "async-inspect: Performance Profiling" \
  --command "bash -c 'echo \"Running: cargo run --example performance_profiling --features tokio\" && timeout 5 cargo run --example performance_profiling --features tokio || echo \"Demo complete\"'" \
  --overwrite

# Demo 4: CLI Help
echo "📹 Demo 4: CLI help command"
asciinema rec "$DEMO_DIR/04-cli-help.cast" \
  --title "async-inspect: CLI Help" \
  --command "bash -c 'cargo run --bin async-inspect -- --help'" \
  --overwrite

# Demo 5: Visualization Example
echo "📹 Demo 5: Task visualization"
asciinema rec "$DEMO_DIR/05-visualization.cast" \
  --title "async-inspect: Task Visualization" \
  --command "bash -c 'echo \"Running: cargo run --example visualization --features tokio\" && timeout 5 cargo run --example visualization --features tokio || echo \"Demo complete\"'" \
  --overwrite

echo ""
echo "✅ All demos recorded!"
echo ""
echo "📦 Demos saved to: $DEMO_DIR"
echo ""
echo "Demo files:"
ls -lh "$DEMO_DIR"/*.cast 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "To play a demo:"
echo "  asciinema play $DEMO_DIR/01-basic-tracking.cast"
echo ""
echo "To upload to asciinema.org:"
echo "  asciinema upload $DEMO_DIR/01-basic-tracking.cast"
echo ""
echo "To embed in documentation:"
echo "  Add the asciinema player script to your HTML and use the .cast file URL"
