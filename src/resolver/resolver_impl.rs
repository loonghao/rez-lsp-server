//! Dependency resolver implementation.

use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

use crate::core::{
    ContextMetadata, DependencyResolver, Package, PlatformInfo, Requirement, ResolutionStats,
    ResolvedContext, ResolverError, Result, Version, VersionConstraint,
};

/// Implementation of the dependency resolver.
pub struct DependencyResolverImpl {
    /// Available packages indexed by name
    packages: HashMap<String, Vec<Package>>,
    /// Resolution cache for performance
    resolution_cache: HashMap<Vec<Requirement>, Option<ResolvedContext>>,
}

impl DependencyResolverImpl {
    /// Create a new dependency resolver.
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            resolution_cache: HashMap::new(),
        }
    }

    /// Set the available packages for resolution.
    pub fn set_packages(&mut self, packages: HashMap<String, Vec<Package>>) {
        self.packages = packages;
        self.resolution_cache.clear(); // Clear cache when packages change
    }

    /// Find the best version of a package that satisfies the constraint.
    fn find_best_version(&self, name: &str, constraint: &VersionConstraint) -> Option<&Package> {
        let versions = self.packages.get(name)?;

        // Filter versions that satisfy the constraint
        let mut candidates: Vec<&Package> = versions
            .iter()
            .filter(|pkg| constraint.satisfies(&pkg.version))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort by version (highest first)
        candidates.sort_by(|a, b| b.version.cmp(&a.version));

        candidates.first().copied()
    }

    /// Check for conflicts between requirements.
    fn check_conflicts(&self, requirements: &[Requirement]) -> Vec<String> {
        let mut conflicts = Vec::new();
        let mut package_constraints: HashMap<String, Vec<&Requirement>> = HashMap::new();

        // Group requirements by package name
        for req in requirements {
            package_constraints
                .entry(req.name.clone())
                .or_default()
                .push(req);
        }

        // Check for conflicts within each package
        for (package_name, reqs) in package_constraints {
            if reqs.len() > 1 {
                // Check if all constraints can be satisfied simultaneously
                if let Some(versions) = self.packages.get(&package_name) {
                    let satisfying_versions: Vec<&Package> = versions
                        .iter()
                        .filter(|pkg| {
                            reqs.iter().all(|req| {
                                if req.conflict {
                                    !req.constraint.satisfies(&pkg.version)
                                } else {
                                    req.constraint.satisfies(&pkg.version)
                                }
                            })
                        })
                        .collect();

                    if satisfying_versions.is_empty() {
                        conflicts.push(format!(
                            "No version of '{}' satisfies all constraints: {}",
                            package_name,
                            reqs.iter()
                                .map(|r| r.to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
            }

            // Check for explicit conflicts
            for req in &reqs {
                if req.conflict {
                    // This is a conflict requirement - check if any version would be selected
                    if let Some(best_version) =
                        self.find_best_version(&req.name, &VersionConstraint::Any)
                    {
                        if req.constraint.satisfies(&best_version.version) {
                            conflicts.push(format!(
                                "Conflict: package '{}' version '{}' is explicitly forbidden",
                                req.name, best_version.version
                            ));
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// Resolve dependencies recursively.
    fn resolve_recursive(
        &self,
        requirements: &[Requirement],
        resolved: &mut HashMap<String, Package>,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        for req in requirements {
            if req.conflict {
                // Skip conflict requirements in resolution
                continue;
            }

            if visited.contains(&req.name) {
                // Circular dependency detection
                return Err(ResolverError::CircularDependency(format!(
                    "Circular dependency detected involving package '{}'",
                    req.name
                ))
                .into());
            }

            if resolved.contains_key(&req.name) {
                // Already resolved, check compatibility
                let existing = &resolved[&req.name];
                if !req.constraint.satisfies(&existing.version) {
                    return Err(ResolverError::Conflict(format!(
                        "Version conflict for package '{}': existing version '{}' does not satisfy constraint '{}'",
                        req.name, existing.version, req.constraint
                    )).into());
                }
                continue;
            }

            // Find the best version for this requirement
            let package = self
                .find_best_version(&req.name, &req.constraint)
                .ok_or_else(|| {
                    ResolverError::PackageNotFound(format!(
                        "No version of package '{}' satisfies constraint '{}'",
                        req.name, req.constraint
                    ))
                })?;

            debug!("Resolved '{}' to version '{}'", req.name, package.version);

            // Add to visited set to detect circular dependencies
            visited.insert(req.name.clone());

            // Recursively resolve dependencies of this package
            self.resolve_recursive(&package.requires, resolved, visited)?;

            // Remove from visited set
            visited.remove(&req.name);

            // Add to resolved packages
            resolved.insert(req.name.clone(), package.clone());
        }

        Ok(())
    }
}

impl Default for DependencyResolverImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DependencyResolver for DependencyResolverImpl {
    async fn resolve(&self, requirements: &[Requirement]) -> Result<ResolvedContext> {
        let start_time = std::time::Instant::now();
        info!(
            "Starting dependency resolution for {} requirements",
            requirements.len()
        );

        // Check for obvious conflicts first
        let conflicts = self.check_conflicts(requirements);
        if !conflicts.is_empty() {
            return Err(ResolverError::Conflict(conflicts.join("; ")).into());
        }

        let mut resolved = HashMap::new();
        let mut visited = HashSet::new();

        // Resolve all requirements
        self.resolve_recursive(requirements, &mut resolved, &mut visited)?;

        let resolution_time = start_time.elapsed();
        let packages: Vec<Package> = resolved.into_values().collect();
        let packages_count = packages.len();

        info!(
            "Dependency resolution completed: {} packages resolved in {:?}",
            packages_count, resolution_time
        );

        Ok(ResolvedContext {
            packages,
            metadata: ContextMetadata {
                timestamp: chrono::Utc::now(),
                resolver_version: env!("CARGO_PKG_VERSION").to_string(),
                platform: PlatformInfo {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
                },
                stats: ResolutionStats {
                    packages_considered: self.packages.values().map(|v| v.len()).sum(),
                    packages_resolved: packages_count,
                    resolution_time_ms: resolution_time.as_millis() as u64,
                    conflicts: conflicts.len(),
                },
            },
        })
    }

    async fn can_resolve(&self, requirements: &[Requirement]) -> Result<bool> {
        match self.resolve(requirements).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn find_conflicts(
        &self,
        requirements: &[Requirement],
    ) -> Result<Vec<crate::core::DependencyConflict>> {
        let conflicts = self.check_conflicts(requirements);
        Ok(conflicts
            .into_iter()
            .map(|description| crate::core::DependencyConflict {
                package: "unknown".to_string(), // TODO: Extract package name from description
                requirements: requirements.to_vec(),
                description,
            })
            .collect())
    }

    async fn get_latest_version(
        &self,
        name: &str,
        constraint: &VersionConstraint,
    ) -> Result<Option<Version>> {
        Ok(self
            .find_best_version(name, constraint)
            .map(|pkg| pkg.version.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Package, Requirement, Version, VersionConstraint};
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
    async fn test_simple_resolution() {
        let mut resolver = DependencyResolverImpl::new();

        // Set up test packages
        let mut packages = HashMap::new();
        packages.insert(
            "python".to_string(),
            vec![create_test_package("python", "3.9.0", vec![])],
        );

        resolver.set_packages(packages);

        // Test resolution
        let requirements = vec![Requirement::new("python", VersionConstraint::Any)];
        let result = resolver.resolve(&requirements).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.packages.len(), 1);
        assert_eq!(context.packages[0].name, "python");
    }

    #[tokio::test]
    async fn test_version_constraint() {
        let mut resolver = DependencyResolverImpl::new();

        // Set up test packages with multiple versions
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

        // Test resolution with version constraint
        let requirements = vec![Requirement::new(
            "python",
            VersionConstraint::GreaterEqual(Version::new("3.8")),
        )];
        let result = resolver.resolve(&requirements).await;

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.packages.len(), 1);
        assert_eq!(context.packages[0].version, Version::new("3.9.0")); // Should pick the latest
    }

    #[tokio::test]
    async fn test_conflict_detection() {
        let mut resolver = DependencyResolverImpl::new();

        // Set up test packages
        let mut packages = HashMap::new();
        packages.insert(
            "python".to_string(),
            vec![create_test_package("python", "3.9.0", vec![])],
        );

        resolver.set_packages(packages);

        // Test conflicting requirements
        let requirements = vec![
            Requirement::new("python", VersionConstraint::Exact(Version::new("3.9.0"))),
            Requirement::new("python", VersionConstraint::Exact(Version::new("3.8.0"))),
        ];

        let result = resolver.resolve(&requirements).await;
        assert!(result.is_err());
    }
}
