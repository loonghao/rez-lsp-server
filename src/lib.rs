//! # Rez LSP Server
//!
//! A Language Server Protocol implementation for Rez package manager.
//!
//! This crate provides intelligent code completion, dependency resolution,
//! and syntax validation for Rez package.py files across all major IDEs.
//!
//! ## License
//!
//! Licensed under the Apache License, Version 2.0 (the "License");
//! you may not use this file except in compliance with the License.
//! You may obtain a copy of the License at
//!
//!     <http://www.apache.org/licenses/LICENSE-2.0>
//!
//! Unless required by applicable law or agreed to in writing, software
//! distributed under the License is distributed on an "AS IS" BASIS,
//! WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//! See the License for the specific language governing permissions and
//! limitations under the License.
//!
//! ## Architecture
//!
//! The server is built with a modular architecture:
//! - **Core**: Fundamental types and traits
//! - **Config**: Rez configuration management
//! - **Discovery**: Package discovery and caching
//! - **Parser**: Rez package.py file parsing
//! - **Resolver**: Dependency resolution engine
//! - **LSP**: Language Server Protocol implementation
//!
//! ## Example
//!
//! ```rust,no_run
//! use rez_lsp_server::server::RezLanguageServer;
//! use tower_lsp::{LspService, Server};
//!
//! #[tokio::main]
//! async fn main() {
//!     let stdin = tokio::io::stdin();
//!     let stdout = tokio::io::stdout();
//!
//!     let (service, socket) = LspService::new(RezLanguageServer::new);
//!     Server::new(stdin, stdout, socket).serve(service).await;
//! }
//! ```

pub mod config;
pub mod core;
pub mod discovery;
pub mod parser;
pub mod resolver;
pub mod server;

// Re-export commonly used types
pub use core::{Error, Result};
pub use server::RezLanguageServer;
