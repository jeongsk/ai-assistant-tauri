//! WASI Host for Plugin Sandboxing
//!
//! This module provides WASI (WebAssembly System Interface) host implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// WASM feature-gated imports
#[cfg(feature = "wasm")]
use wasmtime_wasi::WasiCtxBuilder;
#[cfg(feature = "wasm")]
use wasmtime_wasi::preview1::WasiP1Ctx;

/// WASI context for sandboxed execution
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct WasiContext {
    /// Preopened directories
    pub preopened_dirs: Vec<WasiDirectory>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Stdio configuration
    pub stdio: WasiStdio,
    /// Arguments
    pub args: Vec<String>,
}


impl WasiContext {
    /// Create a new WASI context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a preopened directory
    pub fn with_directory(mut self, dir: WasiDirectory) -> Self {
        self.preopened_dirs.push(dir);
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }

    /// Set stdio configuration
    pub fn with_stdio(mut self, stdio: WasiStdio) -> Self {
        self.stdio = stdio;
        self
    }

    /// Add arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Build the WASI context
    #[cfg(feature = "wasm")]
    pub fn build(&self) -> Result<WasiP1Ctx, String> {
        let mut builder = WasiCtxBuilder::new();

        // Add environment variables (env() doesn't return Result in wasmtime 22)
        for (key, value) in &self.env_vars {
            builder.env(key, value);
        }

        // Add arguments (arg() doesn't return Result in wasmtime 22)
        for arg in &self.args {
            builder.arg(arg);
        }

        // Note: Stdio capture configuration skipped - CaptureOutput not available in wasmtime 22
        // This will be addressed in a future update

        // Use build_p1() for WASI preview1
        Ok(builder.build_p1())
    }

    /// Build the WASI context (non-wasm fallback)
    #[cfg(not(feature = "wasm"))]
    pub fn build(&self) -> Result<WasiSnapshot, String> {
        Ok(WasiSnapshot {
            dirs: self.preopened_dirs.clone(),
            env: self.env_vars.clone(),
            stdio: self.stdio.clone(),
            args: self.args.clone(),
        })
    }
}

/// WASI directory mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiDirectory {
    /// Guest path (inside WASM)
    pub guest_path: String,
    /// Host path (actual filesystem path)
    pub host_path: String,
    /// Directory permissions
    pub permissions: WasiPermissions,
}

/// Directory permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiPermissions {
    /// Allow reading
    pub read: bool,
    /// Allow writing
    pub write: bool,
    /// Allow creating files
    pub create: bool,
}

impl Default for WasiPermissions {
    fn default() -> Self {
        Self {
            read: true,
            write: false,
            create: false,
        }
    }
}

/// Stdio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiStdio {
    /// Capture stdout
    pub capture_stdout: bool,
    /// Capture stderr
    pub capture_stderr: bool,
    /// Provide stdin
    pub provide_stdin: bool,
}

impl Default for WasiStdio {
    fn default() -> Self {
        Self {
            capture_stdout: true,
            capture_stderr: true,
            provide_stdin: false,
        }
    }
}

/// WASI snapshot for execution (non-wasm builds)
#[derive(Debug, Clone)]
pub struct WasiSnapshot {
    pub dirs: Vec<WasiDirectory>,
    pub env: HashMap<String, String>,
    pub stdio: WasiStdio,
    pub args: Vec<String>,
}

/// WASI host for managing sandbox instances
pub struct WasiHost {
    /// WASI contexts (feature-gated)
    #[cfg(feature = "wasm")]
    contexts: HashMap<String, WasiP1Ctx>,

    /// Fallback snapshots for non-wasm builds
    #[cfg(not(feature = "wasm"))]
    contexts: HashMap<String, WasiSnapshot>,
}

