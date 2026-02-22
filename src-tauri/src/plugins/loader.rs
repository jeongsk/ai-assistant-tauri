// Plugin Loader - Loads and validates plugins

#![allow(dead_code)]

use crate::plugins::{PluginManifest, PluginPermission, ResourceLimits};
use std::path::PathBuf;
use std::collections::HashMap;

/// Plugin loader configuration
pub struct LoaderConfig {
    pub plugins_dir: PathBuf,
    pub allow_unsigned: bool,
    pub default_limits: ResourceLimits,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            plugins_dir: PathBuf::from("plugins"),
            allow_unsigned: false,
            default_limits: ResourceLimits::default(),
        }
    }
}

/// Plugin loader
pub struct PluginLoader {
    config: LoaderConfig,
    loaded_plugins: HashMap<String, PluginManifest>,
}

impl PluginLoader {
    pub fn new(config: LoaderConfig) -> Self {
        Self {
            config,
            loaded_plugins: HashMap::new(),
        }
    }

    /// Load plugin from directory
    pub fn load_from_dir(&mut self, path: &std::path::Path) -> Result<PluginManifest, String> {
        let manifest_path = path.join("plugin.json");

        // Read manifest
        let manifest_content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| format!("Invalid manifest: {}", e))?;

        // Validate manifest
        self.validate_manifest(&manifest)?;

        // Store loaded plugin
        self.loaded_plugins.insert(manifest.id.clone(), manifest.clone());

        Ok(manifest)
    }

    /// Validate plugin manifest
    fn validate_manifest(&self, manifest: &PluginManifest) -> Result<(), String> {
        if manifest.id.is_empty() {
            return Err("Plugin ID is required".to_string());
        }

        if manifest.name.is_empty() {
            return Err("Plugin name is required".to_string());
        }

        if manifest.version.is_empty() {
            return Err("Plugin version is required".to_string());
        }

        if manifest.api_version != "1.0" {
            return Err(format!("Unsupported API version: {}", manifest.api_version));
        }

        // Validate permissions
        for perm in &manifest.permissions {
            self.validate_permission(perm)?;
        }

        Ok(())
    }

    /// Validate individual permission
    fn validate_permission(&self, permission: &PluginPermission) -> Result<(), String> {
        match permission {
            PluginPermission::FileSystem { paths, access } => {
                if paths.is_empty() {
                    return Err("FileSystem permission requires at least one path".to_string());
                }
                if access != "read" && access != "readwrite" {
                    return Err("FileSystem access must be 'read' or 'readwrite'".to_string());
                }
            }
            PluginPermission::Network { hosts } => {
                if hosts.is_empty() {
                    return Err("Network permission requires at least one host".to_string());
                }
            }
            PluginPermission::Database { tables } => {
                if tables.is_empty() {
                    return Err("Database permission requires at least one table".to_string());
                }
            }
            PluginPermission::System { capabilities } => {
                let allowed = ["notifications", "clipboard", "shell"];
                for cap in capabilities {
                    if !allowed.contains(&cap.as_str()) {
                        return Err(format!("System capability '{}' not allowed", cap));
                    }
                }
            }
        }
        Ok(())
    }

    /// Get loaded plugin
    pub fn get_plugin(&self, id: &str) -> Option<&PluginManifest> {
        self.loaded_plugins.get(id)
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<&PluginManifest> {
        self.loaded_plugins.values().collect()
    }

    /// Unload plugin
    pub fn unload_plugin(&mut self, id: &str) -> bool {
        self.loaded_plugins.remove(id).is_some()
    }

    /// Check plugin compatibility
    pub fn check_compatibility(&self, manifest: &PluginManifest) -> Result<(), String> {
        // Check API version
        if manifest.api_version != "1.0" {
            return Err(format!("Incompatible API version: {}", manifest.api_version));
        }

        Ok(())
    }
}

/// Discover plugins in directory
pub fn discover_plugins(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut plugins = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("plugin.json").exists() {
                plugins.push(path);
            }
        }
    }

    plugins
}
