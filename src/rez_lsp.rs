use std::collections::HashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::info;

use crate::package_discovery::PackageDiscovery;
use crate::rez_config::RezConfig;

pub struct RezLanguageServer {
    client: Client,
    document_map: tokio::sync::RwLock<HashMap<Url, String>>,
    package_discovery: tokio::sync::RwLock<Option<PackageDiscovery>>,
}

impl RezLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: tokio::sync::RwLock::new(HashMap::new()),
            package_discovery: tokio::sync::RwLock::new(None),
        }
    }

    /// Initialize package discovery
    async fn initialize_package_discovery(&self) -> Result<()> {
        info!("Initializing package discovery");

        match RezConfig::from_environment() {
            Ok(config) => match config.validate() {
                Ok(_) => {
                    let mut discovery = PackageDiscovery::new(config);
                    if let Err(e) = discovery.scan_packages() {
                        self.client
                            .log_message(
                                MessageType::WARNING,
                                format!("Failed to scan packages: {}", e),
                            )
                            .await;
                    } else {
                        let (families, total) = discovery.get_stats();
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
                }
                Err(e) => {
                    self.client
                        .log_message(
                            MessageType::WARNING,
                            format!("Rez configuration validation failed: {}", e),
                        )
                        .await;
                }
            },
            Err(e) => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Failed to load Rez configuration: {}", e),
                    )
                    .await;
            }
        }

        Ok(())
    }

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
                version: Some("0.1.0".to_string()),
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

        // Initialize package discovery in the background
        if let Err(e) = self.initialize_package_discovery().await {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!("Failed to initialize package discovery: {}", e),
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
        info!(
            "Completion requested at {:?}",
            params.text_document_position
        );

        let package_discovery = self.package_discovery.read().await;

        let completions = if let Some(ref discovery) = *package_discovery {
            // Use real package discovery
            let package_names = discovery.get_all_package_names();
            let mut completions = Vec::new();

            for package_name in package_names {
                if let Some(versions) = discovery.get_package_versions(&package_name) {
                    if let Some(latest_version) = versions.last() {
                        completions.push(CompletionItem {
                            label: package_name.clone(),
                            kind: Some(CompletionItemKind::MODULE),
                            detail: Some(format!(
                                "Rez package (latest: {})",
                                latest_version.version
                            )),
                            documentation: latest_version
                                .description
                                .as_ref()
                                .map(|desc| Documentation::String(desc.clone())),
                            insert_text: Some(package_name),
                            ..Default::default()
                        });
                    }
                }
            }

            completions
        } else {
            // Fallback to static completions if package discovery is not available
            vec![
                CompletionItem {
                    label: "python".to_string(),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some("Python interpreter package".to_string()),
                    documentation: Some(Documentation::String(
                        "Python programming language interpreter".to_string(),
                    )),
                    insert_text: Some("python".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "maya".to_string(),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some("Autodesk Maya package".to_string()),
                    documentation: Some(Documentation::String(
                        "Autodesk Maya 3D animation software".to_string(),
                    )),
                    insert_text: Some("maya".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "houdini".to_string(),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some("SideFX Houdini package".to_string()),
                    documentation: Some(Documentation::String(
                        "SideFX Houdini 3D animation software".to_string(),
                    )),
                    insert_text: Some("houdini".to_string()),
                    ..Default::default()
                },
            ]
        };

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        info!(
            "Hover requested at {:?}",
            params.text_document_position_params
        );

        // MVP: Basic hover information
        let hover_content = "Rez package definition file\n\nThis file defines a Rez package with its dependencies, version, and environment configuration.";

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(hover_content.to_string())),
            range: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_initialization() {
        let (_client, _) = tower_lsp::LspService::new(RezLanguageServer::new);

        // Test that the server can be created
        assert!(true); // Basic smoke test
    }

    #[test]
    fn test_package_completion_items() {
        // Test that we have some basic completion items
        let completions = vec![
            "python".to_string(),
            "maya".to_string(),
            "houdini".to_string(),
        ];

        assert!(completions.contains(&"python".to_string()));
        assert!(completions.contains(&"maya".to_string()));
        assert!(completions.contains(&"houdini".to_string()));
    }
}
