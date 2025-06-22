//! Syntax validation for Rez package.py files.

pub mod python_validator;
pub mod rez_validator;
pub mod validation_engine;

pub use python_validator::PythonValidator;
pub use rez_validator::RezValidator;
pub use validation_engine::ValidationEngine;

use crate::core::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a validation issue found in a package.py file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// The severity of the issue
    pub severity: Severity,
    /// The line number where the issue occurs (1-based)
    pub line: u32,
    /// The column number where the issue occurs (1-based)
    pub column: u32,
    /// The length of the problematic text
    pub length: u32,
    /// A human-readable message describing the issue
    pub message: String,
    /// A unique code identifying the type of issue
    pub code: String,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
}

/// Severity levels for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational message
    Info,
    /// Warning that doesn't prevent functionality
    Warning,
    /// Error that may cause issues
    Error,
    /// Critical error that will definitely cause problems
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

impl ValidationIssue {
    /// Create a new validation issue.
    pub fn new(
        severity: Severity,
        line: u32,
        column: u32,
        length: u32,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            line,
            column,
            length,
            message: message.into(),
            code: code.into(),
            suggestion: None,
        }
    }

    /// Add a suggestion for fixing this issue.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Trait for validators that can check package.py files.
pub trait Validator {
    /// Validate the given content and return any issues found.
    fn validate(&self, content: &str, file_path: &str) -> Result<Vec<ValidationIssue>>;

    /// Get the name of this validator.
    fn name(&self) -> &str;

    /// Get the version of this validator.
    fn version(&self) -> &str {
        "1.0.0"
    }
}

/// Result of a validation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// The file that was validated
    pub file_path: String,
    /// All issues found during validation
    pub issues: Vec<ValidationIssue>,
    /// Statistics about the validation
    pub stats: ValidationStats,
}

/// Statistics about a validation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    /// Total number of issues found
    pub total_issues: usize,
    /// Number of critical issues
    pub critical_count: usize,
    /// Number of errors
    pub error_count: usize,
    /// Number of warnings
    pub warning_count: usize,
    /// Number of info messages
    pub info_count: usize,
    /// Time taken for validation in milliseconds
    pub validation_time_ms: u64,
}

impl ValidationResult {
    /// Create a new validation result.
    pub fn new(
        file_path: impl Into<String>,
        issues: Vec<ValidationIssue>,
        validation_time_ms: u64,
    ) -> Self {
        let stats = ValidationStats::from_issues(&issues, validation_time_ms);
        Self {
            file_path: file_path.into(),
            issues,
            stats,
        }
    }

    /// Check if the validation found any errors or critical issues.
    pub fn has_errors(&self) -> bool {
        self.stats.error_count > 0 || self.stats.critical_count > 0
    }

    /// Check if the validation found any issues at all.
    pub fn has_issues(&self) -> bool {
        self.stats.total_issues > 0
    }
}

impl ValidationStats {
    /// Create statistics from a list of issues.
    pub fn from_issues(issues: &[ValidationIssue], validation_time_ms: u64) -> Self {
        let mut critical_count = 0;
        let mut error_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;

        for issue in issues {
            match issue.severity {
                Severity::Critical => critical_count += 1,
                Severity::Error => error_count += 1,
                Severity::Warning => warning_count += 1,
                Severity::Info => info_count += 1,
            }
        }

        Self {
            total_issues: issues.len(),
            critical_count,
            error_count,
            warning_count,
            info_count,
            validation_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_issue_creation() {
        let issue = ValidationIssue::new(Severity::Error, 10, 5, 8, "Invalid syntax", "E001");

        assert_eq!(issue.severity, Severity::Error);
        assert_eq!(issue.line, 10);
        assert_eq!(issue.column, 5);
        assert_eq!(issue.length, 8);
        assert_eq!(issue.message, "Invalid syntax");
        assert_eq!(issue.code, "E001");
        assert!(issue.suggestion.is_none());
    }

    #[test]
    fn test_validation_issue_with_suggestion() {
        let issue = ValidationIssue::new(Severity::Warning, 5, 10, 3, "Deprecated field", "W001")
            .with_suggestion("Use 'new_field' instead");

        assert_eq!(
            issue.suggestion,
            Some("Use 'new_field' instead".to_string())
        );
    }

    #[test]
    fn test_validation_stats() {
        let issues = vec![
            ValidationIssue::new(Severity::Critical, 1, 1, 1, "Critical", "C001"),
            ValidationIssue::new(Severity::Error, 2, 1, 1, "Error", "E001"),
            ValidationIssue::new(Severity::Warning, 3, 1, 1, "Warning", "W001"),
            ValidationIssue::new(Severity::Info, 4, 1, 1, "Info", "I001"),
        ];

        let stats = ValidationStats::from_issues(&issues, 100);

        assert_eq!(stats.total_issues, 4);
        assert_eq!(stats.critical_count, 1);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.info_count, 1);
        assert_eq!(stats.validation_time_ms, 100);
    }

    #[test]
    fn test_validation_result() {
        let issues = vec![
            ValidationIssue::new(Severity::Error, 1, 1, 1, "Error", "E001"),
            ValidationIssue::new(Severity::Warning, 2, 1, 1, "Warning", "W001"),
        ];

        let result = ValidationResult::new("test.py", issues, 50);

        assert_eq!(result.file_path, "test.py");
        assert!(result.has_errors());
        assert!(result.has_issues());
        assert_eq!(result.stats.total_issues, 2);
        assert_eq!(result.stats.validation_time_ms, 50);
    }
}
