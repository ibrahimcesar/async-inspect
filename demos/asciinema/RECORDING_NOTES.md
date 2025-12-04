# Asciinema Recording Notes

## Successfully Recorded Demos ✅

All 5 interactive demos have been recorded with:
- ✅ Powerlevel10k prompt rendering correctly
- ✅ Character-by-character typing animation (70-80ms per keystroke)
- ✅ Natural pauses and timing
- ✅ Full command execution with output

### File Sizes

- `01-basic-tracking.cast` - 27KB (with P10k prompt + typing)
- `02-deadlock-detection.cast` - 19KB
- `03-cli-help.cast` - 12KB
- `04-visualization.cast` - 23KB
- `05-performance-profiling.cast` - 19KB

## Technical Implementation

### Key Features

1. **Interactive Shell**: Uses `zsh -i` to load full `.zshrc` configuration
2. **P10k Initialization**: 1.5-second delay to allow Powerlevel10k to fully render
3. **Log Control**: `log_user 0/1` to hide "spawn zsh" message
4. **Realistic Typing**: Character-by-character with 70-80ms delays
5. **Natural Flow**: Pauses before/after commands for human-like pacing

### Expect Script Pattern

```tcl
#!/usr/bin/expect -f
set timeout -1
log_user 0              # Hide spawn message

spawn zsh -i            # Interactive zsh with full config

sleep 1.5               # Wait for P10k to initialize
log_user 1              # Start logging

# Type command character by character
set command "cargo run --example basic_inspection"
for {set i 0} {$i < [string length $command]} {incr i} {
    send -- [string index $command $i]
    sleep 0.08          # 80ms per keystroke
}

sleep 0.3               # Pause before Enter
send "\r"               # Execute

set timeout 8           # Wait for output
expect {
    timeout { }
    eof { }
}

sleep 1.5               # Pause to view output
send -- "exit\r"        # Clean exit
expect eof
```

## Converting to GIF

### Install agg

```bash
brew install agg
```

### Convert Individual Demos

```bash
# Basic tracking
agg demos/asciinema/01-basic-tracking.cast demos/asciinema/01-basic-tracking.gif

# CLI help
agg demos/asciinema/03-cli-help.cast demos/asciinema/03-cli-help.gif

# Visualization
agg demos/asciinema/04-visualization.cast demos/asciinema/04-visualization.gif
```

### Batch Convert All

```bash
for cast in demos/asciinema/*.cast; do
    gif="${cast%.cast}.gif"
    agg "$cast" "$gif"
    echo "Converted: $gif"
done
```

### Custom Settings

```bash
# Slower speed (1.5x)
agg --speed 1.5 demo.cast demo.gif

# Custom size
agg --cols 100 --rows 30 demo.cast demo.gif

# Different theme
agg --theme monokai demo.cast demo.gif
```

## Playing Demos

### Locally

```bash
asciinema play demos/asciinema/01-basic-tracking.cast
```

### Upload to asciinema.org

```bash
asciinema upload demos/asciinema/01-basic-tracking.cast
```

You'll get a shareable URL like: `https://asciinema.org/a/xxxxx`

## Embedding in Documentation

### HTML (with asciinema player)

```html
<script src="https://cdn.jsdelivr.net/npm/asciinema-player@3.7.0/dist/bundle/asciinema-player.min.js"></script>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/asciinema-player@3.7.0/dist/bundle/asciinema-player.min.css">

<div id="demo"></div>
<script>
  AsciinemaPlayer.create(
    '/demos/asciinema/01-basic-tracking.cast',
    document.getElementById('demo'),
    {
      speed: 1.5,
      theme: 'dracula',
      loop: true
    }
  );
</script>
```

### Markdown (GitHub README)

```markdown
<!-- Using uploaded URL -->
[![Demo](https://asciinema.org/a/xxxxx.svg)](https://asciinema.org/a/xxxxx)

<!-- Using GIF -->
![Basic Tracking Demo](demos/asciinema/01-basic-tracking.gif)
```

### Docusaurus

```mdx
import AsciinemaPlayer from '@site/src/components/AsciinemaPlayer';

<AsciinemaPlayer src="/demos/asciinema/01-basic-tracking.cast" />
```

## Troubleshooting

### Fonts Not Rendering

If special characters don't show correctly:
1. Ensure terminal uses Nerd Fonts (MesloLGS NF for P10k)
2. Convert to GIF with proper font support

### Demo Too Fast/Slow

Adjust sleep timings in [record-interactive-demos.sh](../../scripts/record-interactive-demos.sh):
- Typing speed: Change `sleep 0.08` (currently 80ms)
- Pre-command pause: Change `sleep 1.5`
- Post-command pause: Change `sleep 1.5-2`

### P10k Not Loading

If prompt looks basic:
1. Verify `.zshrc` sources P10k
2. Increase initial sleep from 1.5s to 2-3s
3. Check `zsh -i` loads correctly

## Re-Recording

To re-record any demo:

```bash
./scripts/record-interactive-demos.sh
```

The script overwrites existing recordings automatically.

## Next Steps

1. ✅ Convert to GIFs for GitHub README
2. ✅ Upload to asciinema.org for sharing
3. ✅ Embed in documentation website
4. ✅ Include in release announcement
5. ✅ Share on social media (Twitter/X, Reddit, etc.)

---

**Generated**: 2025-11-23
**Tool**: asciinema 3.x + expect
**Shell**: zsh with Powerlevel10k
