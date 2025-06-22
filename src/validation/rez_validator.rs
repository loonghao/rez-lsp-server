//! Rez-specific validation for package.py files.

use super::{Severity, ValidationIssue, Validator};
use crate::core::{types::Version, Result};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Validates Rez-specific syntax and semantics in package.py files.
pub struct RezValidator {
    /// Required fields for a valid Rez package
    required_fields: HashSet<String>,
    /// Optional but recommended fields
    recommended_fields: HashSet<String>,
    /// Deprecated fields that should be avoided
    deprecated_fields: HashMap<String, String>,
    /// Regex patterns for validation
    patterns: RezPatterns,
}

struct RezPatterns {
    /// Pattern for version strings
    version_pattern: Regex,
    /// Pattern for package names
    name_pattern: Regex,
    /// Pattern for requirement strings
    requirement_pattern: Regex,
    /// Pattern for tool definitions
    #[allow(dead_code)]
    tool_pattern: Regex,
}

impl RezValidator {
    /// Create a new Rez validator.
    pub fn new() -> Result<Self> {
        let mut required_fields = HashSet::new();
        required_fields.insert("name".to_string());
        required_fields.insert("version".to_string());

        let mut recommended_fields = HashSet::new();
        recommended_fields.insert("description".to_string());
        recommended_fields.insert("authors".to_string());
        recommended_fields.insert("requires".to_string());

        let mut deprecated_fields = HashMap::new();
        deprecated_fields.insert(
            "uuid".to_string(),
            "UUIDs are no longer used in Rez packages".to_string(),
        );
        deprecated_fields.insert(
            "config".to_string(),
            "Use 'private_build_requires' instead".to_string(),
        );

        let patterns = RezPatterns {
            version_pattern: Regex::new(r"^[0-9]+(\.[0-9]+)*([a-zA-Z][a-zA-Z0-9]*)?$")?,
            name_pattern: Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$")?,
            requirement_pattern: Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*([<>=!]+[0-9]+(\.[0-9]+)*)?$")?,
            tool_pattern: Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$")?,
        };

        Ok(Self {
            required_fields,
            recommended_fields,
            deprecated_fields,
            patterns,
        })
    }

    /// Extract field assignments from Python code.
    fn extract_fields(&self, content: &str) -> HashMap<String, (u32, String)> {
        let mut fields = HashMap::new();
        let assignment_regex = Regex::new(r"^(\w+)\s*=\s*(.+)$").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num as u32 + 1;
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(captures) = assignment_regex.captures(trimmed) {
                let field_name = captures.get(1).unwrap().as_str().to_string();
                let field_value = captures.get(2).unwrap().as_str().to_string();
                fields.insert(field_name, (line_num, field_value));
            }
        }

