//! Completion handling for the LSP server.

use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::info;

use crate::core::PackageDiscovery;
use crate::discovery::PackageDiscoveryImpl;

/// Handle completion requests.
pub async fn handle_completion(
    params: &CompletionParams,
    package_discovery: &Arc<tokio::sync::RwLock<Option<PackageDiscoveryImpl>>>,
) -> Result<Option<CompletionResponse>> {
    info!(
        "Completion requested at {:?}",
        params.text_document_position
    );

    let package_discovery_guard = package_discovery.read().await;

    let completions = if let Some(ref discovery) = *package_discovery_guard {
        // Use real package discovery
        match discovery.get_all_package_names().await {
            Ok(package_names) => {
                let mut completions = Vec::new();

                for package_name in package_names {
                    match discovery.get_package_versions(&package_name).await {
                        Ok(versions) => {
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
                        Err(e) => {
                            tracing::warn!("Failed to get versions for package {}: {}", package_name, e);
                        }
                    }
                }

                completions
            }
            Err(e) => {
                tracing::warn!("Failed to get package names: {}", e);
                get_fallback_completions()
            }
        }
    } else {
        // Fallback to static completions if package discovery is not available
        get_fallback_completions()
    };

    Ok(Some(CompletionResponse::Array(completions)))
}

/// Get fallback completions when package discovery is not available.
fn get_fallback_completions() -> Vec<CompletionItem> {
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
                "Autodesk Maya 3D animation and modeling software".to_string(),
            )),
            insert_text: Some("maya".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "houdini".to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some("SideFX Houdini package".to_string()),
            documentation: Some(Documentation::String(
                "SideFX Houdini 3D animation and VFX software".to_string(),
            )),
            insert_text: Some("houdini".to_string()),
            ..Default::default()
        },
    ]
}
