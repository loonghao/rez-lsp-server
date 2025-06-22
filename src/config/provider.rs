//! Configuration provider implementation.

use async_trait::async_trait;
use std::env;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::Config;
use crate::core::{ConfigError, ConfigProvider, Result};

/// Implementation of ConfigProvider that reads from environment variables.
#[derive(Debug)]
pub struct RezConfigProvider {
    config: Config,
}

impl RezConfigProvider {
    /// Create a new configuration provider.
    pub fn new() -> Self {
        Self {
            config: Config::new(),
        }
    }

    /// Load configuration from environment variables.
    ///
    /// This method reads Rez configuration from standard environment variables:
    /// - `REZ_PACKAGES_PATH`: Colon/semicolon-separated list of package directories
    /// - `REZ_LOCAL_PACKAGES_PATH`: Local packages directory (highest priority)
    /// - `REZ_RELEASE_PACKAGES_PATH`: Release packages directory (lowest priority)
    /// - `REZ_LSP_DEBUG`: Enable debug logging (true/1)
    ///
    /// # Errors
    ///
    /// Returns an error if environment variables contain invalid paths or values.
    pub async fn load_from_environment(&mut self) -> Result<()> {
        info!("Loading Rez configuration from environment");

        self.config.packages_path = self.get_packages_path_from_env().await?;
        self.config.local_packages_path = self.get_local_packages_path_from_env().await?;
        self.config.release_packages_path = self.get_release_packages_path_from_env().await?;
        self.config.debug_logging = self.get_debug_logging_from_env().await;

        debug!("Packages path: {:?}", self.config.packages_path);
        debug!("Local packages path: {:?}", self.config.local_packages_path);
        debug!(
            "Release packages path: {:?}",
            self.config.release_packages_path
        );

        Ok(())
    }

    /// Get the current configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get REZ_PACKAGES_PATH from environment.
    async fn get_packages_path_from_env(&self) -> Result<Vec<PathBuf>> {
        match env::var("REZ_PACKAGES_PATH") {
            Ok(path_str) => {
                let paths: Vec<PathBuf> = path_str
                    .split(if cfg!(windows) { ';' } else { ':' })
                    .filter(|s| !s.is_empty())
                    .map(PathBuf::from)
                    .collect();

                if paths.is_empty() {
                    warn!("REZ_PACKAGES_PATH is empty, using default paths");
                    Ok(self.get_default_packages_path())
                } else {
                    info!("Found {} package paths in REZ_PACKAGES_PATH", paths.len());
                    Ok(paths)
                }
            }
            Err(_) => {
                warn!("REZ_PACKAGES_PATH not set, using default paths");
                Ok(self.get_default_packages_path())
            }
        }
    }

    /// Get default package paths when REZ_PACKAGES_PATH is not set.
    fn get_default_packages_path(&self) -> Vec<PathBuf> {
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

    /// Get REZ_LOCAL_PACKAGES_PATH from environment.
    async fn get_local_packages_path_from_env(&self) -> Result<Option<PathBuf>> {
        Ok(env::var("REZ_LOCAL_PACKAGES_PATH").ok().map(PathBuf::from))
    }

    /// Get REZ_RELEASE_PACKAGES_PATH from environment.
    async fn get_release_packages_path_from_env(&self) -> Result<Option<PathBuf>> {
        Ok(env::var("REZ_RELEASE_PACKAGES_PATH")
            .ok()
            .map(PathBuf::from))
    }

    /// Get debug logging setting from environment.
    async fn get_debug_logging_from_env(&self) -> bool {
        env::var("REZ_LSP_DEBUG")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false)
    }
}

#[async_trait]
impl ConfigProvider for RezConfigProvider {
    async fn get_package_paths(&self) -> Result<Vec<PathBuf>> {
        Ok(self.config.packages_path.clone())
    }

    async fn get_local_packages_path(&self) -> Result<Option<PathBuf>> {
        Ok(self.config.local_packages_path.clone())
    }

    async fn get_release_packages_path(&self) -> Result<Option<PathBuf>> {
        Ok(self.config.release_packages_path.clone())
    }

    async fn validate(&self) -> Result<()> {
        self.config.validate()?;

        let mut valid_paths = 0;
        for path in &self.config.packages_path {
            if path.exists() && path.is_dir() {
                valid_paths += 1;
            } else {
                warn!(
                    "Package path does not exist or is not a directory: {:?}",
                    path
                );
            }
        }

        if valid_paths == 0 {
            return Err(ConfigError::NoValidPaths.into());
        }

        info!(
            "Rez configuration validated: {}/{} paths are valid",
            valid_paths,
            self.config.packages_path.len()
        );
        Ok(())
    }

    async fn reload(&mut self) -> Result<()> {
        self.load_from_environment().await
    }
}

impl Default for RezConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_config_provider_creation() {
        let provider = RezConfigProvider::new();
        assert!(provider.config.packages_path.is_empty());
    }

    #[tokio::test]
    async fn test_default_packages_path() {
        let provider = RezConfigProvider::new();
        let paths = provider.get_default_packages_path();
        assert!(!paths.is_empty());
    }

    #[tokio::test]
    async fn test_packages_path_parsing() {
        // Test Unix-style path
        env::set_var("REZ_PACKAGES_PATH", "/path1:/path2:/path3");
        let mut provider = RezConfigProvider::new();
        provider.load_from_environment().await.unwrap();

        if !cfg!(windows) {
            assert_eq!(provider.config.packages_path.len(), 3);
            assert_eq!(provider.config.packages_path[0], PathBuf::from("/path1"));
            assert_eq!(provider.config.packages_path[1], PathBuf::from("/path2"));
            assert_eq!(provider.config.packages_path[2], PathBuf::from("/path3"));
        }

        env::remove_var("REZ_PACKAGES_PATH");
    }
}
