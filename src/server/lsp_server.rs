//! Main LSP server implementation.

use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::info;

use crate::config::RezConfigProvider;
use crate::core::{ConfigProvider, PackageDiscovery as PackageDiscoveryTrait};
use crate::discovery::PackageDiscoveryImpl;

/// The main Rez Language Server.
pub struct RezLanguageServer {
    /// LSP client for communication
    client: Client,
    /// Document content cache
    document_map: tokio::sync::RwLock<HashMap<Url, String>>,
    /// Configuration provider
    config_provider: Arc<tokio::sync::RwLock<RezConfigProvider>>,
    /// Package discovery service
    package_discovery: Arc<tokio::sync::RwLock<Option<PackageDiscoveryImpl>>>,
}

impl RezLanguageServer {
    /// Create a new Rez Language Server instance.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: tokio::sync::RwLock::new(HashMap::new()),
            config_provider: Arc::new(tokio::sync::RwLock::new(RezConfigProvider::new())),
            package_discovery: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Initialize the server components.
    async fn initialize_components(&self) -> Result<()> {
        info!("Initializing Rez LSP server components");

        // Load configuration
        let mut config_provider = self.config_provider.write().await;
        if let Err(e) = config_provider.load_from_environment().await {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("Failed to load configuration: {}", e),
                )
                .await;
            return Ok(());
        }

        // Validate configuration
        if let Err(e) = config_provider.validate().await {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("Configuration validation failed: {}", e),
                )
                .await;
            return Ok(());
        }

        // Initialize package discovery
        let config = config_provider.config().clone();
        drop(config_provider); // Release the lock

        let mut discovery = PackageDiscoveryImpl::new(config);
        if let Err(e) = discovery.scan_packages().await {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("Failed to scan packages: {}", e),
                )
                .await;
        } else {
            let (families, total) = discovery.get_stats().await.unwrap_or((0, 0));
            self.client
                .log_message(
                    MessageType::INFO,
                    format!("Discovered {} package families ({} total packages)", families, total),
                )
                .await;
        }

        let mut package_discovery = self.package_discovery.write().await;
        *package_discovery = Some(discovery);

        Ok(())
    }

    /// Handle document changes.
    async fn on_change(&self, params: TextDocumentItem) {
        let mut document_map = self.document_map.write().await;
        document_map.insert(params.uri, params.text);
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for RezLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        info!("Rez LSP Server initializing...");

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "rez-lsp-server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "\"".to_string(),
                        "'".to_string(),
                        "-".to_string(),
                        ".".to_string(),
                    ]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("rez-lsp".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: Default::default(),
                    },
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Rez LSP Server initialized!");

        self.client
            .log_message(MessageType::INFO, "Rez LSP Server initialized")
            .await;

        // Initialize components in the background
        if let Err(e) = self.initialize_components().await {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("Failed to initialize components: {}", e),
                )
                .await;
        }
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Rez LSP Server shutting down...");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        info!("Document opened: {}", params.text_document.uri);
        self.on_change(params.text_document).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        info!("Document changed: {}", params.text_document.uri);

        if let Some(change) = params.content_changes.pop() {
            let mut document_map = self.document_map.write().await;
            document_map.insert(params.text_document.uri, change.text);
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        info!("Document saved: {}", params.text_document.uri);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        info!("Document closed: {}", params.text_document.uri);

        let mut document_map = self.document_map.write().await;
        document_map.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        super::completion::handle_completion(
            &params,
            &self.package_discovery,
        ).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        super::hover::handle_hover(&params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::LspService;

    #[tokio::test]
    async fn test_server_creation() {
        let (client, _) = LspService::new(RezLanguageServer::new);
        // Basic smoke test - server should be created without panicking
        drop(client);
    }
}
