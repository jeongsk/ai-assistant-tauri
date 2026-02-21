// Plugin System Module

pub mod loader;
pub mod sandbox;
pub mod api;


use serde::{Deserialize, Serialize};

/// Plugin manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub main: String,
    pub permissions: Vec<PluginPermission>,
    pub api_version: String,
}

/// Plugin permission types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginPermission {
    FileSystem { paths: Vec<String>, access: String },
    Network { hosts: Vec<String> },
    Database { tables: Vec<String> },
    System { capabilities: Vec<String> },
}

/// Plugin state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginState {
    Installed,
    Enabled,
    Running,
    Disabled,
    Error,
}

/// Plugin instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest: String,
    pub permissions: String,
    pub enabled: bool,
    pub installed_at: String,
    pub updated_at: String,
}

/// Plugin execution context
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub plugin_id: String,
    pub permissions: Vec<PluginPermission>,
    pub resource_limits: ResourceLimits,
}

/// Resource limits for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_cpu_percent: u32,
    pub max_execution_time_ms: u32,
    pub max_file_size_mb: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 128,
            max_cpu_percent: 50,
            max_execution_time_ms: 30000,
            max_file_size_mb: 10,
        }
    }
}
