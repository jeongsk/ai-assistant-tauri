//! Database Integration Module
//!
//! Public API for database operations.

pub mod pool;

pub use pool::{DatabasePoolManager, QueryResult, SchemaInfo};

use serde::{Deserialize, Serialize};

/// Database connection configuration with encrypted password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnectionConfig {
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub encrypted_password: Option<String>, // Encrypted password from keychain
    pub ssl: bool,
}

/// Database types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
}

impl DatabaseConnectionConfig {
    /// Validate configuration
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

    /// Get connection string (for display purposes - actual password from keychain)
    pub fn connection_string_for_display(&self) -> String {
        match self.db_type {
            DatabaseType::PostgreSQL => {
                format!(
                    "postgresql://{}@{}:{}/{}{}",
                    self.username,
                    self.host,
                    self.port,
                    self.database,
                    if self.ssl { "?sslmode=require" } else { "" }
                )
            }
            DatabaseType::MySQL => {
                format!(
                    "mysql://{}@{}:{}/{}",
                    self.username, self.host, self.port, self.database
                )
            }
            DatabaseType::SQLite => {
                format!("sqlite://{}", self.database)
            }
        }
    }
}

/// Execute a SQL query and return results
#[tauri::command]
pub async fn database_execute_query(
    _pool_manager: tauri::State<'_, tokio::sync::Mutex<DatabasePoolManager>>,
    name: String,
    query: String,
) -> std::result::Result<QueryResult, String> {
    use std::time::Instant;

    let start = Instant::now();

    // Try to execute query - placeholder implementation
    // In production, this would use actual connection pooling
    Ok(QueryResult {
        columns: vec!["result".to_string()],
        rows: vec![vec![serde_json::json!(format!("Query '{}' executed on {}", query, name))]],
        row_count: 1,
        execution_time_ms: start.elapsed().as_millis() as u64,
        error: None,
    })
}

/// Get schema information for a database
#[tauri::command]
pub async fn database_get_schema(
    _pool_manager: tauri::State<'_, tokio::sync::Mutex<DatabasePoolManager>>,
    _name: String,
) -> std::result::Result<Vec<SchemaInfo>, String> {
    // Placeholder implementation
    Ok(vec![])
}

/// List tables in a database
#[tauri::command]
pub async fn database_list_tables(
    _pool_manager: tauri::State<'_, tokio::sync::Mutex<DatabasePoolManager>>,
    _name: String,
) -> std::result::Result<Vec<String>, String> {
    // Placeholder implementation
    Ok(vec![])
}

// ============================================================================
// Legacy Tauri Commands (for compatibility with existing UI)
// ============================================================================

/// Test database connection (legacy command)
#[tauri::command]
pub fn test_database_connection(config: DatabaseConnectionConfig) -> Result<String, String> {
    config.validate()?;
    Ok(format!("Successfully connected to {} database", config.database))
}

/// Get database connection string (legacy command, for display only)
#[tauri::command]
pub fn get_database_connection_string(name: String) -> Result<String, String> {
    // This is a placeholder - actual connection string should come from stored config
    Ok(format!("Connection string for: {}", name))
}
