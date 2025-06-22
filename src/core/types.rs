//! Core data types for the Rez LSP server.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a Rez package with all its metadata.
///
/// A package is the fundamental unit in the Rez ecosystem. Each package
/// has a name, version, and various metadata that describes its dependencies,
/// tools, and environment configuration.
///
/// # Examples
///
/// ```rust
/// use rez_lsp_server::core::{Package, Version};
/// use std::path::PathBuf;
/// use std::collections::HashMap;
///
/// let package = Package {
///     name: "python".to_string(),
///     version: Version::new("3.9.0"),
///     description: Some("Python interpreter".to_string()),
///     authors: vec!["Python Software Foundation".to_string()],
///     requires: vec![],
///     tools: vec!["python".to_string(), "pip".to_string()],
///     variants: vec![],
///     path: PathBuf::from("/packages/python/3.9.0"),
///     metadata: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    /// Package name
    pub name: String,
    /// Package version
    pub version: Version,
    /// Package description
    pub description: Option<String>,
    /// Package authors
    pub authors: Vec<String>,
    /// Package requirements
    pub requires: Vec<Requirement>,
    /// Package tools
    pub tools: Vec<String>,
    /// Package variants
    pub variants: Vec<Variant>,
    /// Package installation path
    pub path: PathBuf,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Represents a package version with semantic comparison support.
///
/// Rez versions follow a flexible versioning scheme that supports
/// numeric and string tokens. Versions are compared lexicographically
/// with special handling for numeric tokens.
///
/// # Version Format
///
/// - Numeric tokens: `1`, `2`, `10` (compared numerically)
/// - String tokens: `alpha`, `beta`, `rc` (compared lexicographically)
/// - Underscore tokens: `_` (lowest priority)
///
/// # Examples
///
/// ```rust
/// use rez_lsp_server::core::Version;
///
/// let v1 = Version::new("1.0.0");
/// let v2 = Version::new("1.0.1");
/// let v3 = Version::new("1.1.0");
///
/// assert!(v1 < v2);
/// assert!(v2 < v3);
/// assert!(v1 < v3);
///
/// // String versions
/// let alpha = Version::new("1.0.0-alpha");
/// let beta = Version::new("1.0.0-beta");
/// assert!(alpha < beta);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    /// Version string (e.g., "1.2.3", "2.0.0-beta.1")
    pub value: String,
    /// Parsed version tokens for comparison
    pub tokens: Vec<VersionToken>,
}

/// A single token in a version string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VersionToken {
    /// Numeric token
    Number(u64),
    /// String token
    String(String),
    /// Underscore token (lowest priority)
    Underscore,
}

/// Represents a package requirement/dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    /// Package name
    pub name: String,
    /// Version constraint
    pub constraint: VersionConstraint,
    /// Whether this is a weak requirement
    pub weak: bool,
    /// Whether this is a conflict requirement
    pub conflict: bool,
}

/// Version constraint for package requirements.
///
/// Supports Rez's version constraint syntax:
/// - `python` - Any version
/// - `python==3.9` - Exact version
/// - `python-3.7+` - Minimum version (inclusive)
/// - `python<4` - Maximum version (exclusive)
/// - `python-3.7+<4` - Range constraint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionConstraint {
    /// Any version
    Any,
    /// Exact version match
    Exact(Version),
    /// Minimum version (inclusive)
    GreaterEqual(Version),
    /// Maximum version (exclusive)
    Less(Version),
    /// Range constraint
    Range {
        /// Minimum version (inclusive)
        min: Version,
        /// Maximum version (exclusive)
        max: Version,
    },
    /// OR constraint (multiple alternatives)
    Or(Vec<VersionConstraint>),
}

impl VersionConstraint {
    /// Check if a version satisfies this constraint.
    pub fn satisfies(&self, version: &Version) -> bool {
        match self {
            VersionConstraint::Any => true,
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::GreaterEqual(v) => version >= v,
            VersionConstraint::Less(v) => version < v,
            VersionConstraint::Range { min, max } => version >= min && version < max,
            VersionConstraint::Or(constraints) => constraints.iter().any(|c| c.satisfies(version)),
        }
    }

