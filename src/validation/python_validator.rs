//! Python syntax validation for package.py files.

use super::{Severity, ValidationIssue, Validator};
use crate::core::Result;
use regex::Regex;

/// Validates Python syntax in package.py files.
pub struct PythonValidator {
    /// Regex patterns for common Python syntax issues
    patterns: PythonPatterns,
}

struct PythonPatterns {
    /// Pattern for detecting invalid indentation
    invalid_indent: Regex,
    /// Pattern for detecting unclosed brackets
    unclosed_brackets: Regex,
    /// Pattern for detecting invalid string literals
    invalid_strings: Regex,
    /// Pattern for detecting invalid variable names
    invalid_names: Regex,
    /// Pattern for detecting missing colons
    missing_colons: Regex,
}

impl PythonValidator {
    /// Create a new Python validator.
    pub fn new() -> Result<Self> {
        let patterns = PythonPatterns {
            invalid_indent: Regex::new(r"^[ \t]+[^ \t#]")?,
            unclosed_brackets: Regex::new(r"[\[\(\{][^\]\)\}]*$")?,
            invalid_strings: Regex::new(r#"(["'])[^"']*$"#)?,
            invalid_names: Regex::new(r"\b\d+[a-zA-Z_]")?,
            missing_colons: Regex::new(
                r"^\s*(if|elif|else|for|while|def|class|try|except|finally|with)\b[^:]*$",
            )?,
        };

        Ok(Self { patterns })
    }

    /// Check for indentation issues.
    fn check_indentation(&self, content: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num as u32 + 1;

            // Skip empty lines and comments
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            let indent = line.len() - line.trim_start().len();

            // Check for mixed tabs and spaces
            if line.starts_with('\t') && line.contains(' ') {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        line_num,
                        1,
                        indent as u32,
                        "Mixed tabs and spaces in indentation",
                        "E101",
                    )
                    .with_suggestion("Use either tabs or spaces consistently"),
                );
            }

            // Simple indentation check: if previous line ended with ':', this line should be indented
            if line_num > 1 {
                let lines: Vec<&str> = content.lines().collect();
                if let Some(prev_line) = lines.get((line_num - 2) as usize) {
                    if prev_line.trim().ends_with(':') && indent == 0 && !line.trim().is_empty() {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                line_num,
                                1,
                                1,
                                "Expected an indented block",
                                "E111",
                            )
                            .with_suggestion("Indent this line"),
                        );
                    }
                }
            }
        }

        issues
    }

    /// Check for syntax errors.
    fn check_syntax_errors(&self, content: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num as u32 + 1;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for missing colons
            if self.patterns.missing_colons.is_match(trimmed) {
                let col = line.len() as u32;
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        line_num,
                        col,
                        1,
                        "Missing colon at end of statement",
                        "E999",
                    )
                    .with_suggestion("Add ':' at the end of the line"),
                );
            }

            // Check for unclosed strings
            if self.check_unclosed_strings(line) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        line_num,
                        1,
                        line.len() as u32,
                        "Unclosed string literal",
                        "E902",
                    )
                    .with_suggestion("Close the string literal"),
                );
            }

            // Check for invalid variable names
            if let Some(mat) = self.patterns.invalid_names.find(trimmed) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        line_num,
                        mat.start() as u32 + 1,
                        mat.len() as u32,
                        "Invalid variable name (cannot start with digit)",
                        "E999",
                    )
                    .with_suggestion("Variable names must start with a letter or underscore"),
                );
            }
        }

        issues
    }

    /// Check for unclosed string literals.
    fn check_unclosed_strings(&self, line: &str) -> bool {
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;

        for ch in line.chars() {
            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                _ => {}
            }
        }

        in_single_quote || in_double_quote
    }

    /// Check for bracket matching.
    fn check_bracket_matching(&self, content: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut bracket_stack = Vec::new();
        let mut _line_positions: Vec<u32> = Vec::new();

        // Track line positions for error reporting
        let mut current_line = 1u32;
        let mut current_col = 1u32;

        for ch in content.chars() {
            match ch {
                '(' | '[' | '{' => {
                    bracket_stack.push((ch, current_line, current_col));
                }
                ')' | ']' | '}' => {
                    if let Some((open_bracket, _open_line, _open_col)) = bracket_stack.pop() {
                        let expected_close = match open_bracket {
                            '(' => ')',
                            '[' => ']',
                            '{' => '}',
                            _ => unreachable!(),
                        };

                        if ch != expected_close {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Error,
                                    current_line,
                                    current_col,
                                    1,
                                    format!(
                                        "Mismatched bracket: expected '{}', found '{}'",
                                        expected_close, ch
                                    ),
                                    "E999",
                                )
                                .with_suggestion(format!(
                                    "Change '{}' to '{}'",
                                    ch, expected_close
                                )),
                            );
                        }
                    } else {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                current_line,
                                current_col,
                                1,
                                format!("Unmatched closing bracket '{}'", ch),
                                "E999",
                            )
                            .with_suggestion(
                                "Remove the extra closing bracket or add matching opening bracket",
                            ),
                        );
                    }
                }
                '\n' => {
                    current_line += 1;
                    current_col = 1;
                    continue;
                }
                _ => {}
            }

            current_col += 1;
        }

        // Check for unclosed brackets
        for (bracket, line, col) in bracket_stack {
            let expected_close = match bracket {
                '(' => ')',
                '[' => ']',
                '{' => '}',
                _ => unreachable!(),
            };

            issues.push(
                ValidationIssue::new(
                    Severity::Error,
                    line,
                    col,
                    1,
                    format!("Unclosed bracket '{}'", bracket),
                    "E999",
                )
                .with_suggestion(format!("Add closing bracket '{}'", expected_close)),
            );
        }

        issues
    }

    /// Check for common Python style issues.
    fn check_style_issues(&self, content: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num as u32 + 1;

            // Check line length (PEP 8 recommends 79 characters)
            if line.len() > 79 {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        line_num,
                        80,
                        (line.len() - 79) as u32,
                        "Line too long (>79 characters)",
                        "W501",
                    )
                    .with_suggestion("Break long lines or use line continuation"),
                );
            }

            // Check for trailing whitespace
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed_len = line.trim_end().len();
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        line_num,
                        trimmed_len as u32 + 1,
                        (line.len() - trimmed_len) as u32,
                        "Trailing whitespace",
                        "W291",
                    )
                    .with_suggestion("Remove trailing whitespace"),
                );
            }
        }

        issues
    }
}