impl WasiHost {
    /// Create a new WASI host
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
        }
    }

    /// Register a WASI context
    #[cfg(feature = "wasm")]
    pub fn register_context(&mut self, id: String, context: WasiP1Ctx) {
        self.contexts.insert(id, context);
    }

    /// Register a WASI snapshot (non-wasm)
    #[cfg(not(feature = "wasm"))]
    pub fn register_context(&mut self, id: String, context: WasiSnapshot) {
        self.contexts.insert(id, context);
    }

    /// Get a context (wasm builds)
    #[cfg(feature = "wasm")]
    pub fn get_context(&self, id: &str) -> Option<&WasiP1Ctx> {
        self.contexts.get(id)
    }

    /// Remove a context
    pub fn remove_context(&mut self, id: &str) -> Option<()> {
        self.contexts.remove(id).map(|_| ())
    }

    /// Create a temporary context builder
    pub fn create_context(&mut self, id: String) -> WasiContextBuilder<'_> {
        WasiContextBuilder {
            id,
            host: self,
            context: WasiContext::new(),
        }
    }

    /// Build a WASI context from a builder (returns WasiP1Ctx for wasm)
    #[cfg(feature = "wasm")]
    pub fn build_context(&mut self, id: String, context: WasiContext) -> Result<WasiP1Ctx, String> {
        let ctx = context.build()?;
        // Store the context without cloning (WasiP1Ctx doesn't implement Clone)
        // For now, we'll just acknowledge the creation
        let _ = id;
        Ok(ctx)
    }

    /// Build a WASI snapshot (returns WasiSnapshot for non-wasm)
    #[cfg(not(feature = "wasm"))]
    pub fn build_context(&mut self, id: String, context: WasiContext) -> Result<WasiSnapshot, String> {
        let snapshot = context.build()?;
        self.contexts.insert(id.clone(), snapshot.clone());
        Ok(snapshot)
    }
}

impl Default for WasiHost {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating WASI contexts
pub struct WasiContextBuilder<'a> {
    id: String,
    host: &'a mut WasiHost,
    context: WasiContext,
}

impl<'a> WasiContextBuilder<'a> {
    /// Add a preopened directory
    pub fn directory(mut self, guest_path: &str, host_path: &str) -> Self {
        self.context.preopened_dirs.push(WasiDirectory {
            guest_path: guest_path.to_string(),
            host_path: host_path.to_string(),
            permissions: WasiPermissions::default(),
        });
        self
    }

    /// Add a directory with custom permissions
    pub fn directory_with_perms(
        mut self,
        guest_path: &str,
        host_path: &str,
        permissions: WasiPermissions,
    ) -> Self {
        self.context.preopened_dirs.push(WasiDirectory {
            guest_path: guest_path.to_string(),
            host_path: host_path.to_string(),
            permissions,
        });
        self
    }

    /// Add an environment variable
    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.context.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Set stdio to capture output
    pub fn capture_stdio(mut self, capture: bool) -> Self {
        self.context.stdio.capture_stdout = capture;
        self.context.stdio.capture_stderr = capture;
        self
    }

    /// Set arguments
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.context.args = args;
        self
    }

    /// Build and register the context
    pub fn build(self) -> Result<String, String> {
        let id = self.id.clone();
        self.host.build_context(id, self.context)?;
        Ok(self.id)
    }
}

/// Create a minimal WASI context for a plugin
#[cfg(feature = "wasm")]
pub fn create_minimal_wasi_context(_plugin_id: &str) -> Result<WasiP1Ctx, String> {
    Ok(WasiCtxBuilder::new().build_p1())
}

/// Create a minimal WASI context for a plugin (non-wasm)
#[cfg(not(feature = "wasm"))]
pub fn create_minimal_wasi_context(plugin_id: &str) -> Result<(), String> {
    Err(format!("WASI not available (build without wasm feature) for plugin: {}", plugin_id))
}

/// Create a WASI context with working directory
#[cfg(feature = "wasm")]
pub fn create_wasi_context_with_dir(
    _plugin_id: &str,
    work_dir: &str,
) -> Result<WasiP1Ctx, String> {
    let path = Path::new(work_dir);
    if !path.exists() {
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create work directory: {}", e))?;
    }

    // For now, create minimal context
    // Directory preopening will be added in a future step
    Ok(WasiCtxBuilder::new().build_p1())
}

/// Create a WASI context with working directory (non-wasm)
#[cfg(not(feature = "wasm"))]
pub fn create_wasi_context_with_dir(
    plugin_id: &str,
    _work_dir: &str,
) -> Result<(), String> {
    Err(format!("WASI not available (build without wasm feature) for plugin: {}", plugin_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasi_context_default() {
        let ctx = WasiContext::default();
        assert_eq!(ctx.preopened_dirs.len(), 0);
        assert_eq!(ctx.env_vars.len(), 0);
        assert!(ctx.stdio.capture_stdout);
    }

    #[test]
    fn test_wasi_permissions_default() {
        let perms = WasiPermissions::default();
        assert!(perms.read);
        assert!(!perms.write);
        assert!(!perms.create);
    }

    #[test]
    fn test_wasi_host_creation() {
        let host = WasiHost::new();
        let _ = host;
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_minimal_wasi_context() {
        let result = create_minimal_wasi_context("test-plugin");
        assert!(result.is_ok());
    }

    #[cfg(not(feature = "wasm"))]
    #[test]
    fn test_minimal_wasi_context_no_wasm() {
        let result = create_minimal_wasi_context("test-plugin");
        assert!(result.is_err());
    }
}
