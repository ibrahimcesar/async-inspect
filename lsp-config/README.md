# async-inspect LSP Configuration Guide

This directory contains configuration examples for using the async-inspect Language Server with various editors.

## Building the LSP Server

First, build the LSP server binary:

```bash
cargo build --release --features lsp --bin async-inspect-lsp
```

The binary will be located at `target/release/async-inspect-lsp`.

## Editor Configurations

### Neovim (nvim-lspconfig)

See [neovim.lua](./neovim.lua) for configuration.

### VSCode/VSCodium

Use the official async-inspect VS Code extension from the marketplace, or see [vscode-settings.json](./vscode-settings.json) for manual configuration.

### Emacs (lsp-mode)

See [emacs.el](./emacs.el) for configuration.

### Vim (vim-lsp)

See [vim.vim](./vim.vim) for configuration.

### Sublime Text (LSP)

See [sublime-lsp.json](./sublime-lsp.json) for configuration.

### Helix

See [helix.toml](./helix.toml) for configuration.

## Features

The async-inspect LSP provides:

- **Diagnostics**: Warnings for untracked async tasks and await points
- **Code Actions**: Quick-fix suggestions to add tracking
- **Hover**: Task statistics and performance metrics
- **Completion**: Auto-complete for async-inspect methods

## Troubleshooting

### LSP Server Not Starting

1. Verify the binary exists:
   ```bash
   ls -la target/release/async-inspect-lsp
   ```

2. Test the server manually:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/release/async-inspect-lsp
   ```

3. Check editor logs for LSP errors

### No Diagnostics Appearing

1. Ensure you're editing a Rust file (`.rs` extension)
2. Check that the file contains async code
3. Verify the LSP server is running (check editor status bar)

### Performance Issues

The LSP server is lightweight and should not impact editor performance. If you experience issues:

1. Check system resources with `top` or `htop`
2. Restart the LSP server
3. Report issues at: https://github.com/async-inspect/async-inspect/issues