impl Default for PythonValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create PythonValidator")
    }
}

impl Validator for PythonValidator {
    fn validate(&self, content: &str, _file_path: &str) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Run all validation checks
        issues.extend(self.check_indentation(content));
        issues.extend(self.check_syntax_errors(content));
        issues.extend(self.check_bracket_matching(content));
        issues.extend(self.check_style_issues(content));

        // Sort issues by line number, then by column
        issues.sort_by(|a, b| a.line.cmp(&b.line).then_with(|| a.column.cmp(&b.column)));

        Ok(issues)
    }

    fn name(&self) -> &str {
        "PythonValidator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_validator_creation() {
        let validator = PythonValidator::new();
        assert!(validator.is_ok());
    }

    #[test]
    fn test_valid_python_code() {
        let validator = PythonValidator::new().unwrap();
        let content = r#"
name = "test"
version = "1.0.0"
description = "A test package"

def build():
    pass
"#;

        let issues = validator.validate(content, "test.py").unwrap();
        // Should have minimal issues (maybe style warnings)
        assert!(issues.iter().all(|i| i.severity != Severity::Error));
    }

    #[test]
    fn test_syntax_errors() {
        let validator = PythonValidator::new().unwrap();
        let content = r#"
name = "test
version = 1.0.0"
def build(
    pass
"#;

        let issues = validator.validate(content, "test.py").unwrap();
        assert!(issues.iter().any(|i| i.severity == Severity::Error));
    }

    #[test]
    fn test_indentation_errors() {
        let validator = PythonValidator::new().unwrap();
        let content = r#"def build():
pass"#;

        let issues = validator.validate(content, "test.py").unwrap();
        // Debug print to see what issues we get
        for issue in &issues {
            println!("Issue: {} - {}", issue.code, issue.message);
        }
        assert!(issues.iter().any(|i| i.code.starts_with("E1")));
    }

    #[test]
    fn test_bracket_matching() {
        let validator = PythonValidator::new().unwrap();
        let content = r#"requires = ["python", "maya"
tools = {"tool1": "path1"
"#;

        let issues = validator.validate(content, "test.py").unwrap();
        // Debug print to see what issues we get
        for issue in &issues {
            println!("Issue: {} - {}", issue.code, issue.message);
        }
        // This should find unclosed brackets
        assert!(issues
            .iter()
            .any(|i| i.message.contains("bracket") || i.message.contains("Unclosed")));
    }
}
