//! Hover handling for the LSP server.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tracing::info;

/// Handle hover requests.
pub async fn handle_hover(params: &HoverParams) -> Result<Option<Hover>> {
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
