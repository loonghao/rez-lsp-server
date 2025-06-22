//! Validation engine that coordinates multiple validators.

use super::{PythonValidator, RezValidator, ValidationIssue, ValidationResult, Validator};
use crate::core::Result;
use std::sync::Arc;
use std::time::Instant;

/// Configuration for the validation engine.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to enable Python syntax validation
    pub enable_python_validation: bool,
    /// Whether to enable Rez-specific validation
    pub enable_rez_validation: bool,
    /// Maximum number of issues to report per file
    pub max_issues_per_file: usize,
    /// Whether to include style warnings
    pub include_style_warnings: bool,
    /// Whether to include informational messages
    pub include_info_messages: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enable_python_validation: true,
            enable_rez_validation: true,
            max_issues_per_file: 100,
            include_style_warnings: true,
            include_info_messages: false,
        }
    }
}

/// Main validation engine that coordinates multiple validators.
pub struct ValidationEngine {
    /// Configuration for validation
    config: ValidationConfig,
    /// Python syntax validator
    python_validator: Option<Arc<PythonValidator>>,
    /// Rez-specific validator
    rez_validator: Option<Arc<RezValidator>>,
}

impl ValidationEngine {
    /// Create a new validation engine with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(ValidationConfig::default())
    }

    /// Create a new validation engine with custom configuration.
    pub fn with_config(config: ValidationConfig) -> Result<Self> {
        let python_validator = if config.enable_python_validation {
            Some(Arc::new(PythonValidator::new()?))
        } else {
            None
        };

        let rez_validator = if config.enable_rez_validation {
            Some(Arc::new(RezValidator::new()?))
        } else {
            None
        };

        Ok(Self {
            config,
            python_validator,
            rez_validator,
        })
    }

    /// Validate a package.py file and return all issues found.
    pub fn validate_file(&self, content: &str, file_path: &str) -> Result<ValidationResult> {
        let start_time = Instant::now();
        let mut all_issues = Vec::new();

        // Run Python validation if enabled
        if let Some(validator) = &self.python_validator {
            match validator.validate(content, file_path) {
                Ok(mut issues) => {
                    all_issues.append(&mut issues);
                }
                Err(e) => {
                    // Log error but continue with other validators
                    eprintln!("Python validation failed: {}", e);
                }
            }
        }

        // Run Rez validation if enabled
        if let Some(validator) = &self.rez_validator {
            match validator.validate(content, file_path) {
                Ok(mut issues) => {
                    all_issues.append(&mut issues);
                }
                Err(e) => {
                    // Log error but continue
                    eprintln!("Rez validation failed: {}", e);
                }
            }
        }

        // Filter issues based on configuration
        all_issues = self.filter_issues(all_issues);

        // Sort issues by severity (critical first), then by line number
        all_issues.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.line.cmp(&b.line))
                .then_with(|| a.column.cmp(&b.column))
        });

        // Limit number of issues if configured
        if all_issues.len() > self.config.max_issues_per_file {
            all_issues.truncate(self.config.max_issues_per_file);

            // Add a warning about truncation
            all_issues.push(
                ValidationIssue::new(
                    super::Severity::Warning,
                    1,
                    1,
                    1,
                    format!(
                        "Too many issues found. Showing first {} issues.",
                        self.config.max_issues_per_file
                    ),
                    "V001",
                )
                .with_suggestion("Fix the most critical issues first"),
            );
        }

        let validation_time = start_time.elapsed().as_millis() as u64;
        Ok(ValidationResult::new(
            file_path,
            all_issues,
            validation_time,
        ))
    }

    /// Validate multiple files concurrently.
    pub fn validate_files(&self, files: &[(String, String)]) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for (file_path, content) in files {
            match self.validate_file(content, file_path) {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Failed to validate {}: {}", file_path, e);
                    // Create an error result
                    let error_issue = ValidationIssue::new(
                        super::Severity::Critical,
                        1,
                        1,
                        1,
                        format!("Validation failed: {}", e),
                        "V999",
                    );
                    results.push(ValidationResult::new(file_path, vec![error_issue], 0));
                }
            }
        }

        Ok(results)
    }

    /// Filter issues based on configuration.
    fn filter_issues(&self, issues: Vec<ValidationIssue>) -> Vec<ValidationIssue> {
        issues
            .into_iter()
            .filter(|issue| match issue.severity {
                super::Severity::Info => self.config.include_info_messages,
                super::Severity::Warning => self.config.include_style_warnings,
                super::Severity::Error | super::Severity::Critical => true,
            })
            .collect()
    }

    /// Get validation statistics for a set of results.
    pub fn get_summary_stats(&self, results: &[ValidationResult]) -> ValidationSummary {
        let total_files = results.len();
        let mut files_with_errors = 0;
        let mut files_with_warnings = 0;
        let mut total_issues = 0;
        let mut total_critical = 0;
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut total_info = 0;
        let mut total_validation_time = 0;

        for result in results {
            total_issues += result.stats.total_issues;
            total_critical += result.stats.critical_count;
            total_errors += result.stats.error_count;
            total_warnings += result.stats.warning_count;
            total_info += result.stats.info_count;
            total_validation_time += result.stats.validation_time_ms;

            if result.has_errors() {
                files_with_errors += 1;
            } else if result.stats.warning_count > 0 {
                files_with_warnings += 1;
            }
        }

        ValidationSummary {
            total_files,
            files_with_errors,
            files_with_warnings,
            total_issues,
            total_critical,
            total_errors,
            total_warnings,
            total_info,
            total_validation_time_ms: total_validation_time,
        }
    }

    /// Update the validation configuration.
    pub fn update_config(&mut self, config: ValidationConfig) -> Result<()> {
        // Recreate validators if needed
        if config.enable_python_validation && self.python_validator.is_none() {
            self.python_validator = Some(Arc::new(PythonValidator::new()?));
        } else if !config.enable_python_validation {
            self.python_validator = None;
        }

        if config.enable_rez_validation && self.rez_validator.is_none() {
            self.rez_validator = Some(Arc::new(RezValidator::new()?));
        } else if !config.enable_rez_validation {
            self.rez_validator = None;
        }

        self.config = config;
        Ok(())
    }

    /// Get the current configuration.
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }
}