    /// Parse a Rez requirement string into a constraint.
    pub fn parse(requirement: &str) -> Result<(String, Self), String> {
        let requirement = requirement.trim();

        // Parse version constraints
        if let Some(eq_pos) = requirement.find("==") {
            let name = requirement[..eq_pos].to_string();
            let version_str = &requirement[eq_pos + 2..];
            let version = Version::new(version_str);
            return Ok((name, VersionConstraint::Exact(version)));
        }

        if let Some(range_start) = requirement.find('-') {
            if let Some(plus_pos) = requirement[range_start..].find('+') {
                let name = requirement[..range_start].to_string();
                let version_str = &requirement[range_start + 1..range_start + plus_pos];
                let version = Version::new(version_str);

                // Check for upper bound
                let remaining = &requirement[range_start + plus_pos + 1..];
                if let Some(stripped) = remaining.strip_prefix('<') {
                    let max_version = Version::new(stripped);
                    return Ok((
                        name,
                        VersionConstraint::Range {
                            min: version,
                            max: max_version,
                        },
                    ));
                } else {
                    return Ok((name, VersionConstraint::GreaterEqual(version)));
                }
            }
        }

        if let Some(lt_pos) = requirement.find('<') {
            let name = requirement[..lt_pos].to_string();
            let version_str = &requirement[lt_pos + 1..];
            let version = Version::new(version_str);
            return Ok((name, VersionConstraint::Less(version)));
        }

        // No version constraint - any version
        Ok((requirement.to_string(), VersionConstraint::Any))
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionConstraint::Any => write!(f, "*"),
            VersionConstraint::Exact(v) => write!(f, "=={}", v),
            VersionConstraint::GreaterEqual(v) => write!(f, "{}+", v),
            VersionConstraint::Less(v) => write!(f, "<{}", v),
            VersionConstraint::Range { min, max } => write!(f, "{}+<{}", min, max),
            VersionConstraint::Or(constraints) => {
                let constraint_strs: Vec<String> =
                    constraints.iter().map(|c| c.to_string()).collect();
                write!(f, "{}", constraint_strs.join("|"))
            }
        }
    }
}

/// Represents a package variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variant {
    /// Variant index
    pub index: usize,
    /// Variant requirements
    pub requires: Vec<Requirement>,
    /// Variant metadata
    pub metadata: HashMap<String, String>,
}

/// Represents a resolved package context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedContext {
    /// List of resolved packages
    pub packages: Vec<Package>,
    /// Resolution metadata
    pub metadata: ContextMetadata,
}

/// Metadata for a resolved context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Timestamp when context was resolved
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Resolver version used
    pub resolver_version: String,
    /// Platform information
    pub platform: PlatformInfo,
    /// Resolution statistics
    pub stats: ResolutionStats,
}

/// Platform information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// Platform string
    pub platform: String,
}

/// Resolution statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionStats {
    /// Number of packages considered
    pub packages_considered: usize,
    /// Number of packages resolved
    pub packages_resolved: usize,
    /// Resolution time in milliseconds
    pub resolution_time_ms: u64,
    /// Number of conflicts encountered
    pub conflicts: usize,
}

/// Represents a dependency conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConflict {
    /// Package name involved in the conflict
    pub package: String,
    /// Requirements that caused the conflict
    pub requirements: Vec<Requirement>,
    /// Human-readable description of the conflict
    pub description: String,
}

