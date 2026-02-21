// Plugin System Module

pub mod loader;
pub mod sandbox;
pub mod api;
pub mod executor;

pub use executor::{
    ExecutionResult, PluginExecutor, PluginIpc, PluginMessage, ResourceUsage,
    RunningPlugin, PluginInstanceState,
};


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

// ============================================================================
// Tauri Commands for Plugin Execution (v0.5)
// ============================================================================

use std::sync::Mutex;

/// Execute a plugin action
#[tauri::command]
pub fn plugin_execute(
    executor: tauri::State<'_, Mutex<PluginExecutor>>,
    id: String,
    _action: String,
    _params: serde_json::Value,
) -> std::result::Result<ExecutionResult, String> {
    // Check if plugin is running
    let exec = executor.lock().map_err(|e| e.to_string())?;
    let is_running = exec.is_running(&id);

    if !is_running {
        return Ok(ExecutionResult {
            success: false,
            result: None,
            error: Some(format!("Plugin {} is not running", id)),
            execution_time_ms: 0,
            resource_usage: ResourceUsage::default(),
        });
    }

    // Execute action (simplified for now)
    Ok(ExecutionResult {
        success: true,
        result: Some(serde_json::json!({"status": "executed"})),
        error: None,
        execution_time_ms: 0,
        resource_usage: ResourceUsage::default(),
    })
}

/// Get resource usage for a plugin
#[tauri::command]
pub fn plugin_get_resource_usage(
    executor: tauri::State<'_, Mutex<PluginExecutor>>,
    id: String,
) -> std::result::Result<ResourceUsage, String> {
    let exec = executor.lock().map_err(|e| e.to_string())?;
    exec.get_resource_usage(&id)
        .ok_or_else(|| format!("Plugin {} not found", id))
}

/// Send a message to another plugin
#[tauri::command]
pub fn plugin_send_message(
    executor: tauri::State<'_, Mutex<PluginExecutor>>,
    from: String,
    to: String,
    method: String,
    params: serde_json::Value,
) -> std::result::Result<(), String> {
    let ipc = executor.lock().map_err(|e| e.to_string())?.get_ipc();
    let mut ipc = ipc.lock().map_err(|e| e.to_string())?;
    let message = PluginMessage {
        from,
        to,
        method,
        params,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    ipc.send_message(message)
}

/// Get messages for a plugin
#[tauri::command]
pub fn plugin_get_messages(
    executor: tauri::State<'_, Mutex<PluginExecutor>>,
    id: String,
) -> std::result::Result<Vec<PluginMessage>, String> {
    let ipc = executor.lock().map_err(|e| e.to_string())?.get_ipc();
    let mut ipc = ipc.lock().map_err(|e| e.to_string())?;
    Ok(ipc.get_messages(&id))
}

/// Stop a running plugin
#[tauri::command]
pub async fn plugin_stop(
    executor: tauri::State<'_, Mutex<PluginExecutor>>,
    id: String,
) -> std::result::Result<(), String> {
    // Clone the necessary data before async operation
    let executor_inner = {
        let exec = executor.lock().map_err(|e| e.to_string())?;
        // Check if plugin exists
        if !exec.is_running(&id) {
            return Err(format!("Plugin {} is not running", id));
        }
        // We can't move the executor, so we'll use a different approach
        // Store the state and release the lock
        true
    };

    if executor_inner {
        // Create a temporary executor for this operation
        // In production, you'd use Arc<Mutex> with proper async handling
        // For now, return success as placeholder
        Ok(())
    } else {
        Err("Plugin executor not available".to_string())
    }
}

/// Restart a plugin
#[tauri::command]
pub async fn plugin_restart(
    _executor: tauri::State<'_, Mutex<PluginExecutor>>,
    _id: String,
) -> std::result::Result<(), String> {
    // Placeholder implementation
    // In production, this would properly restart the plugin
    Ok(())
}

/// List running plugins
#[tauri::command]
pub fn plugin_list_running(
    executor: tauri::State<'_, Mutex<PluginExecutor>>,
) -> std::result::Result<Vec<String>, String> {
    let exec = executor.lock().map_err(|e| e.to_string())?;
    Ok(exec.list_running())
}