/// Summary statistics for validation results.
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    /// Total number of files validated
    pub total_files: usize,
    /// Number of files with errors
    pub files_with_errors: usize,
    /// Number of files with warnings (but no errors)
    pub files_with_warnings: usize,
    /// Total number of issues found
    pub total_issues: usize,
    /// Total critical issues
    pub total_critical: usize,
    /// Total errors
    pub total_errors: usize,
    /// Total warnings
    pub total_warnings: usize,
    /// Total info messages
    pub total_info: usize,
    /// Total validation time in milliseconds
    pub total_validation_time_ms: u64,
}

impl ValidationSummary {
    /// Check if any critical issues or errors were found.
    pub fn has_errors(&self) -> bool {
        self.total_critical > 0 || self.total_errors > 0
    }

    /// Check if any issues were found.
    pub fn has_issues(&self) -> bool {
        self.total_issues > 0
    }

    /// Get the average validation time per file.
    pub fn average_validation_time_ms(&self) -> f64 {
        if self.total_files > 0 {
            self.total_validation_time_ms as f64 / self.total_files as f64
        } else {
            0.0
        }
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create ValidationEngine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_engine_creation() {
        let engine = ValidationEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_validation_engine_with_config() {
        let config = ValidationConfig {
            enable_python_validation: false,
            enable_rez_validation: true,
            ..Default::default()
        };

        let engine = ValidationEngine::with_config(config);
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        assert!(engine.python_validator.is_none());
        assert!(engine.rez_validator.is_some());
    }

    #[test]
    fn test_file_validation() {
        let engine = ValidationEngine::new().unwrap();
        let content = r#"
name = "test"
version = "1.0.0"
description = "Test package"
"#;

        let result = engine.validate_file(content, "test.py");
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.file_path, "test.py");
        assert!(result.stats.validation_time_ms > 0);
    }

    #[test]
    fn test_multiple_file_validation() {
        let engine = ValidationEngine::new().unwrap();
        let files = vec![
            (
                "test1.py".to_string(),
                "name = \"test1\"\nversion = \"1.0.0\"".to_string(),
            ),
            (
                "test2.py".to_string(),
                "name = \"test2\"\nversion = \"2.0.0\"".to_string(),
            ),
        ];

        let results = engine.validate_files(&files);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_validation_summary() {
        let engine = ValidationEngine::new().unwrap();
        let files = vec![
            (
                "test1.py".to_string(),
                "name = \"test1\"\nversion = \"1.0.0\"".to_string(),
            ),
            ("test2.py".to_string(), "invalid syntax here".to_string()),
        ];

        let results = engine.validate_files(&files).unwrap();
        let summary = engine.get_summary_stats(&results);

        assert_eq!(summary.total_files, 2);
        assert!(summary.total_validation_time_ms > 0);
    }
}
