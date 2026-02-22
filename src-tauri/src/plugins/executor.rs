//! Plugin Executor - WASM/WASI based plugin execution engine
//!
//! This module provides the actual execution engine for plugins using Wasmtime.

use crate::plugins::{
    api::{handle_request, PluginRequest},
    sandbox::{PluginSandbox, SandboxManager},
    runtime::{WasmRuntime, WasmRuntimeConfig},
    wasi_host::WasiHost,
    monitor::{ResourceMonitor, MetricUpdate},
    PluginContext, PluginManifest, PluginPermission, ResourceLimits,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    #[cfg(feature = "wasm")]
    pub fuel_consumed: u64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 0.0,
            cpu_percent: 0.0,
            execution_time_ms: 0,
            syscall_count: 0,
            #[cfg(feature = "wasm")]
            fuel_consumed: 0,
        }
    }
}

/// Running plugin instance
pub struct RunningPlugin {
    pub id: String,
    pub manifest: PluginManifest,
    pub started_at: Instant,
    pub state: PluginInstanceState,
    /// WASM instance ID (when wasm feature is enabled)
    #[cfg(feature = "wasm")]
    pub wasm_instance_id: Option<String>,
    /// Plugin working directory
    pub work_dir: PathBuf,
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
    /// WASM runtime
    wasm_runtime: Arc<Mutex<WasmRuntime>>,
    /// WASI host
    wasi_host: Arc<Mutex<WasiHost>>,
    /// Resource monitor
    monitor: Arc<Mutex<ResourceMonitor>>,
    /// Plugins directory
    plugins_dir: PathBuf,
}

impl PluginExecutor {
    /// Create a new plugin executor
    pub fn new() -> Self {
        Self {
            running_plugins: Arc::new(Mutex::new(HashMap::new())),
            sandbox_manager: Arc::new(Mutex::new(SandboxManager::new())),
            ipc: Arc::new(Mutex::new(PluginIpc::new())),
            wasm_runtime: Arc::new(Mutex::new(WasmRuntime::new(WasmRuntimeConfig::default()))),
            wasi_host: Arc::new(Mutex::new(WasiHost::new())),
            monitor: Arc::new(Mutex::new(ResourceMonitor::new())),
            plugins_dir: PathBuf::from("plugins"),
        }
    }

    /// Create a new plugin executor with custom plugins directory
    pub fn with_plugins_dir(plugins_dir: PathBuf) -> Self {
        Self {
            running_plugins: Arc::new(Mutex::new(HashMap::new())),
            sandbox_manager: Arc::new(Mutex::new(SandboxManager::new())),
            ipc: Arc::new(Mutex::new(PluginIpc::new())),
            wasm_runtime: Arc::new(Mutex::new(WasmRuntime::new(WasmRuntimeConfig::default()))),
            wasi_host: Arc::new(Mutex::new(WasiHost::new())),
            monitor: Arc::new(Mutex::new(ResourceMonitor::new())),
            plugins_dir,
        }
    }

    /// Start a plugin
    pub async fn start_plugin(&mut self, manifest: PluginManifest) -> Result<(), String> {
        let plugin_id = manifest.id.clone();

        // Check if already running
        if self.is_running(&plugin_id) {
            return Err(format!("Plugin {} is already running", plugin_id));
        }

        // Create working directory
        let work_dir = self.plugins_dir.join(&plugin_id);
        std::fs::create_dir_all(&work_dir)
            .map_err(|e| format!("Failed to create work directory: {}", e))?;

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

        // Load WASM module if wasm feature is enabled
        #[cfg(feature = "wasm")]
        let wasm_instance_id = {
            let wasm_path = PathBuf::from(&manifest.main);
            if wasm_path.exists() {
                // Read WASM bytes
                let wasm_bytes = std::fs::read(&wasm_path)
                    .map_err(|e| format!("Failed to read WASM file: {}", e))?;

                // Load module
                let mut runtime = self.wasm_runtime.lock().unwrap();
                let module_hash = runtime.load_module(wasm_bytes)
                    .map_err(|e| format!("Failed to load WASM module: {}", e))?;

                // Create WASI context with working directory
                let wasi_ctx = create_wasi_context_with_dir(&plugin_id, work_dir.to_str().unwrap())?;

                // Instantiate
                let instance_id = runtime.instantiate(&module_hash, Some(wasi_ctx))
                    .map_err(|e| format!("Failed to instantiate WASM: {}", e))?;

                // Start monitoring
                let mut monitor = self.monitor.lock().unwrap();
                monitor.start_monitoring(instance_id.clone());

                Some(instance_id)
            } else {
                None
            }
        };

        #[cfg(not(feature = "wasm"))]
        let _wasm_instance_id: Option<String> = None;

        // Create running instance
        let running = RunningPlugin {
            id: plugin_id.clone(),
            manifest: manifest.clone(),
            started_at: Instant::now(),
            state: PluginInstanceState::Running,
            #[cfg(feature = "wasm")]
            wasm_instance_id,
            work_dir,
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
        let _plugin = plugins.remove(id)
            .ok_or_else(|| format!("Plugin {} not found", id))?;

        // Stop monitoring
        #[cfg(feature = "wasm")]
        if let Some(instance_id) = plugin.wasm_instance_id {
            let mut monitor = self.monitor.lock().unwrap();
            monitor.stop_monitoring(&instance_id);

            // Remove WASM instance
            let mut runtime = self.wasm_runtime.lock().unwrap();
            let _ = runtime.remove_instance(&instance_id);
        }

        // Destroy sandbox
        let mut sandbox_manager = self.sandbox_manager.lock().unwrap();
        sandbox_manager.destroy_sandbox(id);

        // Remove WASI context
        let mut wasi_host = self.wasi_host.lock().unwrap();
        wasi_host.remove_context(id);

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
            plugins.get(plugin_id).is_some_and(|p| {
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
            #[cfg(feature = "wasm")]
            "wasm.call" => self.execute_wasm_call(plugin_id, params).await,
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

        // Update metrics
        let mut monitor = self.monitor.lock().unwrap();
        monitor.update_metrics(plugin_id, MetricUpdate::Syscall);
        drop(monitor);

        // Handle request
        let response = handle_request(request);

        ExecutionResult {
            success: response.error.is_none(),
            result: response.result,
            error: response.error,
            execution_time_ms: start.elapsed().as_millis() as u64,
            resource_usage: self.get_resource_usage_internal(plugin_id),
        }
    }

    /// Execute a WASM function call
    #[cfg(feature = "wasm")]
    async fn execute_wasm_call(&mut self, plugin_id: &str, params: Value) -> ExecutionResult {
        let start = Instant::now();

        // Get WASM instance ID
        let instance_id = {
            let plugins = self.running_plugins.lock().unwrap();
            plugins.get(plugin_id)
                .and_then(|p| p.wasm_instance_id.clone())
                .ok_or_else(|| format!("No WASM instance for plugin {}", plugin_id))
        };

        let instance_id = match instance_id {
            Ok(id) => id,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    result: None,
                    error: Some(e),
                    execution_time_ms: 0,
                    resource_usage: ResourceUsage::default(),
                };
            }
        };

        // Parse function name and args
        let function_name = params.get("function")
            .and_then(|v| v.as_str())
            .unwrap_or("call");
        let args = params.get("args")
            .and_then(|v| v.as_array())
            .map(|v| v.clone())
            .unwrap_or_default();

        // Call WASM function
        let mut runtime = self.wasm_runtime.lock().unwrap();
        let wasm_result = runtime.call_function(&instance_id, function_name,
            args.into_iter().collect());

        let wasm_result = match wasm_result {
            Ok(r) => r,
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    result: None,
                    error: Some(e),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    resource_usage: ResourceUsage::default(),
                };
            }
        };

