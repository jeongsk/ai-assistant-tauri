// Sidecar management and Agent communication
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

static AGENT_READY: AtomicBool = AtomicBool::new(false);

/// Agent request
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// Agent response
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Initialize the agent runtime (sidecar)
#[tauri::command]
pub async fn init_agent(app: tauri::AppHandle) -> Result<String, String> {
    if AGENT_READY.load(Ordering::SeqCst) {
        return Ok("Agent already initialized".to_string());
    }

    // Start sidecar
    let shell = app.shell();
    let sidecar = shell
        .sidecar("agent-runtime")
        .map_err(|e| format!("Failed to create sidecar: {}", e))?;

    let (mut rx, _child) = sidecar
        .spawn()
        .map_err(|e| format!("Failed to spawn sidecar: {}", e))?;

    // Wait for ready signal
    // TODO: Implement proper handshake
    
    AGENT_READY.store(true, Ordering::SeqCst);
    Ok("Agent initialized".to_string())
}

/// Send request to agent runtime
#[tauri::command]
pub async fn agent_chat(
    app: tauri::AppHandle,
    messages: Vec<super::Message>,
    provider: Option<String>,
) -> Result<super::ChatResponse, String> {
    if !AGENT_READY.load(Ordering::SeqCst) {
        return Err("Agent not initialized. Call init_agent first.".to_string());
    }

    // TODO: Implement actual communication with sidecar via stdio
    // For now, return placeholder
    Ok(super::ChatResponse {
        content: format!(
            "Agent response placeholder. Provider: {}. Messages: {}",
            provider.unwrap_or_else(|| "default".to_string()),
            messages.len()
        ),
        error: None,
    })
}

/// Shutdown agent runtime
#[tauri::command]
pub async fn shutdown_agent() -> Result<(), String> {
    AGENT_READY.store(false, Ordering::SeqCst);
    // TODO: Send shutdown signal to sidecar
    Ok(())
}
