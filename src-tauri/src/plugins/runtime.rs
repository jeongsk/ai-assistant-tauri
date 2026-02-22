//! WASM Runtime for Plugin Execution
//!
//! This module provides WebAssembly runtime using Wasmtime for sandboxed plugin execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

// WASM feature-gated imports
#[cfg(feature = "wasm")]
use wasmtime::{
    Engine, Module, Instance, Store, Linker,
    Config,
};
#[cfg(feature = "wasm")]
use wasmtime_wasi::WasiCtxBuilder;
#[cfg(feature = "wasm")]
use wasmtime_wasi::preview1::{WasiP1Ctx, add_to_linker_sync};

/// WASM runtime configuration
#[derive(Debug, Clone)]
pub struct WasmRuntimeConfig {
    /// Maximum memory in MB (default: 128)
    pub max_memory_mb: u64,
    /// Maximum execution time in milliseconds (default: 5000)
    pub max_execution_time_ms: u64,
    /// Enable fuel metering for execution limiting
    pub enable_fuel: bool,
    /// Initial fuel units (if enabled)
    pub initial_fuel: u64,
}

impl Default for WasmRuntimeConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 128,
            max_execution_time_ms: 5000,
            enable_fuel: true,
            initial_fuel: 1_000_000,
        }
    }
}

/// WebAssembly module loaded from bytes
pub struct WasmModule {
    /// Module hash for validation
    hash: String,

    /// WASM feature-gated module
    #[cfg(feature = "wasm")]
    module: Option<Module>,

    /// Raw bytes for non-wasm builds
    #[cfg(not(feature = "wasm"))]
    bytes: Vec<u8>,
}

impl WasmModule {
    /// Create a new WASM module from bytes
    #[cfg(feature = "wasm")]
    pub fn from_bytes(bytes: Vec<u8>, engine: Option<&Engine>) -> Result<Self, String> {
        // Validate WASM header
        if bytes.len() < 8 {
            return Err("Invalid WASM module: too short".to_string());
        }

        // Check for WASM magic number: \0asm
        if &bytes[0..4] != b"\0asm" {
            return Err("Invalid WASM module: missing magic number".to_string());
        }

        // Simple hash for validation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());

        // Always require engine for wasm feature
        let eng = engine.ok_or_else(|| "Engine required for WASM module compilation".to_string())?;

        let module = Module::from_binary(eng, &bytes)
            .map_err(|e| format!("Failed to compile WASM: {}", e))?;

        Ok(Self { hash, module: Some(module) })
    }

    /// Create a new WASM module from bytes (non-wasm fallback)
    #[cfg(not(feature = "wasm"))]
    pub fn from_bytes(bytes: Vec<u8>, _engine: Option<&()>) -> Result<Self, String> {
        // Validate WASM header
        if bytes.len() < 8 {
            return Err("Invalid WASM module: too short".to_string());
        }

        // Check for WASM magic number: \0asm
        if &bytes[0..4] != b"\0asm" {
            return Err("Invalid WASM module: missing magic number".to_string());
        }

        // Simple hash for validation (using built-in hashing)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash = format!("{:x}", hasher.finish());

        Ok(Self { hash, bytes })
    }

    /// Get module hash
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// Get module bytes
    pub fn bytes(&self) -> &[u8] {
        #[cfg(feature = "wasm")]
        {
            &[]
        }
        #[cfg(not(feature = "wasm"))]
        { &self.bytes }
    }

    /// Get wasmtime Module (only available with wasm feature)
    #[cfg(feature = "wasm")]
    pub fn as_module(&self) -> Option<&Module> {
        self.module.as_ref()
    }
}

/// Running WASM instance
pub struct WasmInstance {
    /// Instance state
    state: InstanceState,
    /// Creation timestamp
    created_at: Instant,

    /// WASM feature-gated fields
    #[cfg(feature = "wasm")]
    store: Option<Store<WasiP1Ctx>>,
    #[cfg(feature = "wasm")]
    instance: Option<Instance>,
}

/// Instance state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceState {
    /// Instance ID
    pub id: String,
    /// Status
    pub status: InstanceStatus,
    /// Memory usage in bytes
    pub memory_used: u64,
    /// Fuel consumed
    pub fuel_consumed: u64,
}

/// Instance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceStatus {
    /// Running normally
    Running,
    /// Paused
    Paused,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed(String),
}

