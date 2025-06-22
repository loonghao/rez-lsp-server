//! Error types for the Rez LSP server.

use std::fmt;

/// Result type alias for the Rez LSP server.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the Rez LSP server.
///
/// This enum represents all possible errors that can occur in the LSP server.
/// Each variant contains specific error information and context to help with
/// debugging and error reporting.
///
/// # Error Categories
///
/// - **Config**: Configuration and environment setup errors
/// - **Discovery**: Package discovery and scanning errors
/// - **Parser**: Package file parsing errors
/// - **Resolver**: Dependency resolution errors
/// - **Lsp**: LSP protocol and communication errors
/// - **Io**: File system and I/O errors
/// - **Other**: Miscellaneous errors
#[derive(Debug)]
pub enum Error {
    /// Configuration related errors
    Config(ConfigError),
    /// Package discovery related errors
    Discovery(DiscoveryError),
    /// Package parsing related errors
    Parser(ParserError),
    /// Dependency resolution related errors
    Resolver(ResolverError),
    /// LSP protocol related errors
    Lsp(LspError),
    /// Invalid file path
    InvalidPath(String),
    /// I/O related errors
    Io(std::io::Error),
    /// Other errors
    Other(String),
}

/// Configuration error types
#[derive(Debug)]
pub enum ConfigError {
    /// Environment variable not found
    EnvVarNotFound(String),
    /// Invalid path
    InvalidPath(String),
    /// No valid package paths found
    NoValidPaths,
    /// Configuration validation failed
    ValidationFailed(String),
}

/// Package discovery error types
#[derive(Debug)]
pub enum DiscoveryError {
    /// Failed to scan directory
    ScanFailed(String),
    /// Package not found
    PackageNotFound(String),
    /// Invalid package structure
    InvalidStructure(String),
    /// Cache operation failed
    CacheFailed(String),
}

/// Parser error types
#[derive(Debug)]
pub enum ParserError {
    /// Failed to read file
    ReadFailed(String),
    /// Invalid syntax
    InvalidSyntax(String),
    /// Missing required field
    MissingField(String),
    /// Invalid field value
    InvalidValue(String),
}

/// Resolver error types
#[derive(Debug)]
pub enum ResolverError {
    /// Dependency conflict
    Conflict(String),
    /// Circular dependency
    CircularDependency(String),
    /// Version constraint not satisfiable
    UnsatisfiableConstraint(String),
    /// Package not found during resolution
    PackageNotFound(String),
}

/// LSP error types
#[derive(Debug)]
pub enum LspError {
    /// Invalid request
    InvalidRequest(String),
    /// Server not initialized
    NotInitialized,
    /// Internal server error
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(e) => write!(f, "Configuration error: {}", e),
            Error::Discovery(e) => write!(f, "Discovery error: {}", e),
            Error::Parser(e) => write!(f, "Parser error: {}", e),
            Error::Resolver(e) => write!(f, "Resolver error: {}", e),
            Error::Lsp(e) => write!(f, "LSP error: {}", e),
            Error::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::EnvVarNotFound(var) => {
                write!(f, "Environment variable not found: {}", var)
            }
            ConfigError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            ConfigError::NoValidPaths => write!(f, "No valid package paths found"),
            ConfigError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryError::ScanFailed(path) => write!(f, "Failed to scan directory: {}", path),
            DiscoveryError::PackageNotFound(name) => write!(f, "Package not found: {}", name),
            DiscoveryError::InvalidStructure(msg) => {
                write!(f, "Invalid package structure: {}", msg)
            }
            DiscoveryError::CacheFailed(msg) => write!(f, "Cache operation failed: {}", msg),
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::ReadFailed(path) => write!(f, "Failed to read file: {}", path),
            ParserError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            ParserError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ParserError::InvalidValue(msg) => write!(f, "Invalid field value: {}", msg),
        }
    }
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolverError::Conflict(msg) => write!(f, "Dependency conflict: {}", msg),
            ResolverError::CircularDependency(msg) => write!(f, "Circular dependency: {}", msg),
            ResolverError::UnsatisfiableConstraint(msg) => {
                write!(f, "Unsatisfiable constraint: {}", msg)
            }
            ResolverError::PackageNotFound(name) => {
                write!(f, "Package not found during resolution: {}", name)
            }
        }
    }
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LspError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            LspError::NotInitialized => write!(f, "Server not initialized"),
            LspError::Internal(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
impl std::error::Error for ConfigError {}
impl std::error::Error for DiscoveryError {}
impl std::error::Error for ParserError {}
impl std::error::Error for ResolverError {}
impl std::error::Error for LspError {}

// Conversion implementations
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Self {
        Error::Config(err)
    }
}

impl From<DiscoveryError> for Error {
    fn from(err: DiscoveryError) -> Self {
        Error::Discovery(err)
    }
}

impl From<ParserError> for Error {
    fn from(err: ParserError) -> Self {
        Error::Parser(err)
    }
}

impl From<ResolverError> for Error {
    fn from(err: ResolverError) -> Self {
        Error::Resolver(err)
    }
}

impl From<LspError> for Error {
    fn from(err: LspError) -> Self {
        Error::Lsp(err)
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::Parser(ParserError::InvalidSyntax(err.to_string()))
    }
}
