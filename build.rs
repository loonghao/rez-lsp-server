//! Build script for Rez LSP Server.
//!
//! This script handles:
//! - Building the LSP server binary
//! - Building the VSCode extension
//! - Running tests
//! - Creating distribution packages

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=vscode-extension/package.json");
    println!("cargo:rerun-if-changed=vscode-extension/src/");
    println!("cargo:rerun-if-changed=vscode-extension/tsconfig.json");

    // Check if we should build the VSCode extension
    if env::var("CARGO_FEATURE_VSCODE_EXTENSION").is_ok()
        || env::var("BUILD_VSCODE_EXTENSION").is_ok()
    {
        build_vscode_extension();
    }
}

fn build_vscode_extension() {
    let vscode_dir = Path::new("vscode-extension");

    if !vscode_dir.exists() {
        println!("cargo:warning=VSCode extension directory not found, skipping extension build");
        return;
    }

    println!("cargo:warning=Building VSCode extension...");

    // Install dependencies
    let npm_install = Command::new("npm")
        .args(["install"])
        .current_dir(vscode_dir)
        .status();

    match npm_install {
        Ok(status) if status.success() => {
            println!("cargo:warning=VSCode extension dependencies installed");
        }
        Ok(_) => {
            println!("cargo:warning=Failed to install VSCode extension dependencies");
            return;
        }
        Err(e) => {
            println!("cargo:warning=Error running npm install: {}", e);
            return;
        }
    }

    // Compile TypeScript
    let npm_compile = Command::new("npm")
        .args(["run", "compile"])
        .current_dir(vscode_dir)
        .status();

    match npm_compile {
        Ok(status) if status.success() => {
            println!("cargo:warning=VSCode extension compiled successfully");
        }
        Ok(_) => {
            println!("cargo:warning=Failed to compile VSCode extension");
        }
        Err(e) => {
            println!("cargo:warning=Error running npm compile: {}", e);
        }
    }
}
