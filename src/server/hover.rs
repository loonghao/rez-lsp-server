//! Hover handling for the LSP server.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

/// Handle hover requests.
pub async fn handle_hover(params: &HoverParams) -> Result<Option<Hover>> {
    let uri = &params.text_document_position_params.text_document.uri;
    let position = &params.text_document_position_params.position;

    // Use debug level for frequent events like hover
    tracing::debug!(
        "Hover at {}:{}:{}",
        uri.path().split('/').next_back().unwrap_or("unknown"),
        position.line + 1,
        position.character + 1
    );

    // MVP: Basic hover information
    let hover_content = "Rez package definition file\n\nThis file defines a Rez package with its dependencies, version, and environment configuration.";

    Ok(Some(Hover {
        contents: HoverContents::Scalar(MarkedString::String(hover_content.to_string())),
        range: None,
    }))
}
