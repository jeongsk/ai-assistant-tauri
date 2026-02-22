// Sidecar management and Agent communication

#![allow(dead_code)]

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
        // In development: target/debug/agent-runtime
        // In production: app bundle binaries folder
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;

        let bin_path = exe_path
            .parent()
            .map(|p| p.join("agent-runtime"))
            .filter(|p| p.exists())
            .or_else(|| {
                // Try parent/binaries for production build
                exe_path
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("binaries").join("agent-runtime"))
            })
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

/// Execute a recipe via agent runtime
#[tauri::command]
pub async fn execute_recipe(
    state: tauri::State<'_, Mutex<SidecarState>>,
    recipe_id: String,
    steps: Vec<serde_json::Value>,
    variables: Option<serde_json::Value>,
) -> Result<String, String> {
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let request = AgentRequest {
        jsonrpc: "2.0".to_string(),
        method: "execute_recipe".to_string(),
        params: json!({
            "recipeId": recipe_id,
            "steps": steps,
            "variables": variables
        }),
        id: uuid::Uuid::new_v4().to_string(),
    };

    let response = state_guard.with_process(|process| process.send_request(&request))?;

    if let Some(error) = response.error {
        return Err(format!("{}: {}", error.code, error.message));
    }

    let result = response.result
        .as_ref()
        .and_then(|r| r.get("result"))
        .and_then(|r| r.as_str())
        .unwrap_or("Recipe executed");

    Ok(result.to_string())
}

/// Execute a skill via agent runtime
#[tauri::command]
pub async fn execute_skill(
    state: tauri::State<'_, Mutex<SidecarState>>,
    skill_id: String,
    prompt: String,
    input: String,
) -> Result<String, String> {
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let request = AgentRequest {
        jsonrpc: "2.0".to_string(),
        method: "execute_skill".to_string(),
        params: json!({
            "skillId": skill_id,
            "prompt": prompt,
            "input": input
        }),
        id: uuid::Uuid::new_v4().to_string(),
    };

    let response = state_guard.with_process(|process| process.send_request(&request))?;

    if let Some(error) = response.error {
        return Err(format!("{}: {}", error.code, error.message));
    }

    let result = response.result
        .as_ref()
        .and_then(|r| r.get("result"))
        .and_then(|r| r.as_str())
        .unwrap_or("");

    Ok(result.to_string())
}

/// Execute a prompt via agent runtime
#[tauri::command]
pub async fn execute_prompt(
    state: tauri::State<'_, Mutex<SidecarState>>,
    prompt: String,
    context: Option<serde_json::Value>,
) -> Result<String, String> {
    let state_guard = state.lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    let request = AgentRequest {
        jsonrpc: "2.0".to_string(),
        method: "execute_prompt".to_string(),
        params: json!({
            "prompt": prompt,
            "context": context
        }),
        id: uuid::Uuid::new_v4().to_string(),
    };

    let response = state_guard.with_process(|process| process.send_request(&request))?;

    if let Some(error) = response.error {
        return Err(format!("{}: {}", error.code, error.message));
    }

    let result = response.result
        .as_ref()
        .and_then(|r| r.get("result"))
        .and_then(|r| r.as_str())
        .unwrap_or("");

    Ok(result.to_string())
}

// ============================================================================
// Voice Command Integration (v0.5)
// ============================================================================

/// Voice command execution result
#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceCommandResult {
    pub success: bool,
    pub action: String,
    pub result: Option<String>,
    pub response_audio: Option<Vec<u8>>,  // TTS response
    pub error: Option<String>,
}

