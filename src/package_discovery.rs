use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug, error};

use crate::rez_config::RezConfig;

/// Information about a discovered Rez package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub requires: Vec<String>,
    pub tools: Vec<String>,
}

/// Package discovery engine for scanning local Rez repositories
pub struct PackageDiscovery {
    config: RezConfig,
    package_cache: HashMap<String, Vec<PackageInfo>>,
}

impl PackageDiscovery {
    /// Create a new package discovery instance
    pub fn new(config: RezConfig) -> Self {
        Self {
            config,
            package_cache: HashMap::new(),
        }
    }
    
    /// Scan all package paths and build the package cache
    pub fn scan_packages(&mut self) -> Result<()> {
        info!("Starting package discovery scan");
        self.package_cache.clear();
        
        let all_paths = self.config.get_all_package_paths();
        let mut total_packages = 0;
        
        for path in all_paths {
            if !path.exists() {
                warn!("Package path does not exist: {:?}", path);
                continue;
            }
            
            match self.scan_package_repository(&path) {
                Ok(count) => {
                    total_packages += count;
                    info!("Scanned {} packages from {:?}", count, path);
                }
                Err(e) => {
                    error!("Failed to scan package repository {:?}: {}", path, e);
                }
            }
        }
        
        info!("Package discovery completed: {} packages found across {} package families", 
              total_packages, self.package_cache.len());
        Ok(())
    }
    
    /// Scan a single package repository directory
    fn scan_package_repository(&mut self, repo_path: &Path) -> Result<usize> {
        debug!("Scanning package repository: {:?}", repo_path);
        let mut package_count = 0;
        
        let entries = fs::read_dir(repo_path)
            .with_context(|| format!("Failed to read directory: {:?}", repo_path))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let package_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                
                if package_name.starts_with('.') {
                    continue; // Skip hidden directories
                }
                
                match self.scan_package_versions(&path, &package_name) {
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
    
    /// Scan all versions of a specific package
    fn scan_package_versions(&self, package_path: &Path, package_name: &str) -> Result<Vec<PackageInfo>> {
        debug!("Scanning package versions for: {}", package_name);
        let mut versions = Vec::new();
        
        let entries = fs::read_dir(package_path)
            .with_context(|| format!("Failed to read package directory: {:?}", package_path))?;
        
        for entry in entries {
            let entry = entry?;
            let version_path = entry.path();
            
            if version_path.is_dir() {
                let version = version_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                
                if version.starts_with('.') {
                    continue; // Skip hidden directories
                }
                
                // Look for package.py file
                let package_py_path = version_path.join("package.py");
                if package_py_path.exists() {
                    match self.parse_package_file(&package_py_path, package_name, &version) {
                        Ok(package_info) => {
                            versions.push(package_info);
                        }
                        Err(e) => {
                            debug!("Failed to parse package file {:?}: {}", package_py_path, e);
                        }
                    }
                }
            }
        }
        
        // Sort versions (simple string sort for now, could be improved with proper version comparison)
        versions.sort_by(|a, b| a.version.cmp(&b.version));
        
        Ok(versions)
    }
    
    /// Parse a package.py file and extract package information
    fn parse_package_file(&self, package_py_path: &Path, expected_name: &str, expected_version: &str) -> Result<PackageInfo> {
        debug!("Parsing package file: {:?}", package_py_path);
        
        // For now, we'll do basic parsing. In the future, we could use a Python AST parser
        let content = fs::read_to_string(package_py_path)
            .with_context(|| format!("Failed to read package file: {:?}", package_py_path))?;
        
        let mut package_info = PackageInfo {
            name: expected_name.to_string(),
            version: expected_version.to_string(),
            path: package_py_path.parent().unwrap().to_path_buf(),
            description: None,
            authors: Vec::new(),
            requires: Vec::new(),
            tools: Vec::new(),
        };
        
        // Simple regex-based parsing (could be improved with proper Python parsing)
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("description") && line.contains('=') {
                if let Some(desc) = self.extract_string_value(line) {
                    package_info.description = Some(desc);
                }
            } else if line.starts_with("authors") && line.contains('=') {
                package_info.authors = self.extract_list_values(line);
            } else if line.starts_with("requires") && line.contains('=') {
                package_info.requires = self.extract_list_values(line);
            } else if line.starts_with("tools") && line.contains('=') {
                package_info.tools = self.extract_list_values(line);
            }
        }
        
        Ok(package_info)
    }
    
    /// Extract string value from a Python assignment line
    fn extract_string_value(&self, line: &str) -> Option<String> {
        if let Some(eq_pos) = line.find('=') {
            let value_part = line[eq_pos + 1..].trim();
            if (value_part.starts_with('"') && value_part.ends_with('"')) ||
               (value_part.starts_with('\'') && value_part.ends_with('\'')) {
                return Some(value_part[1..value_part.len()-1].to_string());
            }
        }
        None
    }
    
    /// Extract list values from a Python assignment line
    fn extract_list_values(&self, line: &str) -> Vec<String> {
        let mut values = Vec::new();
        
        if let Some(eq_pos) = line.find('=') {
            let value_part = line[eq_pos + 1..].trim();
            if value_part.starts_with('[') && value_part.ends_with(']') {
                let list_content = &value_part[1..value_part.len()-1];
                for item in list_content.split(',') {
                    let item = item.trim();
                    if (item.starts_with('"') && item.ends_with('"')) ||
                       (item.starts_with('\'') && item.ends_with('\'')) {
                        values.push(item[1..item.len()-1].to_string());
                    }
                }
            }
        }
        
        values
    }
    
    /// Find packages by name (supports partial matching)
    pub fn find_packages(&self, name_pattern: &str) -> Vec<&PackageInfo> {
        let mut results = Vec::new();
        
        for (package_name, versions) in &self.package_cache {
            if package_name.contains(name_pattern) {
                results.extend(versions.iter());
            }
        }
        
        results
    }
    
    /// Get all versions of a specific package
    pub fn get_package_versions(&self, name: &str) -> Option<&Vec<PackageInfo>> {
        self.package_cache.get(name)
    }
    
    /// Get all package names
    pub fn get_all_package_names(&self) -> Vec<String> {
        self.package_cache.keys().cloned().collect()
    }
    
    /// Get package count statistics
    pub fn get_stats(&self) -> (usize, usize) {
        let package_families = self.package_cache.len();
        let total_packages = self.package_cache.values().map(|v| v.len()).sum();
        (package_families, total_packages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_extract_string_value() {
        let discovery = PackageDiscovery::new(RezConfig::from_environment().unwrap());
        
        assert_eq!(discovery.extract_string_value("description = \"test package\""), 
                   Some("test package".to_string()));
        assert_eq!(discovery.extract_string_value("description = 'test package'"), 
                   Some("test package".to_string()));
        assert_eq!(discovery.extract_string_value("invalid line"), None);
    }
    
    #[test]
    fn test_extract_list_values() {
        let discovery = PackageDiscovery::new(RezConfig::from_environment().unwrap());
        
        let result = discovery.extract_list_values("requires = [\"python-3.7+\", \"maya-2020\"]");
        assert_eq!(result, vec!["python-3.7+", "maya-2020"]);
        
        let result = discovery.extract_list_values("tools = ['tool1', 'tool2']");
        assert_eq!(result, vec!["tool1", "tool2"]);
    }
}
