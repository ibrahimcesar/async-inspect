//! Language Server Protocol (LSP) integration for async-inspect
//!
//! This module provides a Language Server Protocol server that enables async-inspect
//! functionality in any LSP-compatible editor (vim, emacs, Sublime Text, etc.).
//!
//! # Features
//!
//! - **Code Actions**: Quick-fix suggestions for adding async-inspect instrumentation
//! - **Diagnostics**: Warnings for potential async issues and performance problems
//! - **Hover Information**: Display task statistics and performance metrics
//! - **Completion**: Auto-complete for async-inspect methods
//!
//! # Example
//!
//! ```no_run
//! use async_inspect::lsp::AsyncInspectLanguageServer;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (service, socket) = tower_lsp::LspService::new(|client| {
//!         AsyncInspectLanguageServer::new(client)
//!     });
//!
//!     tower_lsp::Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
//!         .serve(service)
//!         .await;
//! }
//! ```

#[cfg(feature = "lsp")]
use crate::inspector::Inspector;

#[cfg(feature = "lsp")]
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[cfg(feature = "lsp")]
use tower_lsp::jsonrpc::Result;

#[cfg(feature = "lsp")]
use tower_lsp::lsp_types::{
    ClientCapabilities, CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams,
    CodeActionProviderCapability, CodeActionResponse, CompletionItem, CompletionItemKind,
    CompletionOptions, CompletionParams, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    Documentation, Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams,
    InitializeResult, InitializedParams, InsertTextFormat, MarkupContent, MarkupKind, MessageType,
    NumberOrString, Position, Range, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Url, WorkspaceEdit,
};

#[cfg(feature = "lsp")]
use tower_lsp::{Client, LanguageServer};

#[cfg(feature = "lsp")]
use std::sync::Arc;

#[cfg(feature = "lsp")]
use parking_lot::RwLock;

/// Language server for async-inspect
#[cfg(feature = "lsp")]
pub struct AsyncInspectLanguageServer {
    /// LSP client for sending notifications
    client: Client,

    /// Inspector instance
    inspector: &'static Inspector,

    /// Server state
    state: Arc<RwLock<ServerState>>,
}

#[cfg(feature = "lsp")]
struct ServerState {
    /// Workspace root URI
    workspace_root: Option<Url>,

    /// Client capabilities
    client_capabilities: Option<ClientCapabilities>,
}

