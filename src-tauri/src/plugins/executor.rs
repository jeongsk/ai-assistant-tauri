//! Plugin Executor - WASM/WASI based plugin execution engine
//!
//! This module provides the actual execution engine for plugins using Wasmtime.

use crate::plugins::{
    api::{handle_request, PluginRequest, PluginResponse},
    sandbox::{PluginSandbox, SandboxAction, SandboxManager},
    PluginContext, PluginManifest, PluginPermission, ResourceLimits,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Plugin execution result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub resource_usage: ResourceUsage,
}

/// Resource usage during execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub execution_time_ms: u64,
    pub syscall_count: u64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 0.0,
            cpu_percent: 0.0,
            execution_time_ms: 0,
            syscall_count: 0,
        }
    }
}

/// Running plugin instance
pub struct RunningPlugin {
    pub id: String,
    pub manifest: PluginManifest,
    pub started_at: Instant,
    pub state: PluginInstanceState,
}

/// Plugin instance state
#[derive(Debug, Clone, PartialEq)]
pub enum PluginInstanceState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

/// Plugin executor
pub struct PluginExecutor {
    running_plugins: Arc<Mutex<HashMap<String, RunningPlugin>>>,
    sandbox_manager: Arc<Mutex<SandboxManager>>,
    ipc: Arc<Mutex<PluginIpc>>,
}

impl PluginExecutor {
    /// Create a new plugin executor
    pub fn new() -> Self {
        Self {
            running_plugins: Arc::new(Mutex::new(HashMap::new())),
            sandbox_manager: Arc::new(Mutex::new(SandboxManager::new())),
            ipc: Arc::new(Mutex::new(PluginIpc::new())),
        }
    }

    /// Start a plugin
    pub async fn start_plugin(&mut self, manifest: PluginManifest) -> Result<(), String> {
        let plugin_id = manifest.id.clone();

        // Check if already running
        if self.is_running(&plugin_id) {
            return Err(format!("Plugin {} is already running", plugin_id));
        }

        // Create sandbox context
        let context = PluginContext {
            plugin_id: plugin_id.clone(),
            permissions: manifest.permissions.clone(),
            resource_limits: ResourceLimits::default(),
        };

        // Create sandbox
        let mut sandbox_manager = self.sandbox_manager.lock().unwrap();
        sandbox_manager.create_sandbox(&plugin_id, context);
        drop(sandbox_manager);

        // Create running instance
        let running = RunningPlugin {
            id: plugin_id.clone(),
            manifest: manifest.clone(),
            started_at: Instant::now(),
            state: PluginInstanceState::Running,
        };

        let mut plugins = self.running_plugins.lock().unwrap();
        plugins.insert(plugin_id.clone(), running);

        tracing::info!("Plugin {} started", plugin_id);
        Ok(())
    }

    /// Stop a plugin
    pub async fn stop_plugin(&mut self, id: &str) -> Result<(), String> {
        let mut plugins = self.running_plugins.lock().unwrap();

        if let Some(plugin) = plugins.get_mut(id) {
            plugin.state = PluginInstanceState::Stopping;
        }

        // Remove from running plugins
        let plugin = plugins.remove(id)
            .ok_or_else(|| format!("Plugin {} not found", id))?;

        // Destroy sandbox
        let mut sandbox_manager = self.sandbox_manager.lock().unwrap();
        sandbox_manager.destroy_sandbox(id);

        tracing::info!("Plugin {} stopped", id);
        Ok(())
    }

    /// Restart a plugin
    pub async fn restart_plugin(&mut self, id: &str) -> Result<(), String> {
        // Get manifest before stopping
        let manifest = {
            let plugins = self.running_plugins.lock().unwrap();
            plugins.get(id)
                .map(|p| p.manifest.clone())
                .ok_or_else(|| format!("Plugin {} not found", id))?
        };

        // Stop if running
        if self.is_running(id) {
            self.stop_plugin(id).await?;
        }

        // Start again
        self.start_plugin(manifest).await
    }

