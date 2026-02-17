// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod sidecar;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Message structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Chat request
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub provider: Option<String>,
}

/// Chat response
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub error: Option<String>,
}

/// Folder permission
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderPermission {
    pub id: String,
    pub path: String,
    pub level: String, // "read" or "readwrite"
}

/// Simple greeting command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Get app version
#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Validate folder path
#[tauri::command]
fn validate_folder_path(path: &str) -> Result<bool, String> {
    let path_buf = PathBuf::from(path);
    
    if !path_buf.exists() {
        return Err("Path does not exist".to_string());
    }
    
    if !path_buf.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    Ok(true)
}

/// Check if folder is accessible
#[tauri::command]
fn check_folder_access(path: &str, permissions: Vec<FolderPermission>) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    
    for perm in permissions {
        let perm_path = PathBuf::from(&perm.path);
        if path_buf.starts_with(&perm_path) {
            return Ok(perm.level);
        }
    }
    
    Err("No permission to access this folder".to_string())
}

/// Read file content
#[tauri::command]
fn read_file_content(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Write file content
#[tauri::command]
fn write_file_content(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

/// List directory contents
#[tauri::command]
fn list_directory(path: &str) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut result = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            if let Ok(name) = entry.file_name().into_string() {
                let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    "dir"
                } else {
                    "file"
                };
                result.push(format!("{}:{}", name, file_type));
            }
        }
    }
    
    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_version,
            validate_folder_path,
            check_folder_access,
            read_file_content,
            write_file_content,
            list_directory,
            sidecar::init_agent,
            sidecar::agent_chat,
            sidecar::shutdown_agent
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
