// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use rez_lsp_server::server::RezLanguageServer;
use std::env;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Initialize tracing - IMPORTANT: Use stderr for logs, not stdout
    // stdout is reserved for LSP protocol communication
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false) // Hide module paths
        .with_thread_ids(false) // Hide thread IDs
        .with_level(true) // Show log level
        .compact() // Use compact format
        .init();

    let args: Vec<String> = env::args().collect();

    // Handle command line arguments
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--version" | "-V" => {
                print_version();
                return;
            }
            "--stdio" => {
                // This is the default mode for LSP, just continue
                tracing::info!("Starting in stdio mode (LSP)");
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    // Start LSP server (default mode)
    tracing::info!(
        "Rez LSP Server {} starting in LSP mode...",
        env!("CARGO_PKG_VERSION")
    );

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(RezLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn print_help() {
    println!("Rez LSP Server {}", env!("CARGO_PKG_VERSION"));
    println!("Language Server Protocol implementation for Rez package management");
    println!();
    println!("USAGE:");
    println!("    rez-lsp-server [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print this help message and exit");
    println!("    -V, --version    Print version information and exit");
    println!("        --stdio      Start LSP server (default mode)");
    println!();
    println!("DESCRIPTION:");
    println!("    When run without arguments, starts the LSP server and communicates");
    println!("    via stdin/stdout using the Language Server Protocol.");
    println!();
    println!("    This server provides language support for Rez package files:");
    println!("    - Code completion for Rez functions and variables");
    println!("    - Hover information for Rez keywords");
    println!("    - Syntax validation and diagnostics");
    println!("    - Go to definition for package dependencies");
}

fn print_version() {
    println!("rez-lsp-server {}", env!("CARGO_PKG_VERSION"));
}
