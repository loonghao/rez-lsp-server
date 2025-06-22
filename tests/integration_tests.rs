//! Integration tests for Rez LSP Server.

use rez_lsp_server::core::DependencyResolver;
use rez_lsp_server::core::{Package, Requirement, Version, VersionConstraint};
use rez_lsp_server::resolver::DependencyResolverImpl;
use std::collections::HashMap;
use std::path::PathBuf;

fn create_test_package(name: &str, version: &str, requires: Vec<Requirement>) -> Package {
    Package {
        name: name.to_string(),
        version: Version::new(version),
        description: Some(format!("Test package {}", name)),
        authors: vec!["Test Author".to_string()],
        requires,
        tools: vec![],
        variants: vec![],
        path: PathBuf::from("/test"),
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_simple_dependency_resolution() {
    let mut resolver = DependencyResolverImpl::new();

    // Set up test packages
    let mut packages = HashMap::new();
    packages.insert(
        "python".to_string(),
        vec![create_test_package("python", "3.9.0", vec![])],
    );
    packages.insert(
        "maya".to_string(),
        vec![create_test_package(
            "maya",
            "2024.0",
            vec![Requirement::new(
                "python",
                VersionConstraint::GreaterEqual(Version::new("3.7")),
            )],
        )],
    );

    resolver.set_packages(packages);

    // Test resolution
    let requirements = vec![Requirement::new("maya", VersionConstraint::Any)];
    let result = resolver.resolve(&requirements).await;

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.packages.len(), 2); // maya + python

    // Check that both packages are resolved
    let package_names: Vec<&str> = context.packages.iter().map(|p| p.name.as_str()).collect();
    assert!(package_names.contains(&"maya"));
    assert!(package_names.contains(&"python"));
}

#[tokio::test]
async fn test_version_constraint_satisfaction() {
    let mut resolver = DependencyResolverImpl::new();

    // Set up packages with multiple versions
    let mut packages = HashMap::new();
    packages.insert(
        "python".to_string(),
        vec![
            create_test_package("python", "3.7.0", vec![]),
            create_test_package("python", "3.8.0", vec![]),
            create_test_package("python", "3.9.0", vec![]),
            create_test_package("python", "3.10.0", vec![]),
        ],
    );

    resolver.set_packages(packages);

    // Test resolution with version constraint
    let requirements = vec![Requirement::new(
        "python",
        VersionConstraint::GreaterEqual(Version::new("3.8")),
    )];
    let result = resolver.resolve(&requirements).await;

    assert!(result.is_ok());
    let context = result.unwrap();
    assert_eq!(context.packages.len(), 1);

    // Should pick the latest version that satisfies the constraint
    let python_package = &context.packages[0];
    assert_eq!(python_package.name, "python");
    // The resolver should pick the highest version >= 3.8
    assert!(python_package.version >= Version::new("3.8.0"));
    // In our test data, this should be 3.10.0
    assert_eq!(python_package.version, Version::new("3.10.0"));
}

#[tokio::test]
async fn test_version_conflict_detection() {
    let mut resolver = DependencyResolverImpl::new();

    // Set up packages with conflicting version requirements
    let mut packages = HashMap::new();
    packages.insert(
        "python".to_string(),
        vec![
            create_test_package("python", "3.7.0", vec![]),
            create_test_package("python", "3.8.0", vec![]),
            create_test_package("python", "3.9.0", vec![]),
        ],
    );

    resolver.set_packages(packages);

    // Test conflicting requirements
    let requirements = vec![
        Requirement::new("python", VersionConstraint::Exact(Version::new("3.7.0"))),
        Requirement::new("python", VersionConstraint::Exact(Version::new("3.9.0"))),
    ];

    let result = resolver.resolve(&requirements).await;
    assert!(result.is_err()); // Should fail due to conflict
}