impl Version {
    /// Create a new version from a string.
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let tokens = Self::parse_tokens(&value);
        Self { value, tokens }
    }

    /// Parse version tokens from a version string.
    fn parse_tokens(version: &str) -> Vec<VersionToken> {
        let mut tokens = Vec::new();

        for part in version.split(&['.', '-'][..]) {
            if part.is_empty() {
                continue;
            }

            if part == "_" {
                tokens.push(VersionToken::Underscore);
            } else if let Ok(num) = part.parse::<u64>() {
                tokens.push(VersionToken::Number(num));
            } else {
                tokens.push(VersionToken::String(part.to_string()));
            }
        }

        tokens
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<&str> for Version {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Version {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        // Compare tokens one by one
        let max_len = self.tokens.len().max(other.tokens.len());

        for i in 0..max_len {
            let self_token = self.tokens.get(i);
            let other_token = other.tokens.get(i);

            match (self_token, other_token) {
                (Some(a), Some(b)) => match a.cmp(b) {
                    Ordering::Equal => continue,
                    other => return other,
                },
                (Some(_), None) => {
                    // self has more tokens, check if remaining tokens are significant
                    // If remaining tokens are all zeros, versions are equal
                    let remaining_significant = self.tokens[i..]
                        .iter()
                        .any(|t| !matches!(t, VersionToken::Number(0) | VersionToken::Underscore));
                    return if remaining_significant {
                        Ordering::Greater
                    } else {
                        Ordering::Equal
                    };
                }
                (None, Some(_)) => {
                    // other has more tokens, check if remaining tokens are significant
                    let remaining_significant = other.tokens[i..]
                        .iter()
                        .any(|t| !matches!(t, VersionToken::Number(0) | VersionToken::Underscore));
                    return if remaining_significant {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    };
                }
                (None, None) => break,
            }
        }

        Ordering::Equal
    }
}

impl Requirement {
    /// Create a new requirement.
    pub fn new(name: impl Into<String>, constraint: VersionConstraint) -> Self {
        Self {
            name: name.into(),
            constraint,
            weak: false,
            conflict: false,
        }
    }

    /// Create a weak requirement.
    pub fn weak(name: impl Into<String>, constraint: VersionConstraint) -> Self {
        Self {
            name: name.into(),
            constraint,
            weak: true,
            conflict: false,
        }
    }

    /// Create a conflict requirement.
    pub fn conflict(name: impl Into<String>, constraint: VersionConstraint) -> Self {
        Self {
            name: name.into(),
            constraint,
            weak: false,
            conflict: true,
        }
    }

    /// Parse a Rez requirement string.
    pub fn parse(requirement_str: &str) -> Result<Self, String> {
        let requirement_str = requirement_str.trim();

        // Handle conflict requirements (starting with !)
        if let Some(stripped) = requirement_str.strip_prefix('!') {
            let (name, constraint) = VersionConstraint::parse(stripped)?;
            return Ok(Self::conflict(name, constraint));
        }

        // Handle weak requirements (starting with ~)
        if let Some(stripped) = requirement_str.strip_prefix('~') {
            let (name, constraint) = VersionConstraint::parse(stripped)?;
            return Ok(Self::weak(name, constraint));
        }

        // Parse normal requirements
        let (name, constraint) = VersionConstraint::parse(requirement_str)?;
        Ok(Self::new(name, constraint))
    }
}

impl std::fmt::Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = if self.conflict {
            "!"
        } else if self.weak {
            "~"
        } else {
            ""
        };

        write!(f, "{}{}", prefix, self.name)?;

        match &self.constraint {
            VersionConstraint::Any => Ok(()),
            VersionConstraint::Exact(v) => write!(f, "=={}", v),
            VersionConstraint::GreaterEqual(v) => write!(f, "-{}+", v),
            VersionConstraint::Less(v) => write!(f, "<{}", v),
            VersionConstraint::Range { min, max } => write!(f, "-{}+<{}", min, max),
            VersionConstraint::Or(constraints) => {
                let constraint_strs: Vec<String> =
                    constraints.iter().map(|c| format!("{:?}", c)).collect();
                write!(f, "|{}", constraint_strs.join("|"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = Version::new("1.2.3");
        assert_eq!(version.value, "1.2.3");
        assert_eq!(version.tokens.len(), 3);

        match &version.tokens[0] {
            VersionToken::Number(n) => assert_eq!(*n, 1),
            _ => panic!("Expected number token"),
        }
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new("1.0.0");
        let v2 = Version::new("1.0.1");
        let v3 = Version::new("1.1.0");

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_requirement_display() {
        let req = Requirement::new(
            "python",
            VersionConstraint::GreaterEqual(Version::new("3.7")),
        );
        assert_eq!(req.to_string(), "python-3.7+");

        let weak_req = Requirement::weak("python", VersionConstraint::Exact(Version::new("3.9")));
        assert_eq!(weak_req.to_string(), "~python==3.9");

        let conflict_req = Requirement::conflict(
            "python",
            VersionConstraint::GreaterEqual(Version::new("4.0")),
        );
        assert_eq!(conflict_req.to_string(), "!python-4.0+");
    }

    #[test]
    fn test_version_tokens() {
        let version = Version::new("1.2.3-alpha.1");
        assert_eq!(version.tokens.len(), 5);

        // Check token types
        match &version.tokens[0] {
            VersionToken::Number(n) => assert_eq!(*n, 1),
            _ => panic!("Expected number token"),
        }

        match &version.tokens[3] {
            VersionToken::String(s) => assert_eq!(s, "alpha"),
            _ => panic!("Expected string token"),
        }
    }

    #[test]
    fn test_version_edge_cases() {
        // Empty version
        let empty = Version::new("");
        assert!(empty.tokens.is_empty());

        // Version with underscores
        let underscore = Version::new("1.0._");
        assert_eq!(underscore.tokens.len(), 3);
        match &underscore.tokens[2] {
            VersionToken::Underscore => {}
            _ => panic!("Expected underscore token"),
        }

        // Complex version
        let complex = Version::new("2.1.0-rc.1+build.123");
        assert!(complex.tokens.len() > 3);
    }

    #[test]
    fn test_requirement_creation() {
        let req = Requirement::new("maya", VersionConstraint::Any);
        assert_eq!(req.name, "maya");
        assert!(!req.weak);
        assert!(!req.conflict);

        let weak = Requirement::weak("houdini", VersionConstraint::Any);
        assert!(weak.weak);
        assert!(!weak.conflict);

        let conflict = Requirement::conflict("nuke", VersionConstraint::Any);
        assert!(!conflict.weak);
        assert!(conflict.conflict);
    }

    #[test]
    fn test_requirement_parsing() {
        // Test basic requirement
        let req = Requirement::parse("python").unwrap();
        assert_eq!(req.name, "python");
        assert_eq!(req.constraint, VersionConstraint::Any);
        assert!(!req.weak);
        assert!(!req.conflict);

        // Test exact version
        let req = Requirement::parse("python==3.9.0").unwrap();
        assert_eq!(req.name, "python");
        assert_eq!(
            req.constraint,
            VersionConstraint::Exact(Version::new("3.9.0"))
        );

        // Test minimum version
        let req = Requirement::parse("python-3.7+").unwrap();
        assert_eq!(req.name, "python");
        assert_eq!(
            req.constraint,
            VersionConstraint::GreaterEqual(Version::new("3.7"))
        );

        // Test conflict requirement
        let req = Requirement::parse("!python-4+").unwrap();
        assert_eq!(req.name, "python");
        assert!(req.conflict);
        assert_eq!(
            req.constraint,
            VersionConstraint::GreaterEqual(Version::new("4"))
        );

        // Test weak requirement
        let req = Requirement::parse("~python-3.8+").unwrap();
        assert_eq!(req.name, "python");
        assert!(req.weak);
        assert_eq!(
            req.constraint,
            VersionConstraint::GreaterEqual(Version::new("3.8"))
        );
    }

    #[test]
    fn test_version_constraint_satisfies() {
        let version = Version::new("3.9.0");

        assert!(VersionConstraint::Any.satisfies(&version));
        assert!(VersionConstraint::Exact(Version::new("3.9.0")).satisfies(&version));
        assert!(!VersionConstraint::Exact(Version::new("3.8.0")).satisfies(&version));
        assert!(VersionConstraint::GreaterEqual(Version::new("3.8")).satisfies(&version));
        assert!(!VersionConstraint::GreaterEqual(Version::new("3.10")).satisfies(&version));
        assert!(VersionConstraint::Less(Version::new("4.0")).satisfies(&version));
        assert!(!VersionConstraint::Less(Version::new("3.9")).satisfies(&version));
    }
}
