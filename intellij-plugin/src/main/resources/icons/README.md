# Plugin Icons

This directory contains icons for the IntelliJ IDEA plugin.

## Required Icons

- `async-inspect-13x13.svg` - Tool window icon (13x13px)
- `start-13x13.svg` - Start monitoring action icon (13x13px)
- `stop-13x13.svg` - Stop monitoring action icon (13x13px)
- `clear-13x13.svg` - Clear tasks action icon (13x13px)
- `export-13x13.svg` - Export data action icon (13x13px)

## Icon Guidelines

- **Format**: SVG preferred for scalability
- **Size**: 13x13 pixels for toolbar icons
- **Colors**: Use IntelliJ's standard icon colors
- **Style**: Follow IntelliJ platform icon guidelines

## Temporary Solution

For development, the plugin uses IntelliJ's built-in `AllIcons.General.Settings` for the settings action.

Custom icons should be created before production release.

## Design Notes

The async-inspect logo should feature:
- Rust-related imagery (gears, cogs)
- Async/concurrency theme (parallel lines, threads)
- Inspector/monitoring theme (magnifying glass, chart)

Consider using colors from the Rust brand:
- Rust Orange: #CE422B
- Dark Gray: #1A1A1A
- Light Gray: #5C5C5C
