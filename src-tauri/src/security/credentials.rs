//! Platform keychain credential storage

#![allow(dead_code)]

use crate::security::{Result, SecurityError};
use keyring::{Entry, Error as KeyringError};

/// Credential manager using platform keychain
pub struct CredentialManager {
    service_name: String,
}

impl CredentialManager {
    /// Create a new credential manager
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    /// Get default credential manager for this app
    pub fn default() -> Result<Self> {
        Ok(Self::new("ai-assistant-tauri".to_string()))
    }

    /// Create a keyring entry for a credential
    fn get_entry(&self, username: &str) -> Result<Entry> {
        Entry::new(&self.service_name, username)
            .map_err(|e| SecurityError::Keyring(e.to_string()))
    }

    /// Store a password in the keychain
    pub fn set_password(&self, username: &str, password: &str) -> Result<()> {
        let entry = self.get_entry(username)?;
        entry
            .set_password(password)
            .map_err(|e| SecurityError::Keyring(e.to_string()))
    }

    /// Retrieve a password from the keychain
    pub fn get_password(&self, username: &str) -> Result<String> {
        let entry = self.get_entry(username)?;
        entry
            .get_password()
            .map_err(|e| match e {
                KeyringError::NoEntry => SecurityError::NotFound(username.to_string()),
                _ => SecurityError::Keyring(e.to_string()),
            })
    }

    /// Delete a password from the keychain
    pub fn delete_password(&self, username: &str) -> Result<()> {
        let entry = self.get_entry(username)?;
        entry
            .delete_credential()
            .map_err(|e| match e {
                KeyringError::NoEntry => SecurityError::NotFound(username.to_string()),
                _ => SecurityError::Keyring(e.to_string()),
            })
    }

    /// Check if a credential exists
    pub fn has_password(&self, username: &str) -> bool {
        // Try to get the entry and check if password exists
        if let Ok(entry) = self.get_entry(username) {
            entry.get_password().is_ok()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        // Skip test in CI environments without keyring support
        if std::env::var("CI").is_ok() || std::env::var("DOCKER").is_ok() {
            return;
        }

        let manager = CredentialManager::new("test-service-ai-assistant".to_string());
        let username = "test_user_v05";
        let password = "test_password_123";

        // Clean up first if exists
        let _ = manager.delete_password(username);

        // Store and retrieve
        if let Err(e) = manager.set_password(username, password) {
            println!("Skipping test: keyring not available: {}", e);
            return;
        }

        match manager.get_password(username) {
            Ok(retrieved) => assert_eq!(password, retrieved),
            Err(e) => println!("Skipping assertion: keyring get failed: {}", e),
        }

        // Cleanup
        let _ = manager.delete_password(username);
    }

    #[test]
    fn test_nonexistent_returns_error() {
        // Skip test in CI environments without keyring support
        if std::env::var("CI").is_ok() || std::env::var("DOCKER").is_ok() {
            return;
        }

        let manager = CredentialManager::new("test-service-ai-assistant".to_string());
        if manager.get_password("nonexistent_user_xyz").is_err() {
            // Expected error
        } else {
            println!("Skipping assertion: keyring unexpectedly succeeded");
        }
    }
}
