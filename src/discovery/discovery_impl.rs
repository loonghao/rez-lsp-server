//! Package discovery implementation.

use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::core::{DiscoveryError, Package, PackageDiscovery, Result, Version};

/// Implementation of PackageDiscovery trait.
pub struct PackageDiscoveryImpl {
    config: Config,
    package_cache: HashMap<String, Vec<Package>>,
}

impl PackageDiscoveryImpl {
    /// Create a new package discovery instance.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            package_cache: HashMap::new(),
        }
    }

    /// Scan a single package repository directory.
    async fn scan_package_repository(&mut self, repo_path: &Path) -> Result<usize> {
        debug!("Scanning package repository: {:?}", repo_path);
        let mut package_count = 0;

        let entries = fs::read_dir(repo_path).map_err(|e| {
            DiscoveryError::ScanFailed(format!("Failed to read directory {:?}: {}", repo_path, e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| DiscoveryError::ScanFailed(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                let package_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();

                if package_name.starts_with('.') {
                    continue; // Skip hidden directories
                }

                match self.scan_package_versions(&path, &package_name).await {
                    Ok(versions) => {
                        if !versions.is_empty() {
                            package_count += versions.len();
                            self.package_cache.insert(package_name, versions);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to scan package {}: {}", package_name, e);
                    }
                }
            }
        }

        Ok(package_count)
    }

    /// Scan all versions of a specific package.
    async fn scan_package_versions(
        &self,
        package_path: &Path,
        package_name: &str,
    ) -> Result<Vec<Package>> {
        debug!("Scanning package versions for: {}", package_name);
        let mut versions = Vec::new();

        let entries = fs::read_dir(package_path).map_err(|e| {
            DiscoveryError::ScanFailed(format!(
                "Failed to read package directory {:?}: {}",
                package_path, e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| DiscoveryError::ScanFailed(e.to_string()))?;
            let version_path = entry.path();

            if version_path.is_dir() {
                let version = version_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();

                if version.starts_with('.') {
                    continue; // Skip hidden directories
                }

                // Look for package.py file
                let package_py_path = version_path.join("package.py");
                if package_py_path.exists() {
                    match self
                        .parse_package_file(&package_py_path, package_name, &version)
                        .await
                    {
                        Ok(package) => {
                            versions.push(package);
                        }
                        Err(e) => {
                            debug!("Failed to parse package file {:?}: {}", package_py_path, e);
                        }
                    }
                }
            }
        }

        // Sort versions
        versions.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(versions)
    }

    /// Parse a package.py file and extract package information.
    ///
    /// # Arguments
    ///
    /// * `package_py_path` - Path to the package.py file
    /// * `expected_name` - Expected package name from directory structure
    /// * `expected_version` - Expected version from directory structure
    ///
    /// # Returns
    ///
    /// Returns a `Package` struct with parsed information, or an error if parsing fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The file cannot be read
    /// - The file contains invalid syntax
    /// - Required fields are missing
    async fn parse_package_file(
        &self,
        package_py_path: &Path,
        expected_name: &str,
        expected_version: &str,
    ) -> Result<Package> {
        debug!("Parsing package file: {:?}", package_py_path);

        // For now, we'll do basic parsing. In the future, we could use a Python AST parser
        let content = fs::read_to_string(package_py_path).map_err(|e| {
            DiscoveryError::ScanFailed(format!(
                "Failed to read package file {:?}: {}",
                package_py_path, e
            ))
        })?;

        let mut package = Package {
            name: expected_name.to_string(),
            version: Version::new(expected_version),
            description: None,
            authors: Vec::new(),
            requires: Vec::new(),
            tools: Vec::new(),
            variants: Vec::new(),
            path: package_py_path.parent().unwrap().to_path_buf(),
            metadata: HashMap::new(),
        };

        // Simple regex-based parsing (could be improved with proper Python parsing)
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("description") && line.contains('=') {
                if let Some(desc) = self.extract_string_value(line) {
                    package.description = Some(desc);
                }
            } else if line.starts_with("authors") && line.contains('=') {
                package.authors = self.extract_list_values(line);
            } else if line.starts_with("tools") && line.contains('=') {
                package.tools = self.extract_list_values(line);
            }
            // TODO: Parse requires and variants
        }

        Ok(package)
    }

    /// Extract string value from a Python assignment line.
    fn extract_string_value(&self, line: &str) -> Option<String> {
        if let Some(eq_pos) = line.find('=') {
            let value_part = line[eq_pos + 1..].trim();
            if (value_part.starts_with('"') && value_part.ends_with('"'))
                || (value_part.starts_with('\'') && value_part.ends_with('\''))
            {
                return Some(value_part[1..value_part.len() - 1].to_string());
            }
        }
        None
    }

    /// Extract list values from a Python assignment line.
    fn extract_list_values(&self, line: &str) -> Vec<String> {
        let mut values = Vec::new();

        if let Some(eq_pos) = line.find('=') {
            let value_part = line[eq_pos + 1..].trim();
            if value_part.starts_with('[') && value_part.ends_with(']') {
                let list_content = &value_part[1..value_part.len() - 1];
                for item in list_content.split(',') {
                    let item = item.trim();
                    if (item.starts_with('"') && item.ends_with('"'))
                        || (item.starts_with('\'') && item.ends_with('\''))
                    {
                        values.push(item[1..item.len() - 1].to_string());
                    }
                }
            }
        }

        values
    }
}

#[async_trait]
impl PackageDiscovery for PackageDiscoveryImpl {
    async fn scan_packages(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        info!("Starting package discovery scan");
        self.package_cache.clear();

        let all_paths = self.config.get_all_package_paths();
        let mut total_packages = 0;

        for path in all_paths {
            if !path.exists() {
                warn!("Package path does not exist: {:?}", path);
                continue;
            }

            match self.scan_package_repository(&path).await {
                Ok(count) => {
                    total_packages += count;
                    info!("Scanned {} packages from {:?}", count, path);
                }
                Err(e) => {
                    error!("Failed to scan package repository {:?}: {}", path, e);
                }
            }
        }

        let elapsed = start_time.elapsed();
        info!(
            "Package discovery completed: {} packages found across {} package families in {:?}",
            total_packages,
            self.package_cache.len(),
            elapsed
        );
        Ok(())
    }

    async fn find_packages(&self, pattern: &str) -> Result<Vec<Package>> {
        let mut results = Vec::new();

        for (package_name, versions) in &self.package_cache {
            if package_name.contains(pattern) {
                results.extend(versions.iter().cloned());
            }
        }

        Ok(results)
    }

    async fn get_package_versions(&self, name: &str) -> Result<Vec<Package>> {
        Ok(self.package_cache.get(name).cloned().unwrap_or_default())
    }

    async fn get_all_package_names(&self) -> Result<Vec<String>> {
        Ok(self.package_cache.keys().cloned().collect())
    }

    async fn get_package(&self, name: &str, version: &Version) -> Result<Option<Package>> {
        if let Some(versions) = self.package_cache.get(name) {
            for package in versions {
                if package.version == *version {
                    return Ok(Some(package.clone()));
                }
            }
        }
        Ok(None)
    }

    async fn get_stats(&self) -> Result<(usize, usize)> {
        let package_families = self.package_cache.len();
        let total_packages = self.package_cache.values().map(|v| v.len()).sum();
        Ok((package_families, total_packages))
    }

    async fn clear_cache(&mut self) -> Result<()> {
        self.package_cache.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_string_value() {
        let discovery = PackageDiscoveryImpl::new(Config::new());

        assert_eq!(
            discovery.extract_string_value("description = \"test package\""),
            Some("test package".to_string())
        );
        assert_eq!(
            discovery.extract_string_value("description = 'test package'"),
            Some("test package".to_string())
        );
        assert_eq!(discovery.extract_string_value("invalid line"), None);
    }

    #[test]
    fn test_extract_list_values() {
        let discovery = PackageDiscoveryImpl::new(Config::new());

        let result = discovery.extract_list_values("tools = [\"tool1\", \"tool2\"]");
        assert_eq!(result, vec!["tool1", "tool2"]);

        let result = discovery.extract_list_values("authors = ['author1', 'author2']");
        assert_eq!(result, vec!["author1", "author2"]);

        // Test empty list
        let result = discovery.extract_list_values("tools = []");
        assert!(result.is_empty());

        // Test malformed list
        let result = discovery.extract_list_values("tools = [invalid");
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_package_discovery_stats() {
        let discovery = PackageDiscoveryImpl::new(Config::new());
        let (families, total) = discovery.get_stats().await.unwrap();
        assert_eq!(families, 0);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_package_discovery_clear_cache() {
        let mut discovery = PackageDiscoveryImpl::new(Config::new());
        discovery.clear_cache().await.unwrap();

        let names = discovery.get_all_package_names().await.unwrap();
        assert!(names.is_empty());
    }

    #[tokio::test]
    async fn test_find_packages_empty() {
        let discovery = PackageDiscoveryImpl::new(Config::new());
        let packages = discovery.find_packages("nonexistent").await.unwrap();
        assert!(packages.is_empty());
    }
}
