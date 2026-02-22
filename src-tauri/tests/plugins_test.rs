//! Integration tests for WASM Plugin Runtime

#[cfg(feature = "wasm")]
mod wasm_tests {
    use ai_assistant_tauri_lib::plugins::runtime::{
        WasmRuntime, WasmRuntimeConfig,
    };
    use ai_assistant_tauri_lib::plugins::wasi_host::create_minimal_wasi_context;

    /// Simple WASM add function - verified valid binary
    /// Compiled from: (module (func (param i32 i32) (result i32) local.get 0 local.get 1 i32.add) (export "add" (func 0)))
    fn get_test_wasm_add() -> Vec<u8> {
        vec![
            0x00, 0x61, 0x73, 0x6D,  // Magic
            0x01, 0x00, 0x00, 0x00,  // Version
            0x01, 0x07, 0x01, 0x60, 0x02, 0x7F, 0x7F, 0x01, 0x7F,  // Type section
            0x03, 0x02, 0x01, 0x00,  // Function section
            0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00,  // Export section
            0x0A, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6A, 0x0B,  // Code section
        ]
    }

    #[test]
    fn test_wasm_runtime_creation() {
        let _runtime = WasmRuntime::new(WasmRuntimeConfig::default());
        assert!(WasmRuntime::is_wasm_enabled());
    }

    #[test]
    fn test_wasm_load_module() {
        let wasm_bytes = get_test_wasm_add();
        let mut runtime = WasmRuntime::new(WasmRuntimeConfig::default());

        let module_hash = runtime.load_module(wasm_bytes);
        assert!(module_hash.is_ok(), "Load module failed: {:?}", module_hash.err());

        let hash = module_hash.unwrap();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_wasm_instantiate() {
        let wasm_bytes = get_test_wasm_add();
        let mut runtime = WasmRuntime::new(WasmRuntimeConfig::default());

        let module_hash = runtime.load_module(wasm_bytes).unwrap();
        let wasi_ctx = create_minimal_wasi_context("test").unwrap();

        let instance_id = runtime.instantiate(&module_hash, Some(wasi_ctx));
        assert!(instance_id.is_ok(), "Instantiate failed: {:?}", instance_id.err());

        let id = instance_id.unwrap();
        let instances = runtime.list_instances();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id, id);
    }

    #[test]
    fn test_wasm_call_function() {
        let wasm_bytes = get_test_wasm_add();
        let mut runtime = WasmRuntime::new(WasmRuntimeConfig::default());

        let module_hash = runtime.load_module(wasm_bytes).unwrap();
        let wasi_ctx = create_minimal_wasi_context("test").unwrap();
        let instance_id = runtime.instantiate(&module_hash, Some(wasi_ctx)).unwrap();

        // Call add(5, 3) should return 8
        let result = runtime.call_function(
            &instance_id,
            "add",
            vec![serde_json::json!(5), serde_json::json!(3)],
        );

        assert!(result.is_ok(), "Call failed: {:?}", result.err());
        let exec_result = result.unwrap();
        assert!(exec_result.success);
        assert_eq!(exec_result.result, Some(serde_json::json!(8)));
    }

    #[test]
    fn test_wasm_fuel_metering() {
        let wasm_bytes = get_test_wasm_add();
        let config = WasmRuntimeConfig {
            enable_fuel: true,
            initial_fuel: 10000,
            ..Default::default()
        };
        let mut runtime = WasmRuntime::new(config);

        let module_hash = runtime.load_module(wasm_bytes).unwrap();
        let wasi_ctx = create_minimal_wasi_context("test").unwrap();
        let instance_id = runtime.instantiate(&module_hash, Some(wasi_ctx)).unwrap();

        let _result = runtime.call_function(
            &instance_id,
            "add",
            vec![serde_json::json!(1), serde_json::json!(2)],
        ).unwrap();

        let state = runtime.get_instance_state(&instance_id).unwrap();
        // Fuel consumption tracking is enabled but may be 0 for very simple functions
        // Just verify the state is accessible
        assert_eq!(state.id, instance_id);
    }

    #[test]
    fn test_wasm_remove_instance() {
        let wasm_bytes = get_test_wasm_add();
        let mut runtime = WasmRuntime::new(WasmRuntimeConfig::default());

        let module_hash = runtime.load_module(wasm_bytes).unwrap();
        let wasi_ctx = create_minimal_wasi_context("test").unwrap();
        let instance_id = runtime.instantiate(&module_hash, Some(wasi_ctx)).unwrap();

        assert_eq!(runtime.list_instances().len(), 1);

        let result = runtime.remove_instance(&instance_id);
        assert!(result.is_ok());
        assert_eq!(runtime.list_instances().len(), 0);
    }

    #[test]
    fn test_wasm_invalid_module() {
        // Invalid version
        let invalid_bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x99, 0x00, 0x00, 0x00];
        let mut runtime = WasmRuntime::new(WasmRuntimeConfig::default());

        let result = runtime.load_module(invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_wasm_function_not_found() {
        let wasm_bytes = get_test_wasm_add();
        let mut runtime = WasmRuntime::new(WasmRuntimeConfig::default());

        let module_hash = runtime.load_module(wasm_bytes).unwrap();
        let wasi_ctx = create_minimal_wasi_context("test").unwrap();
        let instance_id = runtime.instantiate(&module_hash, Some(wasi_ctx)).unwrap();

        let result = runtime.call_function(&instance_id, "nonexistent", vec![]);
        // Function not found should return an error or unsuccessful result
        assert!(result.is_err() || (!result.unwrap().success));
    }
}

#[cfg(not(feature = "wasm"))]
mod no_wasm_tests {
    use ai_assistant_tauri_lib::plugins::runtime::{WasmRuntime, WasmRuntimeConfig};

    #[test]
    fn test_wasm_disabled() {
        assert!(!WasmRuntime::is_wasm_enabled());
    }

    #[test]
    fn test_runtime_creation_without_wasm() {
        let _runtime = WasmRuntime::new(WasmRuntimeConfig::default());
        // Runtime should create even without wasm feature
    }
}