        // Update resource monitor
        let mut monitor = self.monitor.lock().unwrap();
        monitor.update_from_wasm(&instance_id, wasm_result.fuel_consumed,
            wasm_result.execution_time_ms as u64);

        ExecutionResult {
            success: wasm_result.success,
            result: wasm_result.result,
            error: wasm_result.error,
            execution_time_ms: wasm_result.execution_time_ms,
            resource_usage: ResourceUsage {
                memory_mb: 0.0,
                cpu_percent: 0.0,
                execution_time_ms: wasm_result.execution_time_ms,
                syscall_count: 1,
                fuel_consumed: wasm_result.fuel_consumed,
            },
        }
    }

    /// Check if a method is allowed
    fn check_method_permission(&self, method: &str, sandbox: &PluginSandbox) -> Result<(), String> {
        // Parse method to determine permission type
        if method.starts_with("fs.") {
            // File system methods need file permission
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

    fn get_resource_usage_internal(&self, plugin_id: &str) -> ResourceUsage {
        let monitor = self.monitor.lock().unwrap();
        if let Some(metrics) = monitor.get_metrics(plugin_id) {
            ResourceUsage {
                memory_mb: metrics.memory_bytes as f64 / (1024.0 * 1024.0),
                cpu_percent: 0.0,
                execution_time_ms: metrics.cpu_time_ms,
                syscall_count: metrics.syscall_count,
                #[cfg(feature = "wasm")]
                fuel_consumed: metrics.fuel_consumed,
            }
        } else {
            ResourceUsage::default()
        }
    }

    /// Check if a plugin is running
    pub fn is_running(&self, id: &str) -> bool {
        let plugins = self.running_plugins.lock().unwrap();
        plugins.get(id).is_some_and(|p| p.state == PluginInstanceState::Running)
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
        if plugins.get(id).is_some() {
            Some(self.get_resource_usage_internal(id))
        } else {
            None
        }
    }

    /// Get IPC manager
    pub fn get_ipc(&self) -> Arc<Mutex<PluginIpc>> {
        self.ipc.clone()
    }

    /// Get resource monitor
    pub fn get_monitor(&self) -> Arc<Mutex<ResourceMonitor>> {
        self.monitor.clone()
    }

    /// Get WASM runtime
    pub fn get_wasm_runtime(&self) -> Arc<Mutex<WasmRuntime>> {
        self.wasm_runtime.clone()
    }

    /// Get WASI host
    pub fn get_wasi_host(&self) -> Arc<Mutex<WasiHost>> {
        self.wasi_host.clone()
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
        let to = message.to.clone();
        self.message_queue
            .entry(to)
            .or_default()
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

    #[test]
    fn test_executor_with_plugins_dir() {
        let executor = PluginExecutor::with_plugins_dir(PathBuf::from("/tmp/plugins"));
        // Verify executor is created successfully
        let _ = executor;
    }

    #[test]
    fn test_plugin_instance_state() {
        let state = PluginInstanceState::Starting;
        assert_eq!(state, PluginInstanceState::Starting);

        let error_state = PluginInstanceState::Error("test error".to_string());
        assert!(matches!(error_state, PluginInstanceState::Error(_)));
    }
}
