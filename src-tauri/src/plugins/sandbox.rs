// Plugin Sandbox - Sandboxed execution environment

use crate::plugins::{PluginContext, PluginPermission};
use std::collections::HashMap;

/// Sandbox instance for isolated plugin execution
pub struct PluginSandbox {
    context: PluginContext,
    resource_tracker: ResourceTracker,
}

/// Resource usage tracker
pub struct ResourceTracker {
    memory_used: u64,
    cpu_time_ms: u64,
    files_accessed: Vec<String>,
    network_calls: Vec<String>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            memory_used: 0,
            cpu_time_ms: 0,
            files_accessed: Vec::new(),
            network_calls: Vec::new(),
        }
    }

    pub fn record_memory(&mut self, bytes: u64) {
        self.memory_used = bytes;
    }

    pub fn record_cpu_time(&mut self, ms: u64) {
        self.cpu_time_ms = ms;
    }

    pub fn record_file_access(&mut self, path: &str) {
        self.files_accessed.push(path.to_string());
    }

    pub fn record_network_call(&mut self, host: &str) {
        self.network_calls.push(host.to_string());
    }

    pub fn get_memory_used(&self) -> u64 {
        self.memory_used
    }

    pub fn get_cpu_time_ms(&self) -> u64 {
        self.cpu_time_ms
    }
}

impl PluginSandbox {
    pub fn new(context: PluginContext) -> Self {
        Self {
            context,
            resource_tracker: ResourceTracker::new(),
        }
    }

    /// Check if action is permitted
    pub fn check_permission(&self, action: &SandboxAction) -> Result<(), String> {
        match action {
            SandboxAction::ReadFile(path) => {
                self.check_file_permission(path, "read")?;
            }
            SandboxAction::WriteFile(path) => {
                self.check_file_permission(path, "readwrite")?;
            }
            SandboxAction::NetworkRequest(host) => {
                self.check_network_permission(host)?;
            }
            SandboxAction::DatabaseQuery(table) => {
                self.check_database_permission(table)?;
            }
            SandboxAction::SystemCall(capability) => {
                self.check_system_permission(capability)?;
            }
        }
        Ok(())
    }

    fn check_file_permission(&self, path: &str, access: &str) -> Result<(), String> {
        for perm in &self.context.permissions {
            if let PluginPermission::FileSystem { paths, access: perm_access } = perm {
                for allowed_path in paths {
                    if path.starts_with(allowed_path) {
                        if access == "read" || perm_access == "readwrite" {
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(format!("File access denied: {}", path))
    }

    fn check_network_permission(&self, host: &str) -> Result<(), String> {
        for perm in &self.context.permissions {
            if let PluginPermission::Network { hosts } = perm {
                for allowed_host in hosts {
                    if host == allowed_host || host.ends_with(&format!(".{}", allowed_host)) {
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Network access denied: {}", host))
    }

    fn check_database_permission(&self, table: &str) -> Result<(), String> {
        for perm in &self.context.permissions {
            if let PluginPermission::Database { tables } = perm {
                if tables.contains(&table.to_string()) || tables.contains(&"*".to_string()) {
                    return Ok(());
                }
            }
        }
        Err(format!("Database access denied: {}", table))
    }

    fn check_system_permission(&self, capability: &str) -> Result<(), String> {
        for perm in &self.context.permissions {
            if let PluginPermission::System { capabilities } = perm {
                if capabilities.contains(&capability.to_string()) {
                    return Ok(());
                }
            }
        }
        Err(format!("System capability denied: {}", capability))
    }

    /// Check resource limits
    pub fn check_resource_limits(&self) -> Result<(), String> {
        let limits = &self.context.resource_limits;

        if self.resource_tracker.get_memory_used() > (limits.max_memory_mb as u64 * 1024 * 1024) {
            return Err("Memory limit exceeded".to_string());
        }

        if self.resource_tracker.get_cpu_time_ms() > limits.max_execution_time_ms as u64 {
            return Err("Execution time limit exceeded".to_string());
        }

        Ok(())
    }

    /// Get resource tracker
    pub fn get_resource_tracker(&self) -> &ResourceTracker {
        &self.resource_tracker
    }

    /// Get mutable resource tracker
    pub fn get_resource_tracker_mut(&mut self) -> &mut ResourceTracker {
        &mut self.resource_tracker
    }
}

/// Actions that can be performed in sandbox
#[derive(Debug, Clone)]
pub enum SandboxAction {
    ReadFile(String),
    WriteFile(String),
    NetworkRequest(String),
    DatabaseQuery(String),
    SystemCall(String),
}

/// Sandbox manager for multiple plugins
pub struct SandboxManager {
    sandboxes: HashMap<String, PluginSandbox>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self {
            sandboxes: HashMap::new(),
        }
    }

    pub fn create_sandbox(&mut self, plugin_id: &str, context: PluginContext) {
        let sandbox = PluginSandbox::new(context);
        self.sandboxes.insert(plugin_id.to_string(), sandbox);
    }

    pub fn get_sandbox(&self, plugin_id: &str) -> Option<&PluginSandbox> {
        self.sandboxes.get(plugin_id)
    }

    pub fn get_sandbox_mut(&mut self, plugin_id: &str) -> Option<&mut PluginSandbox> {
        self.sandboxes.get_mut(plugin_id)
    }

    pub fn destroy_sandbox(&mut self, plugin_id: &str) {
        self.sandboxes.remove(plugin_id);
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}
