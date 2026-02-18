// Sidecar management and Agent communication
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;

/// Agent request
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: String,
}

/// Agent response
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<AgentError>,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentError {
    pub code: i32,
    pub message: String,
}

/// Sidecar process wrapper
pub(crate) struct SidecarProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl SidecarProcess {
    fn spawn() -> Result<Self, String> {
        // Find the agent-runtime binary
        let bin_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("binaries").join("agent-runtime"))
            .ok_or_else(|| "Failed to determine binaries path".to_string())?;

        if !bin_path.exists() {
            return Err(format!("Agent runtime binary not found at {:?}", bin_path));
        }

        // Spawn the process
        let mut child = Command::new(&bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn agent runtime: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    fn send_request(&mut self, request: &AgentRequest) -> Result<AgentResponse, String> {
        let request_str = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        writeln!(self.stdin, "{}", request_str)
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;

        self.stdin.flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;

        // Read response
        let mut response_str = String::new();
        self.stdout.read_line(&mut response_str)
            .map_err(|e| format!("Failed to read from stdout: {}", e))?;

        serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    fn wait_for_ready(&mut self) -> Result<(), String> {
        let mut ready_line = String::new();
        self.stdout.read_line(&mut ready_line)
            .map_err(|e| format!("Failed to read ready signal: {}", e))?;

        let response: AgentResponse = serde_json::from_str(&ready_line)
            .map_err(|e| format!("Failed to parse ready signal: {}", e))?;

        if response.result.as_ref()
            .and_then(|r| r.get("status"))
            .and_then(|s| s.as_str())
            != Some("ready") {
            return Err("Sidecar not ready".to_string());
        }

        Ok(())
    }
}

impl Drop for SidecarProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// Global sidecar state
pub struct SidecarState {
    process: Mutex<Option<SidecarProcess>>,
}

unsafe impl Send for SidecarState {}

impl SidecarState {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.process.lock().unwrap().is_some()
    }

    pub fn set_initialized(&self, process: SidecarProcess) {
        *self.process.lock().unwrap() = Some(process);
    }

    pub fn reset(&self) {
        *self.process.lock().unwrap() = None;
    }

    pub fn with_process<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut SidecarProcess) -> Result<R, String>,
    {
        let mut guard = self.process.lock().unwrap();
        let process = guard.as_mut()
            .ok_or_else(|| "Sidecar not initialized".to_string())?;
        f(process)
    }
}

/// Initialize the agent runtime (sidecar)
#[tauri::command]
pub async fn init_agent(
    state: tauri::State<'_, Mutex<SidecarState>>,
) -> Result<String, String> {
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if state_guard.is_initialized() {
        return Ok("Agent already initialized".to_string());
    }

    drop(state_guard);

    // Spawn and initialize the sidecar process
    let mut process = SidecarProcess::spawn()?;
    process.wait_for_ready()?;

    // Store the process
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    if state_guard.is_initialized() {
        return Ok("Agent already initialized".to_string());
    }

    state_guard.set_initialized(process);

    Ok("Agent initialized".to_string())
}

/// Send request to agent runtime
#[tauri::command]
pub async fn agent_chat(
    state: tauri::State<'_, Mutex<SidecarState>>,
    messages: Vec<super::Message>,
    provider: Option<String>,
) -> Result<super::ChatResponse, String> {
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let request = AgentRequest {
        jsonrpc: "2.0".to_string(),
        method: "chat".to_string(),
        params: json!({
            "messages": messages,
            "options": {
                "provider": provider
            }
        }),
        id: uuid::Uuid::new_v4().to_string(),
    };

    let response = state_guard.with_process(|process| process.send_request(&request))?;

    if let Some(error) = response.error {
        return Ok(super::ChatResponse {
            content: String::new(),
            error: Some(format!("{}: {}", error.code, error.message)),
        });
    }

    let content = response.result
        .as_ref()
        .and_then(|r| r.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    Ok(super::ChatResponse {
        content,
        error: None,
    })
}

/// Get available tools from agent
#[tauri::command]
pub async fn get_tools(
    state: tauri::State<'_, Mutex<SidecarState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let request = AgentRequest {
        jsonrpc: "2.0".to_string(),
        method: "get_tools".to_string(),
        params: json!({}),
        id: uuid::Uuid::new_v4().to_string(),
    };

    let response = state_guard.with_process(|process| process.send_request(&request))?;

    if let Some(error) = response.error {
        return Err(format!("{}: {}", error.code, error.message));
    }

    let tools = response.result
        .as_ref()
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(tools)
}

/// Configure providers
#[tauri::command]
pub async fn configure_providers(
    state: tauri::State<'_, Mutex<SidecarState>>,
    providers: Vec<serde_json::Value>,
    active_provider: Option<String>,
) -> Result<String, String> {
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let request = AgentRequest {
        jsonrpc: "2.0".to_string(),
        method: "configure_providers".to_string(),
        params: json!({
            "providers": providers,
            "activeProvider": active_provider
        }),
        id: uuid::Uuid::new_v4().to_string(),
    };

    let response = state_guard.with_process(|process| process.send_request(&request))?;

    if let Some(error) = response.error {
        return Err(format!("{}: {}", error.code, error.message));
    }

    Ok("Providers configured".to_string())
}

/// Shutdown agent runtime
#[tauri::command]
pub async fn shutdown_agent(
    state: tauri::State<'_, Mutex<SidecarState>>,
) -> Result<(), String> {
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    // Send shutdown request if process exists
    if state_guard.is_initialized() {
        let request = AgentRequest {
            jsonrpc: "2.0".to_string(),
            method: "shutdown".to_string(),
            params: json!({}),
            id: uuid::Uuid::new_v4().to_string(),
        };

        let _ = state_guard.with_process(|process| process.send_request(&request));
    }

    state_guard.reset();
    Ok(())
}
