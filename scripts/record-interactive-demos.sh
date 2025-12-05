#!/usr/bin/env bash
# Record interactive asciinema demos with simulated human typing
# Requires: asciinema, expect (brew install expect)

set -e

DEMO_DIR="demos/asciinema"
mkdir -p "$DEMO_DIR"

echo "🎬 Recording interactive async-inspect demos"
echo ""

# Function to create expect script for simulated typing
create_typing_script() {
    local command="$1"
    local output_file="$2"
    local delay="${3:-0.1}"  # Default 100ms between keystrokes

    cat > "$output_file" << 'EXPECT_EOF'
#!/usr/bin/expect -f
set timeout -1
set delay DELAY_VALUE

# Start the shell
spawn bash --norc

# Wait for prompt
expect "$ "

# Type the command character by character
set command {COMMAND_VALUE}
foreach char [split $command ""] {
    send "$char"
    sleep $delay
}

# Press enter
send "\r"

# Wait for command to complete (timeout after 10 seconds)
set timeout 10
expect {
    "$ " { }
    timeout { }
    eof { }
}

# Give time to see the output
sleep 2

# Exit the shell
send "exit\r"
expect eof
EXPECT_EOF

    # Replace placeholders
    sed -i '' "s/DELAY_VALUE/$delay/" "$output_file"
    sed -i '' "s|COMMAND_VALUE|$command|" "$output_file"
    chmod +x "$output_file"
}

# Demo 1: Basic tracking with simulated typing
echo "📹 Demo 1: Basic async task tracking (interactive)"
cat > /tmp/demo1.expect << 'EOF'
#!/usr/bin/expect -f
set timeout -1
log_user 0

spawn zsh -i

# Wait for prompt to fully load (Powerlevel10k takes a moment)
sleep 1.5
log_user 1

# Type the command with realistic delays
set command "cargo run --example basic_inspection"
for {set i 0} {$i < [string length $command]} {incr i} {
    send -- [string index $command $i]
    sleep 0.08
}
sleep 0.3
send "\r"

# Wait for output
set timeout 8
expect {
    timeout { }
    eof { }
}

sleep 1.5
send -- "exit\r"
expect eof
EOF
chmod +x /tmp/demo1.expect

asciinema rec "$DEMO_DIR/01-basic-tracking.cast" \
  --title "async-inspect: Basic Task Tracking" \
  --command "/tmp/demo1.expect" \
  --overwrite

# Demo 2: Deadlock detection
echo "📹 Demo 2: Deadlock detection (interactive)"
cat > /tmp/demo2.expect << 'EOF'
#!/usr/bin/expect -f
set timeout -1
log_user 0

spawn zsh -i

sleep 1.5
log_user 1

set command "cargo run --example deadlock_detection"
for {set i 0} {$i < [string length $command]} {incr i} {
    send -- [string index $command $i]
    sleep 0.075
}
sleep 0.2
send "\r"

set timeout 8
expect {
    timeout { }
    eof { }
}

sleep 1.5
send -- "exit\r"
expect eof
EOF
chmod +x /tmp/demo2.expect

asciinema rec "$DEMO_DIR/02-deadlock-detection.cast" \
  --title "async-inspect: Deadlock Detection" \
  --command "/tmp/demo2.expect" \
  --overwrite

# Demo 3: CLI help with typing
echo "📹 Demo 3: CLI help (interactive)"
cat > /tmp/demo3.expect << 'EOF'
#!/usr/bin/expect -f
set timeout -1
log_user 0

spawn zsh -i

sleep 1.5
log_user 1

set command "cargo run --bin async-inspect -- --help"
for {set i 0} {$i < [string length $command]} {incr i} {
    send -- [string index $command $i]
    sleep 0.07
}
sleep 0.3
send "\r"

set timeout 5
expect {
    timeout { }
    eof { }
}

sleep 2
send -- "exit\r"
expect eof
EOF
chmod +x /tmp/demo3.expect

asciinema rec "$DEMO_DIR/03-cli-help.cast" \
  --title "async-inspect: CLI Help" \
  --command "/tmp/demo3.expect" \
  --overwrite

# Demo 4: Showing stats output
echo "📹 Demo 4: Task statistics (interactive)"
cat > /tmp/demo4.expect << 'EOF'
#!/usr/bin/expect -f
set timeout -1
log_user 0

spawn zsh -i

sleep 1.5
log_user 1

set command "cargo run --example visualization --features tokio"
for {set i 0} {$i < [string length $command]} {incr i} {
    send -- [string index $command $i]
    sleep 0.08
}
sleep 0.2
send "\r"

set timeout 8
expect {
    timeout { }
    eof { }
}

sleep 2
send -- "exit\r"
expect eof
EOF
chmod +x /tmp/demo4.expect

asciinema rec "$DEMO_DIR/04-visualization.cast" \
  --title "async-inspect: Task Visualization" \
  --command "/tmp/demo4.expect" \
  --overwrite

# Demo 5: Performance profiling
echo "📹 Demo 5: Performance profiling (interactive)"
cat > /tmp/demo5.expect << 'EOF'
#!/usr/bin/expect -f
set timeout -1
log_user 0

spawn zsh -i

sleep 1.5
log_user 1

set command "cargo run --example performance_profiling --features tokio"
for {set i 0} {$i < [string length $command]} {incr i} {
    send -- [string index $command $i]
    sleep 0.08
}
sleep 0.25
send "\r"

set timeout 8
expect {
    timeout { }
    eof { }
}

sleep 1.8
send -- "exit\r"
expect eof
EOF
chmod +x /tmp/demo5.expect

asciinema rec "$DEMO_DIR/05-performance-profiling.cast" \
  --title "async-inspect: Performance Profiling" \
  --command "/tmp/demo5.expect" \
  --overwrite

echo ""
echo "✅ All interactive demos recorded!"
echo ""
echo "📦 Demos saved to: $DEMO_DIR"
echo ""
echo "Demo files:"
ls -lh "$DEMO_DIR"/*.cast 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "To play a demo:"
echo "  asciinema play $DEMO_DIR/01-basic-tracking.cast"
echo ""
echo "To convert to GIF (requires agg):"
echo "  brew install agg"
echo "  agg $DEMO_DIR/01-basic-tracking.cast $DEMO_DIR/01-basic-tracking.gif"
echo ""
echo "To upload to asciinema.org:"
echo "  asciinema upload $DEMO_DIR/01-basic-tracking.cast"
