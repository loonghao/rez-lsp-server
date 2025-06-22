//! Diagnostic management for the LSP server.

use crate::core::Result;
use crate::validation::{Severity as ValidationSeverity, ValidationEngine, ValidationResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};

/// Manages diagnostics for the LSP server.
pub struct DiagnosticsManager {
    /// Validation engine for checking package.py files
    validation_engine: Arc<ValidationEngine>,
    /// Current diagnostics for each file
    diagnostics: Arc<RwLock<HashMap<Url, Vec<Diagnostic>>>>,
}

impl DiagnosticsManager {
    /// Create a new diagnostics manager.
    pub fn new() -> Result<Self> {
        let validation_engine = Arc::new(ValidationEngine::new()?);
        let diagnostics = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            validation_engine,
            diagnostics,
        })
    }

    /// Validate a file and update diagnostics.
    pub async fn validate_file(&self, uri: &Url, content: &str) -> Result<Vec<Diagnostic>> {
        let file_path = uri.path();

        // Run validation
        let validation_result = self.validation_engine.validate_file(content, file_path)?;

        // Convert validation issues to LSP diagnostics
        let diagnostics = self.convert_validation_result(&validation_result);

        // Store diagnostics
        {
            let mut diag_map = self.diagnostics.write().await;
            diag_map.insert(uri.clone(), diagnostics.clone());
        }

        Ok(diagnostics)
    }

    /// Get current diagnostics for a file.
    pub async fn get_diagnostics(&self, uri: &Url) -> Vec<Diagnostic> {
        let diag_map = self.diagnostics.read().await;
        diag_map.get(uri).cloned().unwrap_or_default()
    }

    /// Clear diagnostics for a file.
    pub async fn clear_diagnostics(&self, uri: &Url) {
        let mut diag_map = self.diagnostics.write().await;
        diag_map.remove(uri);
    }

    /// Get all files with diagnostics.
    pub async fn get_all_diagnostics(&self) -> HashMap<Url, Vec<Diagnostic>> {
        let diag_map = self.diagnostics.read().await;
        diag_map.clone()
    }

    /// Convert validation result to LSP diagnostics.
    fn convert_validation_result(&self, result: &ValidationResult) -> Vec<Diagnostic> {
        result
            .issues
            .iter()
            .map(|issue| {
                let severity = match issue.severity {
                    ValidationSeverity::Critical => DiagnosticSeverity::ERROR,
                    ValidationSeverity::Error => DiagnosticSeverity::ERROR,
                    ValidationSeverity::Warning => DiagnosticSeverity::WARNING,
                    ValidationSeverity::Info => DiagnosticSeverity::INFORMATION,
                };

                let start_pos = Position {
                    line: (issue.line - 1).max(0) as u32, // Convert to 0-based
                    character: (issue.column - 1).max(0) as u32, // Convert to 0-based
                };

                let end_pos = Position {
                    line: (issue.line - 1).max(0) as u32,
                    character: (issue.column - 1 + issue.length).max(0) as u32,
                };

                let range = Range {
                    start: start_pos,
                    end: end_pos,
                };

                let mut diagnostic = Diagnostic {
                    range,
                    severity: Some(severity),
                    code: Some(NumberOrString::String(issue.code.clone())),
                    code_description: None,
                    source: Some("rez-lsp".to_string()),
                    message: issue.message.clone(),
                    related_information: None,
                    tags: None,
                    data: None,
                };

                // Add suggestion as related information if available
                if let Some(suggestion) = &issue.suggestion {
                    diagnostic.message =
                        format!("{}\nSuggestion: {}", diagnostic.message, suggestion);
                }

                diagnostic
            })
            .collect()
    }

    /// Get validation statistics for all files.
    pub async fn get_validation_stats(&self) -> ValidationStats {
        let diag_map = self.diagnostics.read().await;

        let mut total_files = 0;
        let mut files_with_errors = 0;
        let mut files_with_warnings = 0;
        let mut total_diagnostics = 0;
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut total_info = 0;

        for diagnostics in diag_map.values() {
            total_files += 1;
            total_diagnostics += diagnostics.len();

            let mut has_errors = false;
            let mut has_warnings = false;

            for diagnostic in diagnostics {
                match diagnostic.severity {
                    Some(DiagnosticSeverity::ERROR) => {
                        total_errors += 1;
                        has_errors = true;
                    }
                    Some(DiagnosticSeverity::WARNING) => {
                        total_warnings += 1;
                        has_warnings = true;
                    }
                    Some(DiagnosticSeverity::INFORMATION) => {
                        total_info += 1;
                    }
                    _ => {}
                }
            }

            if has_errors {
                files_with_errors += 1;
            } else if has_warnings {
                files_with_warnings += 1;
            }
        }

        ValidationStats {
            total_files,
            files_with_errors,
            files_with_warnings,
            total_diagnostics,
            total_errors,
            total_warnings,
            total_info,
        }
    }
}

/// Statistics about validation across all files.
#[derive(Debug, Clone)]
pub struct ValidationStats {
    /// Total number of files with diagnostics
    pub total_files: usize,
    /// Number of files with errors
    pub files_with_errors: usize,
    /// Number of files with warnings (but no errors)
    pub files_with_warnings: usize,
    /// Total number of diagnostics
    pub total_diagnostics: usize,
    /// Total number of errors
    pub total_errors: usize,
    /// Total number of warnings
    pub total_warnings: usize,
    /// Total number of info messages
    pub total_info: usize,
}

impl ValidationStats {
    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.total_errors > 0
    }

    /// Check if there are any diagnostics.
    pub fn has_diagnostics(&self) -> bool {
        self.total_diagnostics > 0
    }

    /// Get the percentage of files with issues.
    pub fn error_rate(&self) -> f64 {
        if self.total_files > 0 {
            (self.files_with_errors as f64 / self.total_files as f64) * 100.0
        } else {
            0.0
        }
    }
}