/// WASM runtime for plugin execution
pub struct WasmRuntime {
    config: WasmRuntimeConfig,
    /// Loaded modules
    modules: HashMap<String, WasmModule>,
    /// Active instances
    instances: HashMap<String, WasmInstance>,

    /// WASM feature-gated fields
    #[cfg(feature = "wasm")]
    engine: Option<Engine>,
    #[cfg(feature = "wasm")]
    linker: Option<Linker<WasiP1Ctx>>,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub fn new(config: WasmRuntimeConfig) -> Self {
        #[cfg(feature = "wasm")]
        let mut engine_config = Config::new();
        #[cfg(feature = "wasm")]
        {
            engine_config.wasm_simd(true);
            engine_config.wasm_multi_memory(true);
            // Enable fuel consumption at engine level if fuel is enabled in config
            engine_config.consume_fuel(true);
        }

        #[cfg(feature = "wasm")]
        let engine = Engine::new(&engine_config)
            .map_err(|e| format!("Failed to create WASM engine: {}", e))
            .ok();

        #[cfg(feature = "wasm")]
        let mut linker = Linker::new(engine.as_ref().unwrap());
        #[cfg(feature = "wasm")]
        {
            // Add WASI preview1 to the linker using wasmtime 22+ API
            add_to_linker_sync(&mut linker, |s| s)
                .expect("Failed to add WASI to linker");
        }

        Self {
            config,
            modules: HashMap::new(),
            instances: HashMap::new(),
            #[cfg(feature = "wasm")]
            engine,
            #[cfg(feature = "wasm")]
            linker: Some(linker),
        }
    }

    /// Load a module from bytes
    pub fn load_module(&mut self, bytes: Vec<u8>) -> Result<String, String> {
        #[cfg(feature = "wasm")]
        let module = WasmModule::from_bytes(bytes, self.engine.as_ref())?;

        #[cfg(not(feature = "wasm"))]
        let module = WasmModule::from_bytes(bytes, None)?;

        let module_hash = module.hash().to_string();
        self.modules.insert(module_hash.clone(), module);
        Ok(module_hash)
    }

    /// Instantiate a module with WASI context
    #[cfg(feature = "wasm")]
    pub fn instantiate(
        &mut self,
        module_hash: &str,
        wasi_ctx: Option<WasiP1Ctx>,
    ) -> Result<String, String> {
        // Find module
        let module = self.modules
            .get(module_hash)
            .ok_or_else(|| format!("Module not found: {}", module_hash))?;

        // Get the actual wasmtime Module
        let wasmtime_module = module.as_module()
            .ok_or_else(|| format!("Module {:?} is not a valid WASM module", module_hash))?;

        // Create instance ID
        let instance_id = uuid::Uuid::new_v4().to_string();

        // Get engine and linker references
        let engine = self.engine.as_ref()
            .ok_or_else(|| "WASM engine not initialized".to_string())?;
        let linker = self.linker.as_ref()
            .ok_or_else(|| "WASM linker not initialized".to_string())?;

        // Create WASI context (use provided or create minimal)
        let wasi_ctx = wasi_ctx.unwrap_or_else(|| {
            WasiCtxBuilder::new().build_p1()
        });

        // Create store with WASI context as data
        let mut store = Store::new(engine, wasi_ctx);

        // Configure fuel if enabled
        if self.config.enable_fuel {
            store.set_fuel(self.config.initial_fuel)
                .map_err(|e| format!("Failed to set fuel: {}", e))?;
        }

        // Instantiate the module with the linker
        let instance = linker.instantiate(&mut store, wasmtime_module)
            .map_err(|e| format!("Failed to instantiate WASM module: {}", e))?;

        // Calculate initial memory size
        let memory_size = self.get_instance_memory_size(&instance, &mut store);

        // Create instance state
        let state = InstanceState {
            id: instance_id.clone(),
            status: InstanceStatus::Running,
            memory_used: memory_size,
            fuel_consumed: 0,
        };

        // Store the instance with actual Store and Instance
        self.instances.insert(instance_id.clone(), WasmInstance {
            state,
            created_at: Instant::now(),
            #[cfg(feature = "wasm")]
            store: Some(store),
            #[cfg(feature = "wasm")]
            instance: Some(instance),
        });

        Ok(instance_id)
    }