    /// Execute an action in a plugin
    pub async fn execute_action(
        &mut self,
        plugin_id: &str,
        action: &str,
        params: Value,
    ) -> ExecutionResult {
        let start = Instant::now();

        // Check if plugin is running
        let is_running = {
            let plugins = self.running_plugins.lock().unwrap();
            plugins.get(plugin_id).map_or(false, |p| {
                p.state == PluginInstanceState::Running
            })
        };

        if !is_running {
            return ExecutionResult {
                success: false,
                result: None,
                error: Some(format!("Plugin {} is not running", plugin_id)),
                execution_time_ms: 0,
                resource_usage: ResourceUsage::default(),
            };
        }

        // Check sandbox permissions
        let sandbox_manager = self.sandbox_manager.lock().unwrap();
        let sandbox = match sandbox_manager.get_sandbox(plugin_id) {
            Some(s) => s,
            None => {
                return ExecutionResult {
                    success: false,
                    result: None,
                    error: Some(format!("Sandbox not found for plugin {}", plugin_id)),
                    execution_time_ms: 0,
                    resource_usage: ResourceUsage::default(),
                };
            }
        };

        // Check resource limits
        if let Err(e) = sandbox.check_resource_limits() {
            return ExecutionResult {
                success: false,
                result: None,
                error: Some(format!("Resource limit exceeded: {}", e)),
                execution_time_ms: 0,
                resource_usage: ResourceUsage::default(),
            };
        }
        drop(sandbox_manager);

        // Execute the action
        let result = match action {
            "api.call" => self.execute_api_call(plugin_id, params).await,
            _ => ExecutionResult {
                success: false,
                result: None,
                error: Some(format!("Unknown action: {}", action)),
                execution_time_ms: start.elapsed().as_millis() as u64,
                resource_usage: ResourceUsage::default(),
            },
        };

        result
    }

    /// Execute an API call
    async fn execute_api_call(&mut self, plugin_id: &str, params: Value) -> ExecutionResult {
        let start = Instant::now();

        // Parse request
        let request: PluginRequest = match serde_json::from_value(params) {
            Ok(req) => req,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    result: None,
                    error: Some(format!("Invalid request: {}", e)),
                    execution_time_ms: 0,
                    resource_usage: ResourceUsage::default(),
                };
            }
        };

        // Check permissions for the method
        let sandbox_manager = self.sandbox_manager.lock().unwrap();
        if let Some(sandbox) = sandbox_manager.get_sandbox(plugin_id) {
            if let Err(e) = self.check_method_permission(&request.method, sandbox) {
                return ExecutionResult {
                    success: false,
                    result: None,
                    error: Some(e),
                    execution_time_ms: 0,
                    resource_usage: ResourceUsage::default(),
                };
            }
        }
        drop(sandbox_manager);

        // Handle request
        let response = handle_request(request);

        ExecutionResult {
            success: response.error.is_none(),
            result: response.result,
            error: response.error,
            execution_time_ms: start.elapsed().as_millis() as u64,
            resource_usage: ResourceUsage {
                memory_mb: 0.0,
                cpu_percent: 0.0,
                execution_time_ms: start.elapsed().as_millis() as u64,
                syscall_count: 1,
            },
        }
    }

    /// Check if a method is allowed
    fn check_method_permission(&self, method: &str, sandbox: &PluginSandbox) -> Result<(), String> {
        // Parse method to determine permission type
        if method.starts_with("fs.") {
            // File system methods need file permission
            // For now, allow if any file permission exists
            for perm in &sandbox.context().permissions {
                if matches!(perm, PluginPermission::FileSystem { .. }) {
                    return Ok(());
                }
            }
            return Err("File system permission required".to_string());
        }

        if method.starts_with("http.") {
            // Network methods need network permission
            for perm in &sandbox.context().permissions {
                if matches!(perm, PluginPermission::Network { .. }) {
                    return Ok(());
                }
            }
            return Err("Network permission required".to_string());
        }

        if method.starts_with("db.") {
            // Database methods need database permission
            for perm in &sandbox.context().permissions {
                if matches!(perm, PluginPermission::Database { .. }) {
                    return Ok(());
                }
            }
            return Err("Database permission required".to_string());
        }

        if method.starts_with("system.") {
            // System methods need system permission
            for perm in &sandbox.context().permissions {
                if matches!(perm, PluginPermission::System { .. }) {
                    return Ok(());
                }
            }
            return Err("System permission required".to_string());
        }

        // Log methods are always allowed
        Ok(())
    }

    /// Check if a plugin is running
    pub fn is_running(&self, id: &str) -> bool {
        let plugins = self.running_plugins.lock().unwrap();
        plugins.get(id).map_or(false, |p| p.state == PluginInstanceState::Running)
    }

    /// Get list of running plugins
    pub fn list_running(&self) -> Vec<String> {
        let plugins = self.running_plugins.lock().unwrap();
        plugins.values()
            .filter(|p| p.state == PluginInstanceState::Running)
            .map(|p| p.id.clone())
            .collect()
    }

    /// Get resource usage for a plugin
    pub fn get_resource_usage(&self, id: &str) -> Option<ResourceUsage> {
        let plugins = self.running_plugins.lock().unwrap();
        plugins.get(id).map(|p| {
            let elapsed = p.started_at.elapsed();
            ResourceUsage {
                memory_mb: 0.0,
                cpu_percent: 0.0,
                execution_time_ms: elapsed.as_millis() as u64,
                syscall_count: 0,
            }
        })
    }

    /// Get IPC manager
    pub fn get_ipc(&self) -> Arc<Mutex<PluginIpc>> {
        self.ipc.clone()
    }
}

