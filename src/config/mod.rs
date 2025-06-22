//! Configuration management for the Rez LSP server.

mod provider;

pub use provider::RezConfigProvider;

use crate::core::{ConfigError, Result};
use std::path::PathBuf;

/// Configuration for the Rez LSP server.
#[derive(Debug, Clone)]
pub struct Config {
    /// Package search paths
    pub packages_path: Vec<PathBuf>,
    /// Local packages path
    pub local_packages_path: Option<PathBuf>,
    /// Release packages path
    pub release_packages_path: Option<PathBuf>,
    /// Maximum number of packages to cache
    pub max_cache_size: usize,
    /// Cache expiration time in seconds
    pub cache_expiration_secs: u64,
    /// Enable debug logging
    pub debug_logging: bool,
}

impl Config {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self {
            packages_path: Vec::new(),
            local_packages_path: None,
            release_packages_path: None,
            max_cache_size: 10000,
            cache_expiration_secs: 3600, // 1 hour
            debug_logging: false,
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.packages_path.is_empty() {
            return Err(ConfigError::NoValidPaths.into());
        }

        let mut valid_paths = 0;
        for path in &self.packages_path {
            if path.exists() && path.is_dir() {
                valid_paths += 1;
            }
        }

        if valid_paths == 0 {
            return Err(ConfigError::NoValidPaths.into());
        }

        Ok(())
    }

    /// Get all package search paths in priority order.
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
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