    /// Instantiate a module with WASI context (non-wasm fallback)
    #[cfg(not(feature = "wasm"))]
    pub fn instantiate(
        &mut self,
        module_hash: &str,
        _wasi_ctx: Option<()>,
    ) -> Result<String, String> {
        // Find module
        let _module = self.modules
            .get(module_hash)
            .ok_or_else(|| format!("Module not found: {}", module_hash))?;

        // Create instance ID
        let instance_id = uuid::Uuid::new_v4().to_string();

        // Fallback for non-wasm builds
        let state = InstanceState {
            id: instance_id.clone(),
            status: InstanceStatus::Running,
            memory_used: 0,
            fuel_consumed: 0,
        };

        self.instances.insert(instance_id.clone(), WasmInstance {
            state,
            created_at: Instant::now(),
            #[cfg(feature = "wasm")]
            store: None,
            #[cfg(feature = "wasm")]
            instance: None,
        });

        Ok(instance_id)
    }

    /// Call a function on an instance
    pub fn call_function(
        &mut self,
        instance_id: &str,
        function_name: &str,
        args: Vec<serde_json::Value>,
    ) -> Result<WasmExecutionResult, String> {
        let start = Instant::now();

        // Find instance
        let instance_entry = self.instances
            .get_mut(instance_id)
            .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

        #[cfg(feature = "wasm")]
        {
            // Get mutable references to store and instance
            let store = instance_entry.store.as_mut()
                .ok_or_else(|| "Store not initialized".to_string())?;
            let instance = instance_entry.instance.as_ref()
                .ok_or_else(|| "Instance not initialized".to_string())?;

            // Check execution time limit before calling
            if start.elapsed().as_millis() > self.config.max_execution_time_ms as u128 {
                instance_entry.state.status = InstanceStatus::Failed(
                    "Execution time limit exceeded".to_string()
                );
                return Ok(WasmExecutionResult {
                    success: false,
                    result: None,
                    error: Some("Execution time limit exceeded".to_string()),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                    fuel_consumed: 0,
                });
            }

            // Try calling with different argument patterns
            let result_value: serde_json::Value = match args.len() {
                2 => {
                    // Try (i32, i32) -> i32
                    if let Ok(f) = instance.get_typed_func::<(i32, i32), i32>(&mut *store, function_name) {
                        let a = args[0].as_i64().unwrap_or(0) as i32;
                        let b = args[1].as_i64().unwrap_or(0) as i32;
                        serde_json::json!(f.call(&mut *store, (a, b)).map_err(|e| e.to_string())?)
                    } else {
                        // Function not found with this signature, try to fail gracefully
                        return Ok(WasmExecutionResult {
                            success: false,
                            result: None,
                            error: Some(format!("Function '{}' not found or has wrong signature", function_name)),
                            execution_time_ms: start.elapsed().as_millis() as u64,
                            fuel_consumed: 0,
                        });
                    }
                }
                0 => {
                    // Try () -> i32
                    if let Ok(f) = instance.get_typed_func::<(), i32>(&mut *store, function_name) {
                        serde_json::json!(f.call(&mut *store, ()).map_err(|e| e.to_string())?)
                    } else if let Ok(f) = instance.get_typed_func::<(), ()>(&mut *store, function_name) {
                        f.call(&mut *store, ()).map_err(|e| e.to_string())?;
                        serde_json::json!({"status": "ok"})
                    } else {
                        // Function not found with any signature
                        return Ok(WasmExecutionResult {
                            success: false,
                            result: None,
                            error: Some(format!("Function '{}' not found or has wrong signature", function_name)),
                            execution_time_ms: start.elapsed().as_millis() as u64,
                            fuel_consumed: 0,
                        });
                    }
                }
                _ => {
                    return Ok(WasmExecutionResult {
                        success: false,
                        result: None,
                        error: Some(format!("Unsupported argument count: {}", args.len())),
                        execution_time_ms: start.elapsed().as_millis() as u64,
                        fuel_consumed: 0,
                    });
                }
            };

            // Get fuel consumed (fuel_consumed() method not available, use 0 for now)
            let fuel_consumed = if self.config.enable_fuel {
                0  // TODO: Implement fuel tracking when API is available
            } else {
                0
            };

            // Calculate memory size (inline to avoid borrow checker issues)
            let memory_used = match instance.get_memory(&mut *store, "memory") {
                Some(memory) => memory.size(&mut *store) * 65536,
                None => 0,
            };

            // Update instance state
            instance_entry.state.fuel_consumed = fuel_consumed;
            instance_entry.state.memory_used = memory_used;

            Ok(WasmExecutionResult {
                success: true,
                result: Some(result_value),
                error: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
                fuel_consumed,
            })
        }

        #[cfg(not(feature = "wasm"))]
        {
            // Check execution time limit
            let elapsed = start.elapsed();
            if elapsed.as_millis() > self.config.max_execution_time_ms as u128 {
                instance_entry.state.status = InstanceStatus::Failed(
                    "Execution time limit exceeded".to_string()
                );
                return Ok(WasmExecutionResult {
                    success: false,
                    result: None,
                    error: Some("Execution time limit exceeded".to_string()),
                    execution_time_ms: elapsed.as_millis() as u64,
                    fuel_consumed: 0,
                });
            }

            // Simplified function call for non-wasm builds
            let result: Result<serde_json::Value, String> = match function_name {
                "init" => Ok(serde_json::json!({"status": "initialized"})),
                "shutdown" => Ok(serde_json::json!({"status": "shutdown"})),
                _ => Ok(serde_json::json!({"result": "ok"})),
            };

            let elapsed = start.elapsed();
            let fuel_used = if self.config.enable_fuel {
                elapsed.as_millis() as u64 * 1000
            } else {
                0
            };

            match result {
                Ok(value) => {
                    instance_entry.state.fuel_consumed += fuel_used;
                    Ok(WasmExecutionResult {
                        success: true,
                        result: Some(value),
                        error: None,
                        execution_time_ms: elapsed.as_millis() as u64,
                        fuel_consumed: fuel_used,
                    })
                }
                Err(e) => {
                    instance_entry.state.status = InstanceStatus::Failed(e.clone());
                    Ok(WasmExecutionResult {
                        success: false,
                        result: None,
                        error: Some(e),
                        execution_time_ms: elapsed.as_millis() as u64,
                        fuel_consumed: fuel_used,
                    })
                }
            }
        }
    }

