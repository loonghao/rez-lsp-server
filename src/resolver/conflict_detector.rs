//! Conflict detection for dependency resolution.

use std::collections::HashMap;
use tracing::debug;

use crate::core::{DependencyConflict, Package, Requirement, Version, VersionConstraint};

/// Detects conflicts in package requirements.
pub struct ConflictDetector {
    /// Available packages indexed by name
    packages: HashMap<String, Vec<Package>>,
}

impl ConflictDetector {
    /// Create a new conflict detector.
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Set the available packages for conflict detection.
    pub fn set_packages(&mut self, packages: HashMap<String, Vec<Package>>) {
        self.packages = packages;
    }

    /// Detect conflicts in a set of requirements.
    pub fn detect_conflicts(&self, requirements: &[Requirement]) -> Vec<DependencyConflict> {
        let mut conflicts = Vec::new();

        // Group requirements by package name
        let mut package_requirements: HashMap<String, Vec<&Requirement>> = HashMap::new();
        for req in requirements {
            package_requirements
                .entry(req.name.clone())
                .or_default()
                .push(req);
        }

        // Check each package for conflicts
        for (package_name, reqs) in package_requirements {
            conflicts.extend(self.check_package_conflicts(&package_name, &reqs));
        }

        conflicts
    }

    /// Check conflicts for a specific package.
    fn check_package_conflicts(
        &self,
        package_name: &str,
        requirements: &[&Requirement],
    ) -> Vec<DependencyConflict> {
        let mut conflicts = Vec::new();

        if requirements.len() <= 1 {
            return conflicts; // No conflicts possible with single requirement
        }

        debug!(
            "Checking conflicts for package '{}' with {} requirements",
            package_name,
            requirements.len()
        );

        // Get available versions for this package
        let available_versions = match self.packages.get(package_name) {
            Some(versions) => versions,
            None => {
                // Package not found - this is a different kind of error
                return vec![DependencyConflict {
                    package: package_name.to_string(),
                    requirements: requirements.iter().map(|&r| r.clone()).collect(),
                    description: format!("Package '{}' not found", package_name),
                }];
            }
        };

        // Check if any version can satisfy all requirements
        let satisfying_versions: Vec<&Package> = available_versions
            .iter()
            .filter(|pkg| self.version_satisfies_all_requirements(&pkg.version, requirements))
            .collect();

        if satisfying_versions.is_empty() {
            // No version satisfies all requirements - this is a conflict
            conflicts.push(DependencyConflict {
                package: package_name.to_string(),
                requirements: requirements.iter().map(|&r| r.clone()).collect(),
                description: format!(
                    "No version of '{}' satisfies all requirements: {}",
                    package_name,
                    requirements
                        .iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
            // If no version satisfies all requirements, don't check for other conflicts
            return conflicts;
        }

        // Check for explicit conflict requirements
        for req in requirements {
            if req.conflict {
                // This is a conflict requirement - check if it would exclude valid versions
                let excluded_versions: Vec<&Package> = available_versions
                    .iter()
                    .filter(|pkg| req.constraint.satisfies(&pkg.version))
                    .collect();

                if !excluded_versions.is_empty() {
                    conflicts.push(DependencyConflict {
                        package: package_name.to_string(),
                        requirements: vec![(*req).clone()],
                        description: format!(
                            "Conflict requirement '{}' excludes {} available version(s)",
                            req,
                            excluded_versions.len()
                        ),
                    });
                }
            }
        }

        conflicts
    }

    /// Check if a version satisfies all requirements.
    fn version_satisfies_all_requirements(
        &self,
        version: &Version,
        requirements: &[&Requirement],
    ) -> bool {
        for req in requirements {
            if req.conflict {
                // Conflict requirements should NOT be satisfied
                if req.constraint.satisfies(version) {
                    return false;
                }
            } else {
                // Normal requirements should be satisfied
                if !req.constraint.satisfies(version) {
                    return false;
                }
            }
        }
        true
    }

    /// Check if two constraints are mutually exclusive.
    fn are_constraints_mutually_exclusive(
        &self,
        constraint1: &VersionConstraint,
        constraint2: &VersionConstraint,
        available_versions: &[Package],
    ) -> bool {
        // Check if there's any version that satisfies both constraints
        for package in available_versions {
            if constraint1.satisfies(&package.version) && constraint2.satisfies(&package.version) {
                return false; // Found a version that satisfies both
            }
        }
        true // No version satisfies both constraints
    }

    /// Get detailed conflict analysis.
    pub fn analyze_conflicts(&self, requirements: &[Requirement]) -> ConflictAnalysis {
        let conflicts = self.detect_conflicts(requirements);
        let total_packages = requirements
            .iter()
            .map(|r| &r.name)
            .collect::<std::collections::HashSet<_>>()
            .len();

        ConflictAnalysis {
            total_requirements: requirements.len(),
            total_packages,
            conflicts: conflicts.clone(),
            has_conflicts: !conflicts.is_empty(),
            severity: if conflicts.is_empty() {
                ConflictSeverity::None
            } else if conflicts.len() == 1 {
                ConflictSeverity::Minor
            } else if conflicts.len() <= 3 {
                ConflictSeverity::Moderate
            } else {
                ConflictSeverity::Severe
            },
        }
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis result for conflicts.
#[derive(Debug, Clone)]
pub struct ConflictAnalysis {
    /// Total number of requirements analyzed
    pub total_requirements: usize,
    /// Total number of unique packages
    pub total_packages: usize,
    /// Detected conflicts
    pub conflicts: Vec<DependencyConflict>,
    /// Whether any conflicts were found
    pub has_conflicts: bool,
    /// Severity of conflicts
    pub severity: ConflictSeverity,
}

/// Severity levels for conflicts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictSeverity {
    /// No conflicts
    None,
    /// Minor conflicts (1 conflict)
    Minor,
    /// Moderate conflicts (2-3 conflicts)
    Moderate,
    /// Severe conflicts (4+ conflicts)
    Severe,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Package, Version, VersionConstraint};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_package(name: &str, version: &str) -> Package {
        Package {
            name: name.to_string(),
            version: Version::new(version),
            description: Some(format!("Test package {}", name)),
            authors: vec!["Test Author".to_string()],
            requires: vec![],
            tools: vec![],
            variants: vec![],
            path: PathBuf::from("/test"),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_no_conflicts() {
        let mut detector = ConflictDetector::new();

        let mut packages = HashMap::new();
        packages.insert(
            "python".to_string(),
            vec![
                create_test_package("python", "3.7.0"),
                create_test_package("python", "3.8.0"),
                create_test_package("python", "3.9.0"),
            ],
        );
        detector.set_packages(packages);

        let requirements = vec![Requirement::new(
            "python",
            VersionConstraint::GreaterEqual(Version::new("3.7")),
        )];

        let conflicts = detector.detect_conflicts(&requirements);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_version_conflict() {
        let mut detector = ConflictDetector::new();

        let mut packages = HashMap::new();
        packages.insert(
            "python".to_string(),
            vec![
                create_test_package("python", "3.7.0"),
                create_test_package("python", "3.8.0"),
                create_test_package("python", "3.9.0"),
            ],
        );
        detector.set_packages(packages);

        let requirements = vec![
            Requirement::new("python", VersionConstraint::Exact(Version::new("3.7.0"))),
            Requirement::new("python", VersionConstraint::Exact(Version::new("3.9.0"))),
        ];

        let conflicts = detector.detect_conflicts(&requirements);
        assert!(!conflicts.is_empty());
        assert_eq!(conflicts.len(), 1);
    }

    #[test]
    fn test_conflict_analysis() {
        let mut detector = ConflictDetector::new();

        let mut packages = HashMap::new();
        packages.insert(
            "python".to_string(),
            vec![create_test_package("python", "3.9.0")],
        );
        detector.set_packages(packages);

        let requirements = vec![
            Requirement::new("python", VersionConstraint::Exact(Version::new("3.7.0"))),
            Requirement::new("python", VersionConstraint::Exact(Version::new("3.9.0"))),
        ];

        let analysis = detector.analyze_conflicts(&requirements);
        assert!(analysis.has_conflicts);
        assert_eq!(analysis.severity, ConflictSeverity::Minor);
        assert_eq!(analysis.total_requirements, 2);
        assert_eq!(analysis.total_packages, 1);
    }
}
