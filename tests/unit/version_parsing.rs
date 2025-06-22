//! Unit tests for version parsing and comparison.

use rez_lsp_server::core::{Version, VersionConstraint, VersionToken, Requirement};

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
fn test_version_constraint_parsing() {
    // Test exact version constraint
    let (name, constraint) = VersionConstraint::parse("python==3.9.0").unwrap();
    assert_eq!(name, "python");
    assert_eq!(constraint, VersionConstraint::Exact(Version::new("3.9.0")));

    // Test minimum version constraint
    let (name, constraint) = VersionConstraint::parse("python-3.7+").unwrap();
    assert_eq!(name, "python");
    assert_eq!(constraint, VersionConstraint::GreaterEqual(Version::new("3.7")));

    // Test maximum version constraint
    let (name, constraint) = VersionConstraint::parse("python<4.0").unwrap();
    assert_eq!(name, "python");
    assert_eq!(constraint, VersionConstraint::Less(Version::new("4.0")));

    // Test range constraint
    let (name, constraint) = VersionConstraint::parse("python-3.7+<4.0").unwrap();
    assert_eq!(name, "python");
    assert_eq!(constraint, VersionConstraint::Range {
        min: Version::new("3.7"),
        max: Version::new("4.0")
    });

    // Test any version
    let (name, constraint) = VersionConstraint::parse("python").unwrap();
    assert_eq!(name, "python");
    assert_eq!(constraint, VersionConstraint::Any);
}

#[test]
fn test_version_constraint_display() {
    assert_eq!(VersionConstraint::Any.to_string(), "*");
    assert_eq!(VersionConstraint::Exact(Version::new("3.9.0")).to_string(), "==3.9.0");
    assert_eq!(VersionConstraint::GreaterEqual(Version::new("3.7")).to_string(), "3.7+");
    assert_eq!(VersionConstraint::Less(Version::new("4.0")).to_string(), "<4.0");
    assert_eq!(VersionConstraint::Range {
        min: Version::new("3.7"),
        max: Version::new("4.0")
    }.to_string(), "3.7+<4.0");
}
