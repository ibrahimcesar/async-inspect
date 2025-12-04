//! async-inspect Language Server
//!
//! This binary provides LSP (Language Server Protocol) support for async-inspect,
//! enabling integration with any LSP-compatible editor.
//!
//! # Usage
//!
//! The LSP server communicates via stdin/stdout following the LSP specification.
//!
//! ```bash
//! async-inspect-lsp
//! ```
//!
//! Configure your editor to use this binary as the language server for Rust files.

#[cfg(feature = "lsp")]
use async_inspect::lsp::AsyncInspectLanguageServer;

#[cfg(feature = "lsp")]
use tower_lsp::{LspService, Server};

#[cfg(feature = "lsp")]
#[tokio::main]
async fn main() {
    eprintln!("async-inspect LSP server starting...");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| AsyncInspectLanguageServer::new(client));

    eprintln!("LSP server listening on stdin/stdout");

    Server::new(stdin, stdout, socket).serve(service).await;

    eprintln!("async-inspect LSP server stopped");
}

#[cfg(not(feature = "lsp"))]
fn main() {
    eprintln!("Error: async-inspect-lsp requires the 'lsp' feature to be enabled");
    eprintln!("Build with: cargo build --features lsp --bin async-inspect-lsp");
    std::process::exit(1);
}
