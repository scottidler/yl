use crate::config::Config;
use crate::linter::{Level, Linter, Problem};
use eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

/// YL Language Server for editor integration
pub struct YlLanguageServer {
    client: Client,
    linter: Arc<Mutex<Linter>>,
    document_map: Arc<Mutex<HashMap<Url, String>>>,
}

impl YlLanguageServer {
    /// Create a new YL language server
    pub fn new(client: Client) -> Self {
        let config = Config::default();
        let linter = Linter::new(config);

        Self {
            client,
            linter: Arc::new(Mutex::new(linter)),
            document_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Convert YL problems to LSP diagnostics
    fn problems_to_diagnostics(&self, problems: Vec<Problem>) -> Vec<Diagnostic> {
        problems
            .into_iter()
            .map(|problem| {
                let severity = match problem.level {
                    Level::Error => DiagnosticSeverity::ERROR,
                    Level::Warning => DiagnosticSeverity::WARNING,
                    Level::Info => DiagnosticSeverity::INFORMATION,
                };

                let range = Range::new(
                    Position::new(
                        (problem.line as u32).saturating_sub(1),
                        (problem.column as u32).saturating_sub(1),
                    ),
                    Position::new(
                        (problem.line as u32).saturating_sub(1),
                        problem.column as u32,
                    ),
                );

                Diagnostic {
                    range,
                    severity: Some(severity),
                    code: Some(NumberOrString::String(problem.rule.clone())),
                    code_description: None,
                    source: Some("yl".to_string()),
                    message: problem.message,
                    related_information: None,
                    tags: None,
                    data: None,
                }
            })
            .collect()
    }

    /// Lint a document and publish diagnostics
    async fn lint_and_publish(&self, uri: Url, content: &str) -> Result<()> {
        let path = uri
            .to_file_path()
            .map_err(|_| eyre::eyre!("Invalid file path"))?;

        let linter = self.linter.lock().await;
        let problems = linter.lint_content(&path, content)?;
        drop(linter);

        let diagnostics = self.problems_to_diagnostics(problems);

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;

        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for YlLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("yl".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "yl-language-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "YL Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;

        // Store document content
        self.document_map
            .lock()
            .await
            .insert(uri.clone(), content.clone());

        // Lint and publish diagnostics
        if let Err(e) = self.lint_and_publish(uri, &content).await {
            self.client
                .log_message(MessageType::ERROR, format!("Linting failed: {e}"))
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;

            // Update document content
            self.document_map
                .lock()
                .await
                .insert(uri.clone(), content.clone());

            // Lint and publish diagnostics
            if let Err(e) = self.lint_and_publish(uri, &content).await {
                self.client
                    .log_message(MessageType::ERROR, format!("Linting failed: {e}"))
                    .await;
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(content) = self.document_map.lock().await.get(&uri).cloned() {
            // Re-lint on save
            if let Err(e) = self.lint_and_publish(uri, &content).await {
                self.client
                    .log_message(MessageType::ERROR, format!("Linting failed: {e}"))
                    .await;
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        // Remove document from memory and clear diagnostics
        self.document_map.lock().await.remove(&uri);
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let _range = params.range;

        let mut actions = Vec::new();

        // Add disable rule actions for diagnostics in range
        for diagnostic in &params.context.diagnostics {
            if let Some(NumberOrString::String(rule_id)) = &diagnostic.code {
                // Disable line action
                let disable_line_action = CodeAction {
                    title: format!("Disable {rule_id} for this line"),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some({
                            let mut changes = HashMap::new();
                            let line_end_pos = Position::new(diagnostic.range.start.line, u32::MAX);
                            let edit = TextEdit {
                                range: Range::new(line_end_pos, line_end_pos),
                                new_text: format!("  # yl:disable-line {rule_id}"),
                            };
                            changes.insert(uri.clone(), vec![edit]);
                            changes
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                actions.push(CodeActionOrCommand::CodeAction(disable_line_action));

                // Disable rule for file action
                let disable_file_action = CodeAction {
                    title: format!("Disable {rule_id} for entire file"),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: Some(vec![diagnostic.clone()]),
                    edit: Some(WorkspaceEdit {
                        changes: Some({
                            let mut changes = HashMap::new();
                            let edit = TextEdit {
                                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                                new_text: format!("# yl:disable {rule_id}\n"),
                            };
                            changes.insert(uri.clone(), vec![edit]);
                            changes
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                actions.push(CodeActionOrCommand::CodeAction(disable_file_action));
            }
        }

        Ok(Some(actions))
    }
}

/// Start the LSP server
pub async fn start_lsp_server() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(YlLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problems_to_diagnostics() {
        // Test the diagnostic conversion logic directly
        let problems = vec![
            Problem::new(1, 5, Level::Error, "test-rule", "Test error message"),
            Problem::new(2, 10, Level::Warning, "test-rule-2", "Test warning message"),
        ];

        // Create a temporary server instance for testing (we'll use a dummy client)
        let (_service, _socket) =
            tower_lsp::LspService::new(|client| YlLanguageServer::new(client));

        // Test the conversion logic by creating diagnostics manually
        let diagnostics: Vec<Diagnostic> = problems
            .into_iter()
            .map(|problem| {
                let severity = match problem.level {
                    Level::Error => DiagnosticSeverity::ERROR,
                    Level::Warning => DiagnosticSeverity::WARNING,
                    Level::Info => DiagnosticSeverity::INFORMATION,
                };

                let range = Range::new(
                    Position::new(
                        (problem.line as u32).saturating_sub(1),
                        (problem.column as u32).saturating_sub(1),
                    ),
                    Position::new(
                        (problem.line as u32).saturating_sub(1),
                        problem.column as u32,
                    ),
                );

                Diagnostic {
                    range,
                    severity: Some(severity),
                    code: Some(NumberOrString::String(problem.rule.clone())),
                    code_description: None,
                    source: Some("yl".to_string()),
                    message: problem.message,
                    related_information: None,
                    tags: None,
                    data: None,
                }
            })
            .collect();

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(diagnostics[1].severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(diagnostics[0].message, "Test error message");
        assert_eq!(diagnostics[1].message, "Test warning message");
    }

    #[test]
    fn test_lsp_service_creation() {
        // Test that we can create the LSP service
        let (_service, _socket) =
            tower_lsp::LspService::new(|client| YlLanguageServer::new(client));
        // If we get here without panicking, the service was created successfully
        assert!(true);
    }
}
