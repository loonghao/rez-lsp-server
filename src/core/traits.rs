//! Core traits for the Rez LSP server.

use async_trait::async_trait;
use std::path::Path;

use super::{
    DependencyConflict, Package, Requirement, ResolvedContext, Result, Version, VersionConstraint,
};

/// Trait for Rez configuration management.
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Get package search paths.
    async fn get_package_paths(&self) -> Result<Vec<std::path::PathBuf>>;

    /// Get local packages path.
    async fn get_local_packages_path(&self) -> Result<Option<std::path::PathBuf>>;

    /// Get release packages path.
    async fn get_release_packages_path(&self) -> Result<Option<std::path::PathBuf>>;

    /// Validate configuration.
    async fn validate(&self) -> Result<()>;

    /// Reload configuration from environment.
    async fn reload(&mut self) -> Result<()>;
}

/// Trait for package discovery and caching.
#[async_trait]
pub trait PackageDiscovery: Send + Sync {
    /// Scan all configured package repositories.
    async fn scan_packages(&mut self) -> Result<()>;

    /// Find packages by name pattern.
    async fn find_packages(&self, pattern: &str) -> Result<Vec<Package>>;

    /// Get all versions of a specific package.
    async fn get_package_versions(&self, name: &str) -> Result<Vec<Package>>;

    /// Get all package names.
    async fn get_all_package_names(&self) -> Result<Vec<String>>;

    /// Get package by exact name and version.
    async fn get_package(&self, name: &str, version: &Version) -> Result<Option<Package>>;

    /// Get discovery statistics.
    async fn get_stats(&self) -> Result<(usize, usize)>; // (families, total_packages)

    /// Clear package cache.
    async fn clear_cache(&mut self) -> Result<()>;
}

/// Trait for parsing Rez package files.
#[async_trait]
pub trait PackageParser: Send + Sync {
    /// Parse a package.py file.
    async fn parse_package_file(&self, path: &Path) -> Result<Package>;

    /// Parse package content from string.
    async fn parse_package_content(&self, content: &str, base_path: &Path) -> Result<Package>;

    /// Validate package syntax.
    async fn validate_syntax(&self, content: &str) -> Result<Vec<SyntaxError>>;

    /// Extract requirements from package content.
    async fn extract_requirements(&self, content: &str) -> Result<Vec<Requirement>>;
}

/// Trait for dependency resolution.
#[async_trait]
pub trait DependencyResolver: Send + Sync {
    /// Resolve a list of package requirements.
    async fn resolve(&self, requirements: &[Requirement]) -> Result<ResolvedContext>;

    /// Check if requirements can be satisfied.
    async fn can_resolve(&self, requirements: &[Requirement]) -> Result<bool>;

    /// Find conflicts in requirements.
    async fn find_conflicts(&self, requirements: &[Requirement])
        -> Result<Vec<DependencyConflict>>;

    /// Get latest version of a package that satisfies constraints.
    async fn get_latest_version(
        &self,
        name: &str,
        constraint: &VersionConstraint,
    ) -> Result<Option<Version>>;
}

/// Trait for completion providers.
#[async_trait]
pub trait CompletionProvider: Send + Sync {
    /// Provide package name completions.
    async fn complete_package_names(&self, prefix: &str) -> Result<Vec<CompletionItem>>;

    /// Provide version completions for a package.
    async fn complete_versions(
        &self,
        package_name: &str,
        prefix: &str,
    ) -> Result<Vec<CompletionItem>>;

    /// Provide requirement completions.
    async fn complete_requirements(&self, prefix: &str) -> Result<Vec<CompletionItem>>;

    /// Provide tool completions.
    async fn complete_tools(&self, prefix: &str) -> Result<Vec<CompletionItem>>;
}

/// Trait for hover information providers.
#[async_trait]
pub trait HoverProvider: Send + Sync {
    /// Provide hover information for a package.
    async fn hover_package(
        &self,
        name: &str,
        version: Option<&Version>,
    ) -> Result<Option<HoverInfo>>;

    /// Provide hover information for a requirement.
    async fn hover_requirement(&self, requirement: &str) -> Result<Option<HoverInfo>>;

    /// Provide hover information for a tool.
    async fn hover_tool(&self, tool: &str) -> Result<Option<HoverInfo>>;
}

/// Trait for diagnostic providers.
#[async_trait]
pub trait DiagnosticProvider: Send + Sync {
    /// Analyze package file and return diagnostics.
    async fn analyze_package(&self, content: &str, path: &Path) -> Result<Vec<Diagnostic>>;

    /// Check for dependency conflicts.
    async fn check_conflicts(&self, requirements: &[Requirement]) -> Result<Vec<Diagnostic>>;

    /// Validate package syntax.
    async fn validate_syntax(&self, content: &str) -> Result<Vec<Diagnostic>>;
}

/// Represents a completion item.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// The label of the completion item.
    pub label: String,
    /// The kind of completion item.
    pub kind: CompletionItemKind,
    /// Additional detail information.
    pub detail: Option<String>,
    /// Documentation for the item.
    pub documentation: Option<String>,
    /// Text to insert when this completion is selected.
    pub insert_text: Option<String>,
    /// Sort text for ordering completions.
    pub sort_text: Option<String>,
}

/// The kind of completion item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionItemKind {
    /// A package
    Package,
    /// A version
    Version,
    /// A tool
    Tool,
    /// A requirement
    Requirement,
    /// A keyword
    Keyword,
    /// A variable
    Variable,
    /// A function
    Function,
}

/// Represents hover information.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// The main content of the hover.
    pub content: String,
    /// Optional range that the hover applies to.
    pub range: Option<Range>,
}

/// Represents a text range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// Start position.
    pub start: Position,
    /// End position.
    pub end: Position,
}

/// Represents a position in a text document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Line number (0-based).
    pub line: u32,
    /// Character offset (0-based).
    pub character: u32,
}

/// Represents a diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The range where the diagnostic applies.
    pub range: Range,
    /// The severity of the diagnostic.
    pub severity: DiagnosticSeverity,
    /// The diagnostic message.
    pub message: String,
    /// Optional source of the diagnostic.
    pub source: Option<String>,
    /// Optional diagnostic code.
    pub code: Option<String>,
}

/// The severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// An informational message.
    Information,
    /// A hint.
    Hint,
}

/// Represents a syntax error.
#[derive(Debug, Clone)]
pub struct SyntaxError {
    /// The range where the error occurs.
    pub range: Range,
    /// The error message.
    pub message: String,
    /// Optional suggestions for fixing the error.
    pub suggestions: Vec<String>,
}