        fields
    }

    /// Validate required fields.
    fn check_required_fields(
        &self,
        fields: &HashMap<String, (u32, String)>,
    ) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for required_field in &self.required_fields {
            if !fields.contains_key(required_field) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        1,
                        1,
                        1,
                        format!("Missing required field '{}'", required_field),
                        "R001",
                    )
                    .with_suggestion(format!(
                        "Add '{}' field to the package definition",
                        required_field
                    )),
                );
            }
        }

        issues
    }

    /// Validate recommended fields.
    fn check_recommended_fields(
        &self,
        fields: &HashMap<String, (u32, String)>,
    ) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for recommended_field in &self.recommended_fields {
            if !fields.contains_key(recommended_field) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        1,
                        1,
                        1,
                        format!("Missing recommended field '{}'", recommended_field),
                        "R101",
                    )
                    .with_suggestion(format!(
                        "Consider adding '{}' field for better package documentation",
                        recommended_field
                    )),
                );
            }
        }

        issues
    }

    /// Validate deprecated fields.
    fn check_deprecated_fields(
        &self,
        fields: &HashMap<String, (u32, String)>,
    ) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for (field_name, (line_num, _)) in fields {
            if let Some(reason) = self.deprecated_fields.get(field_name) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        *line_num,
                        1,
                        field_name.len() as u32,
                        format!("Deprecated field '{}': {}", field_name, reason),
                        "R201",
                    )
                    .with_suggestion("Remove this deprecated field"),
                );
            }
        }

        issues
    }

    /// Validate package name.
    fn validate_name(&self, fields: &HashMap<String, (u32, String)>) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some((line_num, value)) = fields.get("name") {
            let clean_value = self.clean_string_value(value);

            if !self.patterns.name_pattern.is_match(&clean_value) {
                issues.push(ValidationIssue::new(
                    Severity::Error,
                    *line_num,
                    1,
                    value.len() as u32,
                    "Invalid package name format",
                    "R002",
                ).with_suggestion("Package names must start with a letter and contain only letters, numbers, and underscores"));
            }

            // Check for reserved names
            let reserved_names = ["test", "build", "install", "package"];
            if reserved_names.contains(&clean_value.as_str()) {
                issues.push(
                    ValidationIssue::new(
                        Severity::Warning,
                        *line_num,
                        1,
                        value.len() as u32,
                        format!("Package name '{}' is a reserved word", clean_value),
                        "R102",
                    )
                    .with_suggestion("Consider using a different package name"),
                );
            }
        }

        issues
    }

    /// Validate version field.
    fn validate_version(&self, fields: &HashMap<String, (u32, String)>) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some((line_num, value)) = fields.get("version") {
            let clean_value = self.clean_string_value(value);

            // Try to parse as Rez version
            match Version::new(&clean_value) {
                version if version.tokens.is_empty() => {
                    issues.push(
                        ValidationIssue::new(
                            Severity::Error,
                            *line_num,
                            1,
                            value.len() as u32,
                            "Invalid version format",
                            "R003",
                        )
                        .with_suggestion("Use semantic versioning (e.g., '1.0.0')"),
                    );
                }
                _ => {
                    // Version is valid, check for best practices
                    if !self.patterns.version_pattern.is_match(&clean_value) {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                *line_num,
                                1,
                                value.len() as u32,
                                "Version format doesn't follow semantic versioning",
                                "R103",
                            )
                            .with_suggestion(
                                "Consider using semantic versioning (major.minor.patch)",
                            ),
                        );
                    }
                }
            }
        }

        issues
    }

    /// Validate requires field.
    fn validate_requires(&self, fields: &HashMap<String, (u32, String)>) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some((line_num, value)) = fields.get("requires") {
            // Parse the requires list
            if let Some(requirements) = self.parse_list_value(value) {
                for requirement in requirements.iter() {
                    let clean_req = self.clean_string_value(requirement);

                    // Validate requirement format
                    if !self.patterns.requirement_pattern.is_match(&clean_req) {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Error,
                                *line_num,
                                1,
                                requirement.len() as u32,
                                format!("Invalid requirement format: '{}'", clean_req),
                                "R004",
                            )
                            .with_suggestion(
                                "Requirements should be in format 'package' or 'package>=1.0.0'",
                            ),
                        );
                    }

                    // Check for common typos
                    let common_packages = ["python", "maya", "houdini", "nuke", "blender"];
                    if !common_packages
                        .iter()
                        .any(|&pkg| clean_req.starts_with(pkg))
                    {
                        // This is a custom package, check for naming conventions
                        if clean_req.contains('-') {
                            issues.push(
                                ValidationIssue::new(
                                    Severity::Warning,
                                    *line_num,
                                    1,
                                    requirement.len() as u32,
                                    "Package names with hyphens may cause issues",
                                    "R104",
                                )
                                .with_suggestion("Consider using underscores instead of hyphens"),
                            );
                        }
                    }
                }

                // Check for duplicate requirements
                let mut seen = HashSet::new();
                for requirement in &requirements {
                    let clean_req = self.clean_string_value(requirement);
                    let package_name = clean_req
                        .split(&['<', '>', '=', '!'][..])
                        .next()
                        .unwrap_or(&clean_req)
                        .to_string();

                    if !seen.insert(package_name.clone()) {
                        issues.push(
                            ValidationIssue::new(
                                Severity::Warning,
                                *line_num,
                                1,
                                value.len() as u32,
                                format!("Duplicate requirement: '{}'", package_name),
                                "R105",
                            )
                            .with_suggestion("Remove duplicate requirements"),
                        );
                    }
                }
            }
        }

        issues
    }

    /// Validate tools field.
    fn validate_tools(&self, fields: &HashMap<String, (u32, String)>) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if let Some((line_num, value)) = fields.get("tools") {
            // Basic validation for tools dictionary format
            if !value.trim().starts_with('{') || !value.trim().ends_with('}') {
                issues.push(
                    ValidationIssue::new(
                        Severity::Error,
                        *line_num,
                        1,
                        value.len() as u32,
                        "Tools field must be a dictionary",
                        "R005",
                    )
                    .with_suggestion("Use dictionary format: tools = {'tool_name': 'tool_path'}"),
                );
            }
        }

        issues
    }

    /// Clean string values by removing quotes.
    fn clean_string_value(&self, value: &str) -> String {
        value
            .trim()
            .trim_start_matches('"')
            .trim_end_matches('"')
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .to_string()
    }

    /// Parse a list value from Python syntax.
    fn parse_list_value(&self, value: &str) -> Option<Vec<String>> {
        let trimmed = value.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return None;
        }

        let content = &trimmed[1..trimmed.len() - 1];
        let items: Vec<String> = content
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Some(items)
    }
}

