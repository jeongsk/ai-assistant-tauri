//! Security Module - Credential encryption and keychain integration
//!
//! This module provides secure credential storage using platform keychains
//! and AES-256-GCM encryption for sensitive data.

#![allow(dead_code)]

pub mod credentials;
pub mod encryption;
pub mod migration;

pub use credentials::CredentialManager;
pub use migration::migrate_plaintext_passwords;

use std::sync::Mutex;

/// Security error types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Invalid credential format")]
    InvalidFormat,

    #[error("Credential not found: {0}")]
    NotFound(String),
}

/// Result type for security operations
pub type Result<T> = std::result::Result<T, SecurityError>;

// ============================================================================
// Tauri Commands for Credential Management
// ============================================================================

/// Set a password in the keychain
#[tauri::command]
pub fn credentials_set_password(
    manager: tauri::State<'_, Mutex<CredentialManager>>,
    username: String,
    password: String,
) -> std::result::Result<(), String> {
    let mgr = manager.lock().map_err(|e| e.to_string())?;
    mgr.set_password(&username, &password).map_err(|e| e.to_string())
}

/// Get a password from the keychain
#[tauri::command]
pub fn credentials_get_password(
    manager: tauri::State<'_, Mutex<CredentialManager>>,
    username: String,
) -> std::result::Result<String, String> {
    let mgr = manager.lock().map_err(|e| e.to_string())?;
    mgr.get_password(&username).map_err(|e| e.to_string())
}

/// Delete a password from the keychain
#[tauri::command]
pub fn credentials_delete_password(
    manager: tauri::State<'_, Mutex<CredentialManager>>,
    username: String,
) -> std::result::Result<(), String> {
    let mgr = manager.lock().map_err(|e| e.to_string())?;
    mgr.delete_password(&username).map_err(|e| e.to_string())
}

/// Run migration to encrypt plaintext passwords
#[tauri::command]
pub fn run_migration(
    db: tauri::State<'_, crate::db::DbState>,
) -> std::result::Result<usize, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    migrate_plaintext_passwords(&conn).map_err(|e| e.to_string())
}
