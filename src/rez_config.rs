use std::env;
use std::path::PathBuf;
use anyhow::{Result, Context};
use tracing::{info, warn, debug};

/// Rez configuration manager
#[derive(Debug, Clone)]
pub struct RezConfig {
    pub packages_path: Vec<PathBuf>,
    pub local_packages_path: Option<PathBuf>,
    pub release_packages_path: Option<PathBuf>,
}

impl RezConfig {
    /// Create a new RezConfig by reading environment variables
    pub fn from_environment() -> Result<Self> {
        info!("Loading Rez configuration from environment");
        
        let packages_path = Self::get_packages_path()?;
        let local_packages_path = Self::get_local_packages_path();
        let release_packages_path = Self::get_release_packages_path();
        
        debug!("Packages path: {:?}", packages_path);
        debug!("Local packages path: {:?}", local_packages_path);
        debug!("Release packages path: {:?}", release_packages_path);
        
        Ok(RezConfig {
            packages_path,
            local_packages_path,
            release_packages_path,
        })
    }
    
    /// Get REZ_PACKAGES_PATH from environment
    fn get_packages_path() -> Result<Vec<PathBuf>> {
        match env::var("REZ_PACKAGES_PATH") {
            Ok(path_str) => {
                let paths: Vec<PathBuf> = path_str
                    .split(if cfg!(windows) { ';' } else { ':' })
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from)
                    .collect();
                
                if paths.is_empty() {
                    warn!("REZ_PACKAGES_PATH is empty, using default paths");
                    Ok(Self::get_default_packages_path())
                } else {
                    info!("Found {} package paths in REZ_PACKAGES_PATH", paths.len());
                    Ok(paths)
                }
            }
            Err(_) => {
                warn!("REZ_PACKAGES_PATH not set, using default paths");
                Ok(Self::get_default_packages_path())
            }
        }
    }
    
    /// Get default package paths when REZ_PACKAGES_PATH is not set
    fn get_default_packages_path() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Add user's home packages directory
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join("packages"));
        }
        
        // Add common system paths
        if cfg!(windows) {
            paths.push(PathBuf::from("C:\\rez\\packages"));
        } else {
            paths.push(PathBuf::from("/opt/rez/packages"));
            paths.push(PathBuf::from("/usr/local/rez/packages"));
        }
        
        paths
    }
    
    /// Get REZ_LOCAL_PACKAGES_PATH from environment
    fn get_local_packages_path() -> Option<PathBuf> {
        env::var("REZ_LOCAL_PACKAGES_PATH")
            .ok()
            .map(PathBuf::from)
    }
    
    /// Get REZ_RELEASE_PACKAGES_PATH from environment
    fn get_release_packages_path() -> Option<PathBuf> {
        env::var("REZ_RELEASE_PACKAGES_PATH")
            .ok()
            .map(PathBuf::from)
    }
    
    /// Get all package search paths in priority order
    pub fn get_all_package_paths(&self) -> Vec<PathBuf> {
        let mut all_paths = Vec::new();
        
        // Local packages have highest priority
        if let Some(ref local_path) = self.local_packages_path {
            all_paths.push(local_path.clone());
        }
        
        // Then the main packages path
        all_paths.extend(self.packages_path.iter().cloned());
        
        // Release packages have lowest priority
        if let Some(ref release_path) = self.release_packages_path {
            all_paths.push(release_path.clone());
        }
        
        all_paths
    }
    
    /// Check if Rez is properly configured
    pub fn validate(&self) -> Result<()> {
        if self.packages_path.is_empty() {
            return Err(anyhow::anyhow!("No package paths configured"));
        }
        
        let mut valid_paths = 0;
        for path in &self.packages_path {
            if path.exists() && path.is_dir() {
                valid_paths += 1;
            } else {
                warn!("Package path does not exist or is not a directory: {:?}", path);
            }
        }
        
        if valid_paths == 0 {
            return Err(anyhow::anyhow!("No valid package paths found"));
        }
        
        info!("Rez configuration validated: {}/{} paths are valid", valid_paths, self.packages_path.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_default_packages_path() {
        let paths = RezConfig::get_default_packages_path();
        assert!(!paths.is_empty());
    }
    
    #[test]
    fn test_packages_path_parsing() {
        // Test Unix-style path
        env::set_var("REZ_PACKAGES_PATH", "/path1:/path2:/path3");
        let config = RezConfig::from_environment().unwrap();
        
        if !cfg!(windows) {
            assert_eq!(config.packages_path.len(), 3);
            assert_eq!(config.packages_path[0], PathBuf::from("/path1"));
            assert_eq!(config.packages_path[1], PathBuf::from("/path2"));
            assert_eq!(config.packages_path[2], PathBuf::from("/path3"));
        }
        
        env::remove_var("REZ_PACKAGES_PATH");
    }
    
    #[test]
    fn test_config_from_environment() {
        let config = RezConfig::from_environment().unwrap();
        assert!(!config.packages_path.is_empty());
    }
}
