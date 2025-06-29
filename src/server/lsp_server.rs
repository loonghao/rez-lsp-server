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
use crate::server::{navigation::NavigationHandler, DiagnosticsManager};

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
    /// Diagnostics manager
    diagnostics_manager: Arc<DiagnosticsManager>,
    /// Navigation handler
    navigation_handler: Arc<NavigationHandler>,
}

impl RezLanguageServer {
    /// Create a new Rez Language Server instance.
    pub fn new(client: Client) -> Self {
        let diagnostics_manager =
            Arc::new(DiagnosticsManager::new().expect("Failed to create diagnostics manager"));

        let package_discovery = Arc::new(tokio::sync::RwLock::new(None));
        let navigation_handler = Arc::new(NavigationHandler::new(package_discovery.clone()));

        Self {
            client,
            document_map: tokio::sync::RwLock::new(HashMap::new()),
            config_provider: Arc::new(tokio::sync::RwLock::new(RezConfigProvider::new())),
            package_discovery,
            diagnostics_manager,
            navigation_handler,
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
                    format!(
                        "Discovered {} package families ({} total packages)",
                        families, total
                    ),
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
        document_map.insert(params.uri.clone(), params.text.clone());
        drop(document_map); // Release the lock early

        // Run diagnostics for package.py files
        if params.uri.path().ends_with("package.py") {
            if let Ok(diagnostics) = self
                .diagnostics_manager
                .validate_file(&params.uri, &params.text)
                .await
            {
                // Publish diagnostics to the client
                self.client
                    .publish_diagnostics(params.uri, diagnostics, None)
                    .await;
            }
        }
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
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
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
        let filename = params
            .text_document
            .uri
            .path()
            .split('/')
            .next_back()
            .unwrap_or("unknown");
        info!("Opened: {}", filename);
        self.on_change(params.text_document).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        // Use debug level for frequent document changes
        let filename = params
            .text_document
            .uri
            .path()
            .split('/')
            .next_back()
            .unwrap_or("unknown");
        tracing::debug!("Document changed: {}", filename);

        if let Some(change) = params.content_changes.pop() {
            let mut document_map = self.document_map.write().await;
            document_map.insert(params.text_document.uri.clone(), change.text.clone());
            drop(document_map); // Release the lock early

            // Run diagnostics for package.py files
            if params.text_document.uri.path().ends_with("package.py") {
                if let Ok(diagnostics) = self
                    .diagnostics_manager
                    .validate_file(&params.text_document.uri, &change.text)
                    .await
                {
                    // Publish diagnostics to the client
                    self.client
                        .publish_diagnostics(params.text_document.uri, diagnostics, None)
                        .await;
                }
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let filename = params
            .text_document
            .uri
            .path()
            .split('/')
            .next_back()
            .unwrap_or("unknown");
        info!("Saved: {}", filename);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let filename = params
            .text_document
            .uri
            .path()
            .split('/')
            .next_back()
            .unwrap_or("unknown");
        tracing::debug!("Closed: {}", filename);

        let mut document_map = self.document_map.write().await;
        document_map.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        super::completion::handle_completion(&params, &self.package_discovery).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        super::hover::handle_hover(&params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        match self
            .navigation_handler
            .handle_goto_definition(&params)
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Go to definition failed: {}", e),
                    )
                    .await;
                Ok(None)
            }
        }
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        match self
            .navigation_handler
            .handle_find_references(&params)
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => {
                self.client
                    .log_message(MessageType::ERROR, format!("Find references failed: {}", e))
                    .await;
                Ok(None)
            }
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        match self
            .navigation_handler
            .handle_document_symbols(&params)
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Document symbols failed: {}", e),
                    )
                    .await;
                Ok(None)
            }
        }
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        match self
            .navigation_handler
            .handle_workspace_symbols(&params)
            .await
        {
            Ok(response) => Ok(response),
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Workspace symbols failed: {}", e),
                    )
                    .await;
                Ok(None)
            }
        }
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