    /// Get memory size from instance
    #[cfg(feature = "wasm")]
    fn get_memory_size(_instance: &Instance, _store: &Store<WasiP1Ctx>) -> u64 {
        // Simplified - always return 0 for compatibility
        0
    }

    /// Get memory size from instance (new helper)
    #[cfg(feature = "wasm")]
    fn get_instance_memory_size(&self, instance: &Instance, store: &mut Store<WasiP1Ctx>) -> u64 {
        match instance.get_memory(&mut *store, "memory") {
            Some(memory) => memory.size(&mut *store) * 65536, // Wasm pages to bytes
            None => 0,
        }
    }

    /// Get instance state
    pub fn get_instance_state(&self, instance_id: &str) -> Option<InstanceState> {
        self.instances
            .get(instance_id)
            .map(|i| i.state.clone())
    }

    /// Remove an instance
    pub fn remove_instance(&mut self, instance_id: &str) -> Result<(), String> {
        self.instances
            .remove(instance_id)
            .map(|_| ())
            .ok_or_else(|| format!("Instance not found: {}", instance_id))
    }

    /// List all instances
    pub fn list_instances(&self) -> Vec<InstanceState> {
        self.instances.values().map(|i| i.state.clone()).collect()
    }

    /// Check if wasm feature is enabled
    pub fn is_wasm_enabled() -> bool {
        cfg!(feature = "wasm")
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new(WasmRuntimeConfig::default())
    }
}

/// WASM execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmExecutionResult {
    /// Success flag
    pub success: bool,
    /// Return value (if successful)
    pub result: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Fuel consumed (if fuel metering enabled)
    pub fuel_consumed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_module_creation() {
        // Minimal valid WASM module (just header)
        let bytes = vec![
            0x00, 0x61, 0x73, 0x6D, // \0asm
            0x01, 0x00, 0x00, 0x00, // version 1
        ];

        #[cfg(feature = "wasm")]
        let runtime = WasmRuntime::new(WasmRuntimeConfig::default());

        #[cfg(feature = "wasm")]
        let result = WasmModule::from_bytes(bytes, runtime.engine.as_ref());

        #[cfg(not(feature = "wasm"))]
        let result = WasmModule::from_bytes(bytes, None);

        assert!(result.is_ok());

        let module = result.unwrap();
        assert!(!module.hash().is_empty());
    }

    #[test]
    fn test_runtime_creation() {
        let runtime = WasmRuntime::new(WasmRuntimeConfig::default());
        assert_eq!(runtime.modules.len(), 0);
        assert_eq!(runtime.instances.len(), 0);
    }

    #[test]
    fn test_runtime_default_config() {
        let config = WasmRuntimeConfig::default();
        assert_eq!(config.max_memory_mb, 128);
        assert_eq!(config.max_execution_time_ms, 5000);
        assert!(config.enable_fuel);
    }
}