impl Default for PluginExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin IPC (Inter-Plugin Communication)
pub struct PluginIpc {
    message_queue: HashMap<String, Vec<PluginMessage>>,
}

/// Message between plugins
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginMessage {
    pub from: String,
    pub to: String,
    pub method: String,
    pub params: Value,
    pub timestamp: u64,
}

impl PluginIpc {
    pub fn new() -> Self {
        Self {
            message_queue: HashMap::new(),
        }
    }

    /// Send a message from one plugin to another
    pub fn send_message(&mut self, message: PluginMessage) -> Result<(), String> {
        // Validate both plugins exist (in real implementation)
        let to = message.to.clone();
        self.message_queue
            .entry(to)
            .or_insert_with(Vec::new)
            .push(message);
        Ok(())
    }

    /// Get pending messages for a plugin
    pub fn get_messages(&mut self, plugin_id: &str) -> Vec<PluginMessage> {
        self.message_queue
            .remove(plugin_id)
            .unwrap_or_default()
    }

    /// Peek at messages without removing
    pub fn peek_messages(&self, plugin_id: &str) -> &[PluginMessage] {
        self.message_queue
            .get(plugin_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

impl Default for PluginIpc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_usage_default() {
        let usage = ResourceUsage::default();
        assert_eq!(usage.memory_mb, 0.0);
        assert_eq!(usage.execution_time_ms, 0);
    }

    #[test]
    fn test_executor_new() {
        let executor = PluginExecutor::new();
        assert!(!executor.is_running("test"));
    }

    #[test]
    fn test_ipc_send_get() {
        let mut ipc = PluginIpc::new();
        let message = PluginMessage {
            from: "plugin1".to_string(),
            to: "plugin2".to_string(),
            method: "test".to_string(),
            params: serde_json::json!({}),
            timestamp: 0,
        };

        ipc.send_message(message.clone()).unwrap();
        let messages = ipc.get_messages("plugin2");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from, "plugin1");
    }
}
