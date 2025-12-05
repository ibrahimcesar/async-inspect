# async-inspect Language Server Protocol (LSP) Guide

The async-inspect LSP server provides real-time diagnostics, code actions, and insights for async Rust code directly in your editor.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Features](#features)
- [Editor Configuration](#editor-configuration)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)

## Overview

The async-inspect LSP server enables any LSP-compatible editor to:

- **Detect untracked async tasks**: Warns when using `tokio::spawn` without tracking
- **Suggest tracking improvements**: Quick-fix actions to add `.inspect()` calls
- **Display real-time statistics**: Hover information showing current task metrics
- **Autocomplete tracking methods**: Intelligent completion for async-inspect APIs

### Supported Editors

- ✅ Neovim (nvim-lspconfig)
- ✅ Emacs (lsp-mode)
- ✅ Vim (vim-lsp)
- ✅ Helix
- ✅ Sublime Text
- ✅ VSCode (via official extension)
- ✅ Any LSP-compatible editor

## Installation

### 1. Build the LSP Server

```bash
cd /path/to/async-inspect
cargo build --release --features lsp --bin async-inspect-lsp
```

The binary will be located at `target/release/async-inspect-lsp`.

### 2. Verify Installation

```bash
./target/release/async-inspect-lsp --help
```

### 3. Configure Your Editor

See the [lsp-config directory on GitHub](https://github.com/ibrahimcesar/async-inspect/tree/main/lsp-config) for editor-specific configurations:

- [Neovim](https://github.com/ibrahimcesar/async-inspect/blob/main/lsp-config/neovim.lua)
- [Emacs](https://github.com/ibrahimcesar/async-inspect/blob/main/lsp-config/emacs.el)
- [Vim](https://github.com/ibrahimcesar/async-inspect/blob/main/lsp-config/vim.vim)
- [Helix](https://github.com/ibrahimcesar/async-inspect/blob/main/lsp-config/helix.toml)
- [Sublime Text](https://github.com/ibrahimcesar/async-inspect/blob/main/lsp-config/sublime-lsp.json)

## Features

### 1. Diagnostics

The LSP server analyzes your Rust code and provides diagnostics for:

#### Untracked `tokio::spawn` Calls

**Code:**
```rust
tokio::spawn(async {
    // Do work
});
```

**Diagnostic:**
```
Consider using spawn_tracked for better async debugging
Code: async-inspect-001
```

**Quick Fix:**
- Replace `tokio::spawn` with `spawn_tracked`

#### Missing `.inspect()` on Await Points

**Code:**
```rust
let result = fetch_data().await;
```

**Diagnostic:**
```
Add .inspect() to track this await point
Code: async-inspect-002
```

**Quick Fix:**
- Add `.inspect("await_point")` before `.await`

### 2. Code Actions

Press your editor's code action keybinding (typically `<leader>ca` or `Ctrl+.`) to:

- **Convert to `spawn_tracked`**: Automatically replace `tokio::spawn` with `spawn_tracked`
- **Add `.inspect()` tracking**: Insert `.inspect("label")` before `.await`

### 3. Hover Information

Hover over async code to see:

```markdown
## Async Inspect Statistics

- **Total Tasks:** 42
- **Running Tasks:** 8
- **Completed Tasks:** 30
- **Failed Tasks:** 2
- **Blocked Tasks:** 2

[Open Dashboard](http://localhost:8080)
```

### 4. Autocompletion

Type `.` after a future to get intelligent suggestions:

- `spawn_tracked(` - Spawn a tracked async task
- `inspect(` - Track an await point

## Editor Configuration

### Neovim (with nvim-lspconfig)

```lua
-- Add to ~/.config/nvim/init.lua or ~/.config/nvim/lua/lsp.lua

local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.async_inspect then
  configs.async_inspect = {
    default_config = {
      cmd = { '/path/to/async-inspect/target/release/async-inspect-lsp' },
      filetypes = { 'rust' },
      root_dir = lspconfig.util.root_pattern('Cargo.toml', '.git'),
    },
  }
end

lspconfig.async_inspect.setup{}
```

### Emacs (with lsp-mode)

```elisp
;; Add to ~/.emacs or ~/.emacs.d/init.el

(require 'lsp-mode)

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection '("/path/to/async-inspect/target/release/async-inspect-lsp"))
  :major-modes '(rust-mode rustic-mode)
  :server-id 'async-inspect))

(add-hook 'rust-mode-hook #'lsp-deferred)
```

### Helix

```toml
# Add to ~/.config/helix/languages.toml

[[language]]
name = "rust"
language-servers = ["rust-analyzer", "async-inspect-lsp"]

[language-server.async-inspect-lsp]
command = "/path/to/async-inspect/target/release/async-inspect-lsp"
```

## Usage

### Basic Workflow

1. **Open a Rust file** with async code in your LSP-enabled editor
2. **See diagnostics** appear automatically for untracked tasks
3. **Apply quick fixes** using your editor's code action command
4. **Hover over code** to see real-time async statistics

### Example Session

1. **Write async code:**

```rust
async fn process_data() {
    let data = fetch_data().await;
    tokio::spawn(async {
        process(data).await;
    });
}
```

2. **See diagnostics:**
   - Line 2: "Add .inspect() to track this await point"
   - Line 3: "Consider using spawn_tracked for better async debugging"

3. **Apply fixes:**

```rust
async fn process_data() {
    let data = fetch_data()
        .inspect("fetch_data")  // ← Added by quick fix
        .await;

    spawn_tracked("process", async {  // ← Converted by quick fix
        process(data).await;
    });
}
```

4. **Run with dashboard:**

```bash
cargo run --features dashboard
```

5. **Hover to see metrics** in your editor

## Troubleshooting

### LSP Server Not Starting

**Check if binary exists:**
```bash
ls -la target/release/async-inspect-lsp
```

**Test manually:**
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  ./target/release/async-inspect-lsp
```

**Expected output:** JSON response with server capabilities

### No Diagnostics Appearing

1. **Verify file type**: Ensure you're editing a `.rs` file
2. **Check LSP status**: Look for "async-inspect-lsp" in your editor's LSP status
3. **Restart LSP**: Use your editor's LSP restart command
4. **Check logs**: Enable LSP debug logging in your editor

### Code Actions Not Working

1. **Trigger manually**: Use your editor's code action keybinding
2. **Check cursor position**: Ensure cursor is on the diagnostic line
3. **Verify LSP capabilities**: Check server supports code actions

### Performance Issues

The LSP server is designed to be lightweight. If you experience issues:

1. **Check system resources**: Use `top` or `htop`
2. **Restart editor**: Sometimes helps clear cached state
3. **Report issue**: https://github.com/async-inspect/async-inspect/issues

## Architecture

### LSP Server Components

```
┌─────────────────────────────────────────┐
│       Editor (Client)                    │
│  ┌────────────────────────────────────┐ │
│  │  LSP Client                        │ │
│  │  - Send requests                   │ │
│  │  - Receive notifications           │ │
│  │  - Display diagnostics             │ │
│  └────────────────────────────────────┘ │
└──────────────┬──────────────────────────┘
               │ JSON-RPC over stdin/stdout
┌──────────────┴──────────────────────────┐
│       async-inspect-lsp                  │
│  ┌────────────────────────────────────┐ │
│  │  Language Server                   │ │
│  │  - Parse documents                 │ │
│  │  - Generate diagnostics            │ │
│  │  - Provide code actions            │ │
│  │  - Query Inspector stats           │ │
│  └────────────────────────────────────┘ │
└──────────────┬──────────────────────────┘
               │
┌──────────────┴──────────────────────────┐
│       Inspector (Global)                 │
│  - Task tracking                         │
│  - Statistics                            │
│  - Timeline events                       │
└─────────────────────────────────────────┘
```

### Communication Protocol

The LSP server implements the [Language Server Protocol specification](https://microsoft.github.io/language-server-protocol/):

1. **Initialization**: Client sends `initialize` request
2. **Document Sync**: Client notifies on file open/change/save
3. **Diagnostics**: Server publishes diagnostics for open documents
4. **Requests**: Client requests hover, code actions, completion

### Diagnostic Codes

- `async-inspect-001`: Untracked `tokio::spawn` call
- `async-inspect-002`: Missing `.inspect()` on await point

### Capabilities

The server advertises these capabilities:

- `textDocumentSync`: Full document synchronization
- `hoverProvider`: Hover information with statistics
- `codeActionProvider`: Quick-fix suggestions
- `completionProvider`: Autocomplete for async-inspect APIs

## Advanced Configuration

### Custom Diagnostic Severity

Some editors allow customizing diagnostic severity. You can configure:

- **Error**: Critical issues requiring immediate attention
- **Warning**: Recommendations for best practices
- **Info**: Helpful suggestions
- **Hint**: Non-intrusive improvements (default for async-inspect)

### Integration with rust-analyzer

The async-inspect LSP can run alongside rust-analyzer:

**Neovim:**
```lua
lspconfig.rust_analyzer.setup{}
lspconfig.async_inspect.setup{}
```

**Helix:**
```toml
[[language]]
name = "rust"
language-servers = ["rust-analyzer", "async-inspect-lsp"]
```

### Disabling Specific Diagnostics

If you want to disable certain diagnostics, you can use your editor's LSP configuration to filter by diagnostic code.

## Contributing

Contributions to improve the LSP server are welcome:

- **Report bugs**: https://github.com/ibrahimcesar/async-inspect/issues
- **Suggest features**: Open a discussion or issue
- **Submit PRs**: See [CONTRIBUTING.md](https://github.com/ibrahimcesar/async-inspect/blob/main/CONTRIBUTING.md)

## Resources

- [LSP Specification](https://microsoft.github.io/language-server-protocol/)
- [async-inspect Documentation](https://github.com/ibrahimcesar/async-inspect)
- [Editor Configurations](https://github.com/ibrahimcesar/async-inspect/tree/main/lsp-config)

---

**Made with ❤️ for the Rust async ecosystem**
