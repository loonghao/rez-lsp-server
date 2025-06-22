//! LSP server implementation for Rez.

mod lsp_server;
mod completion;
mod hover;
mod diagnostics;

pub use lsp_server::RezLanguageServer;