impl Default for RezValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create RezValidator")
    }
}

impl Validator for RezValidator {
    fn validate(&self, content: &str, _file_path: &str) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // Extract field assignments
        let fields = self.extract_fields(content);

        // Run all Rez-specific validations
        issues.extend(self.check_required_fields(&fields));
        issues.extend(self.check_recommended_fields(&fields));
        issues.extend(self.check_deprecated_fields(&fields));
        issues.extend(self.validate_name(&fields));
        issues.extend(self.validate_version(&fields));
        issues.extend(self.validate_requires(&fields));
        issues.extend(self.validate_tools(&fields));

        // Sort issues by line number
        issues.sort_by_key(|issue| issue.line);

        Ok(issues)
    }

    fn name(&self) -> &str {
        "RezValidator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rez_validator_creation() {
        let validator = RezValidator::new();
        assert!(validator.is_ok());
    }

    #[test]
    fn test_valid_rez_package() {
        let validator = RezValidator::new().unwrap();
        let content = r#"
name = "test_package"
version = "1.0.0"
description = "A test package"
authors = ["Test Author"]
requires = ["python>=3.7"]
"#;

        let issues = validator.validate(content, "package.py").unwrap();
        // Should have no errors, maybe some warnings
        assert!(issues.iter().all(|i| i.severity != Severity::Error));
    }

    #[test]
    fn test_missing_required_fields() {
        let validator = RezValidator::new().unwrap();
        let content = r#"
description = "A test package"
"#;

        let issues = validator.validate(content, "package.py").unwrap();
        assert!(issues
            .iter()
            .any(|i| i.code == "R001" && i.message.contains("name")));
        assert!(issues
            .iter()
            .any(|i| i.code == "R001" && i.message.contains("version")));
    }

    #[test]
    fn test_invalid_package_name() {
        let validator = RezValidator::new().unwrap();
        let content = r#"
name = "123invalid"
version = "1.0.0"
"#;

        let issues = validator.validate(content, "package.py").unwrap();
        assert!(issues.iter().any(|i| i.code == "R002"));
    }

    #[test]
    fn test_deprecated_fields() {
        let validator = RezValidator::new().unwrap();
        let content = r#"
name = "test"
version = "1.0.0"
uuid = "some-uuid"
"#;

        let issues = validator.validate(content, "package.py").unwrap();
        assert!(issues.iter().any(|i| i.code == "R201"));
    }
}