#[cfg(feature = "lsp")]
impl AsyncInspectLanguageServer {
    /// Create a new language server instance
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            inspector: Inspector::global(),
            state: Arc::new(RwLock::new(ServerState {
                workspace_root: None,
                client_capabilities: None,
            })),
        }
    }

    /// Analyze document and provide diagnostics
    async fn analyze_document(&self, _uri: Url, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for untracked async tasks
        if text.contains("tokio::spawn") && !text.contains("spawn_tracked") {
            // Find all tokio::spawn occurrences
            for (line_num, line) in text.lines().enumerate() {
                if line.contains("tokio::spawn") && !line.contains("spawn_tracked") {
                    let col = line.find("tokio::spawn").unwrap_or(0);

                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: col as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: (col + "tokio::spawn".len()) as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::HINT),
                        code: Some(NumberOrString::String("async-inspect-001".to_string())),
                        source: Some("async-inspect".to_string()),
                        message: "Consider using spawn_tracked for better async debugging"
                            .to_string(),
                        related_information: None,
                        tags: None,
                        code_description: None,
                        data: None,
                    });
                }
            }
        }

        // Check for missing .inspect() on awaits
        if text.contains(".await") && text.contains("async fn") {
            for (line_num, line) in text.lines().enumerate() {
                if line.contains(".await") && !line.contains(".inspect(") {
                    let col = line.find(".await").unwrap_or(0);

                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: col as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: (col + ".await".len()) as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::INFORMATION),
                        code: Some(NumberOrString::String("async-inspect-002".to_string())),
                        source: Some("async-inspect".to_string()),
                        message: "Add .inspect() to track this await point".to_string(),
                        related_information: None,
                        tags: None,
                        code_description: None,
                        data: None,
                    });
                }
            }
        }

        diagnostics
    }

    /// Provide code actions for diagnostics
    fn provide_code_actions(&self, params: &CodeActionParams) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        for diagnostic in &params.context.diagnostics {
            if let Some(NumberOrString::String(code)) = &diagnostic.code {
                match code.as_str() {
                    "async-inspect-001" => {
                        // Convert tokio::spawn to spawn_tracked
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Use spawn_tracked instead".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(WorkspaceEdit {
                                changes: Some(
                                    [(
                                        params.text_document.uri.clone(),
                                        vec![TextEdit {
                                            range: diagnostic.range,
                                            new_text: "spawn_tracked".to_string(),
                                        }],
                                    )]
                                    .iter()
                                    .cloned()
                                    .collect(),
                                ),
                                document_changes: None,
                                change_annotations: None,
                            }),
                            command: None,
                            is_preferred: Some(true),
                            disabled: None,
                            data: None,
                        }));
                    }
                    "async-inspect-002" => {
                        // Add .inspect() before .await
                        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Add .inspect() tracking".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![diagnostic.clone()]),
                            edit: Some(WorkspaceEdit {
                                changes: Some(
                                    [(
                                        params.text_document.uri.clone(),
                                        vec![TextEdit {
                                            range: Range {
                                                start: diagnostic.range.start,
                                                end: diagnostic.range.start,
                                            },
                                            new_text: ".inspect(\"await_point\")".to_string(),
                                        }],
                                    )]
                                    .iter()
                                    .cloned()
                                    .collect(),
                                ),
                                document_changes: None,
                                change_annotations: None,
                            }),
                            command: None,
                            is_preferred: Some(true),
                            disabled: None,
                            data: None,
                        }));
                    }
                    _ => {}
                }
            }
        }

        actions
    }

    /// Provide hover information
    fn provide_hover(&self, _params: &HoverParams) -> Option<Hover> {
        let stats = self.inspector.stats();

        let markdown = format!(
            "## Async Inspect Statistics\n\n\
            - **Total Tasks:** {}\n\
            - **Running Tasks:** {}\n\
            - **Completed Tasks:** {}\n\
            - **Failed Tasks:** {}\n\
            - **Blocked Tasks:** {}\n\n\
            [Open Dashboard](http://localhost:8080)",
            stats.total_tasks,
            stats.running_tasks,
            stats.completed_tasks,
            stats.failed_tasks,
            stats.blocked_tasks
        );

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        })
    }
}

#[cfg(feature = "lsp")]
#[tower_lsp::async_trait]
impl LanguageServer for AsyncInspectLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        {
            let mut state = self.state.write();
            state.workspace_root = params.root_uri;
            state.client_capabilities = Some(params.capabilities);
        } // Drop the lock before await

        self.client
            .log_message(MessageType::INFO, "async-inspect LSP server initialized")
            .await;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "async-inspect-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "async-inspect LSP server ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        let diagnostics = self.analyze_document(uri.clone(), &text).await;

        self.client
            .publish_diagnostics(uri, diagnostics, Some(params.text_document.version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = &params.content_changes[0].text;

        let diagnostics = self.analyze_document(uri.clone(), text).await;

        self.client
            .publish_diagnostics(uri, diagnostics, Some(params.text_document.version))
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            let diagnostics = self
                .analyze_document(params.text_document.uri.clone(), &text)
                .await;

            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(self.provide_hover(&params))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let actions = self.provide_code_actions(&params);

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = vec![
            CompletionItem {
                label: "spawn_tracked".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Spawn a tracked async task".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Spawns an async task with automatic tracking by async-inspect.\n\n\
                            ```rust\n\
                            spawn_tracked(\"task_name\", async { /* ... */ });\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("spawn_tracked(\"${1:task_name}\", ${2:future})".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "inspect".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Track an await point".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Track an await point with a label.\n\n\
                            ```rust\n\
                            future.inspect(\"await_label\").await\n\
                            ```"
                    .to_string(),
                })),
                insert_text: Some("inspect(\"${1:label}\")".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ];

        Ok(Some(CompletionResponse::Array(items)))
    }
}

#[cfg(not(feature = "lsp"))]
compile_error!("The lsp module requires the 'lsp' feature to be enabled");
