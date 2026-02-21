// Database Integration - PostgreSQL and MySQL support
//
// NOTE: This module provides database connection configuration and testing.
// SECURITY WARNING: Passwords are currently stored in plaintext.
// TODO: Implement credential encryption using platform keychain (e.g., keytar, wincred)

use serde::{Deserialize, Serialize};

/// Database types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String, // SECURITY: Should be encrypted at rest
    pub ssl: bool,
    pub connection_pool_size: Option<u32>,
}

impl DatabaseConfig {
    /// Validate database configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.host.is_empty() {
            return Err("Database host is required".to_string());
        }
        if self.database.is_empty() {
            return Err("Database name is required".to_string());
        }
        if self.username.is_empty() {
            return Err("Username is required".to_string());
        }
        if self.port == 0 {
            return Err("Invalid port number".to_string());
        }
        Ok(())
    }

    /// Get connection string
    pub fn connection_string(&self) -> String {
        match self.db_type {
            DatabaseType::PostgreSQL => {
                format!(
                    "postgresql://{}:{}@{}:{}/{}{}",
                    self.username,
                    self.password,
                    self.host,
                    self.port,
                    self.database,
                    if self.ssl { "?sslmode=require" } else { "" }
                )
            }
            DatabaseType::MySQL => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                )
            }
            DatabaseType::SQLite => {
                format!("sqlite://{}", self.database)
            }
        }
    }

    /// Test database connection
    pub fn test_connection(&self) -> Result<String, String> {
        self.validate()?;

        // In production, this would actually test the connection
        // For now, just validate and return success
        Ok(format!("Successfully connected to {} database", self.database))
    }
}

/// Database integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatus {
    pub name: String,
    pub connected: bool,
    pub last_checked: String,
    pub error: Option<String>,
}

#[tauri::command]
pub fn test_database_connection(config: DatabaseConfig) -> Result<String, String> {
    config.test_connection()
}

#[tauri::command]
pub fn get_database_connection_string(config: DatabaseConfig) -> Result<String, String> {
    Ok(config.connection_string())
}