/// Execute a voice command via agent runtime
#[tauri::command]
pub async fn execute_voice_command(
    state: tauri::State<'_, Mutex<SidecarState>>,
    transcript: String,
    language: Option<String>,
) -> Result<VoiceCommandResult, String> {
    use crate::voice::{VoiceAction, ParsedVoiceCommand};
    use crate::voice::commands::parse_voice_command;

    // Parse the voice command
    let parsed: ParsedVoiceCommand = parse_voice_command(transcript.clone(), language)?;

    let action_type = match &parsed.action {
        VoiceAction::ExecuteSkill { .. } => "execute_skill",
        VoiceAction::RunRecipe { .. } => "run_recipe",
        VoiceAction::SendMessage { .. } => "send_message",
        VoiceAction::OpenFeature { .. } => "open_feature",
        VoiceAction::Search { .. } => "search",
        VoiceAction::Unknown => "unknown",
    }.to_string();

    // Execute based on action type
    let result = match &parsed.action {
        VoiceAction::ExecuteSkill { skill_name } => {
            // Find skill ID by name (simplified - in production would query DB)
            let skill_id = format!("skill-{}", skill_name.to_lowercase().replace(' ', "-"));
            execute_skill(
                state,
                skill_id,
                format!("Execute skill: {}", skill_name),
                transcript.clone(),
            ).await.map_err(|e| format!("Skill execution failed: {}", e))?
        }
        VoiceAction::RunRecipe { recipe_name } => {
            // Find recipe ID by name (simplified)
            let recipe_id = format!("recipe-{}", recipe_name.to_lowercase().replace(' ', "-"));
            execute_recipe(
                state,
                recipe_id,
                vec![],
                None,
            ).await.map_err(|e| format!("Recipe execution failed: {}", e))?
        }
        VoiceAction::SendMessage { content } => {
            // Send as a chat message
            agent_chat(
                state,
                vec![super::Message {
                    role: "user".to_string(),
                    content: content.clone(),
                }],
                None,
            ).await.map(|r| r.content)?
        }
        VoiceAction::OpenFeature { feature } => {
            format!("Opening feature: {}", feature)
        }
        VoiceAction::Search { query } => {
            agent_chat(
                state,
                vec![super::Message {
                    role: "user".to_string(),
                    content: format!("Search for: {}", query),
                }],
                None,
            ).await.map(|r| r.content)?
        }
        VoiceAction::Unknown => {
            // Unknown command - treat as chat message
            agent_chat(
                state,
                vec![super::Message {
                    role: "user".to_string(),
                    content: transcript.clone(),
                }],
                None,
            ).await.map(|r| r.content)?
        }
    };

    Ok(VoiceCommandResult {
        success: true,
        action: action_type,
        result: Some(result),
        response_audio: None,  // TTS would be generated here
        error: None,
    })
}

/// Start a voice conversation session (multi-turn)
#[tauri::command]
pub async fn start_voice_conversation(
    _state: tauri::State<'_, Mutex<SidecarState>>,
    language: String,
) -> Result<String, String> {
    // Initialize a conversation session
    let session_id = uuid::Uuid::new_v4().to_string();

    // In production, this would store the session in a database
    tracing::info!("Started voice conversation session: {} (language: {})", session_id, language);

    Ok(session_id)
}

/// Continue a voice conversation (multi-turn)
#[tauri::command]
pub async fn continue_voice_conversation(
    state: tauri::State<'_, Mutex<SidecarState>>,
    session_id: String,
    transcript: String,
) -> Result<VoiceCommandResult, String> {
    // In production, this would load conversation history from the database
    tracing::info!("Continuing voice conversation: {}", session_id);

    // For now, treat as a chat message with conversation context
    let response = agent_chat(
        state,
        vec![super::Message {
            role: "user".to_string(),
            content: transcript.clone(),
        }],
        None,
    ).await?;

    Ok(VoiceCommandResult {
        success: true,
        action: "conversation".to_string(),
        result: Some(response.content),
        response_audio: None,
        error: response.error,
    })
}

/// End a voice conversation session
#[tauri::command]
pub async fn end_voice_conversation(
    _session_id: String,
) -> Result<(), String> {
    // In production, this would save the conversation history
    tracing::info!("Ended voice conversation session: {}", _session_id);
    Ok(())
}
