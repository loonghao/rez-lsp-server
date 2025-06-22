//! Diagnostics handling for the LSP server.

use tower_lsp::lsp_types::*;

/// Handle diagnostic requests.
#[allow(dead_code)] // TODO: Implement diagnostics in future versions
pub async fn handle_diagnostics(_uri: &str, _content: &str) -> Vec<Diagnostic> {
    // TODO: Implement actual diagnostics
    Vec::new()
}
