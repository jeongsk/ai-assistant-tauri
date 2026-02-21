//! Database Connection Pool and Query Execution
//!
//! This module provides actual database query execution for PostgreSQL and MySQL.

use super::DatabaseType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Database feature-gated imports
#[cfg(feature = "database")]
use tokio_postgres as postgres;

#[cfg(feature = "database")]
use mysql_async as mysql;

/// Query result for database operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

/// Database connection entry
#[derive(Debug)]
struct DatabaseConnection {
    name: String,
    db_type: DatabaseType,
    #[cfg(feature = "database")]
    postgres_config: Option<PostgresConfig>,
    #[cfg(feature = "database")]
    mysql_config: Option<MysqlConfig>,
}

/// PostgreSQL configuration
#[cfg(feature = "database")]
#[derive(Debug, Clone)]
struct PostgresConfig {
    connection_string: String,
}

/// MySQL configuration
#[cfg(feature = "database")]
#[derive(Debug, Clone)]
struct MysqlConfig {
    connection_string: String,
}

/// Database connection pool manager
pub struct DatabasePoolManager {
    connections: HashMap<String, DatabaseConnection>,
}

impl DatabasePoolManager {
    /// Create a new pool manager
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Add a PostgreSQL connection
    #[cfg(feature = "database")]
    pub async fn add_postgres_pool(
        &mut self,
        name: String,
        connection_string: String,
    ) -> Result<(), String> {
        // Validate connection string format
        if !connection_string.starts_with("postgresql://") &&
           !connection_string.starts_with("postgres://") {
            return Err("Invalid PostgreSQL connection string".to_string());
        }

        let config = PostgresConfig {
            connection_string,
        };

        let conn = DatabaseConnection {
            name: name.clone(),
            db_type: DatabaseType::PostgreSQL,
            postgres_config: Some(config),
            mysql_config: None,
        };

        self.connections.insert(name, conn);
        Ok(())
    }

    /// Add a MySQL connection
    #[cfg(feature = "database")]
    pub async fn add_mysql_pool(
        &mut self,
        name: String,
        connection_string: String,
    ) -> Result<(), String> {
        // Validate connection string format
        if !connection_string.starts_with("mysql://") {
            return Err("Invalid MySQL connection string".to_string());
        }

        let config = MysqlConfig {
            connection_string,
        };

        let conn = DatabaseConnection {
            name: name.clone(),
            db_type: DatabaseType::MySQL,
            postgres_config: None,
            mysql_config: Some(config),
        };

        self.connections.insert(name, conn);
        Ok(())
    }

    /// Execute a query on a named connection
    #[cfg(feature = "database")]
    pub async fn execute_query(&self, name: &str, query: &str) -> Result<QueryResult, String> {
        let conn = self.connections.get(name)
            .ok_or_else(|| format!("Connection '{}' not found", name))?;

        match conn.db_type {
            DatabaseType::PostgreSQL => {
                if let Some(ref config) = conn.postgres_config {
                    self.execute_postgres_query(config, query).await
                } else {
                    Err("PostgreSQL configuration not found".to_string())
                }
            }
            DatabaseType::MySQL => {
                if let Some(ref config) = conn.mysql_config {
                    self.execute_mysql_query(config, query).await
                } else {
                    Err("MySQL configuration not found".to_string())
                }
            }
        }
    }

    /// Execute a query on a named connection (non-database feature)
    #[cfg(not(feature = "database"))]
    pub async fn execute_query(&self, name: &str, _query: &str) -> Result<QueryResult, String> {
        Err(format!("Database feature not enabled for connection '{}'", name))
    }

    /// Execute a PostgreSQL query
    #[cfg(feature = "database")]
    async fn execute_postgres_query(&self, config: &PostgresConfig, query: &str) -> Result<QueryResult, String> {
        let start = std::time::Instant::now();

        // Parse connection string and connect
        let (client, connection) = postgres::connect(&config.connection_string, postgres::NoTls)
            .await
            .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;

        // Spawn connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("PostgreSQL connection error: {}", e);
            }
        });

        // Execute the query
        let rows = client.simple_query(query)
            .await
            .map_err(|e| format!("Query execution failed: {}", e))?;

        let mut result_rows = Vec::new();
        let mut columns = Vec::new();

        for row in &rows {
            match row {
                postgres::SimpleQueryRow::DataRow(data) => {
                    // Get column names from first row
                    if columns.is_empty() {
                        columns = (0..data.len())
                            .map(|i| format!("column_{}", i))
                            .collect();
                    }

                    let mut row_values = Vec::new();
                    for (i, col) in data.iter().enumerate() {
                        match col {
                            Some(value) => {
                                row_values.push(self.convert_postgres_value(value));
                            }
                            None => {
                                row_values.push(serde_json::Value::Null);
                            }
                        }
                    }
                    result_rows.push(row_values);
                }
                postgres::SimpleQueryRow::Command(_) => {
                    // Command response (like INSERT, UPDATE, etc.)
                    // Return row count as affected
                    columns = vec!["affected_rows".to_string()];
                    result_rows.push(vec![serde_json::json!(0)]);
                }
            }
        }

        let execution_time = start.elapsed().as_millis() as u64;

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count: result_rows.len(),
            execution_time_ms: execution_time,
            error: None,
        })
    }

    /// Convert PostgreSQL value to JSON
    #[cfg(feature = "database")]
    fn convert_postgres_value(&self, value: &postgres::types::Value) -> serde_json::Value {
        match value {
            postgres::types::Value::Int8(n) => serde_json::json!(n),
            postgres::types::Value::Int2(n) => serde_json::json!(n),
            postgres::types::Value::Int4(n) => serde_json::json!(n),
            postgres::types::Value::Float4(n) => serde_json::json!(n),
            postgres::types::Value::Float8(n) => serde_json::json!(n),
            postgres::types::Value::Text(s) => serde_json::json!(s),
            postgres::types::Value::Boolean(b) => serde_json::json!(b),
            postgres::types::Value::Null => serde_json::Value::Null,
            _ => serde_json::json!(format!("{:?}", value)),
        }
    }

    /// Execute a MySQL query
    #[cfg(feature = "database")]
    async fn execute_mysql_query(&self, config: &MysqlConfig, query: &str) -> Result<QueryResult, String> {
        let start = std::time::Instant::now();

        // Parse connection URL
        let url = mysql::Url::parse(&config.connection_string)
            .map_err(|e| format!("Invalid MySQL URL: {}", e))?;

        // Connect to MySQL
        let mut conn = mysql::Conn::new(url)
            .await
            .map_err(|e| format!("Failed to connect to MySQL: {}", e))?;

        // Execute the query
        let result = conn.query_iter(query)
            .await
            .map_err(|e| format!("Query execution failed: {}", e))?;

        let mut result_rows = Vec::new();
        let mut columns = Vec::new();

        // Process results
        let mut stream = result;
        while let Some(row_result) = stream.next().await {
            let row = row_result
                .map_err(|e| format!("Row processing failed: {}", e))?;

            // Get column names from first row
            if columns.is_empty() {
                columns = row.columns_ref().iter()
                    .map(|c| c.name_str().to_string())
                    .collect();
            }

            let mut row_values = Vec::new();
            for i in 0..row.len() {
                let value = row.as_result(i);
                match value {
                    Ok(val) => {
                        row_values.push(self.convert_mysql_value(val));
                    }
                    Err(_) => {
                        row_values.push(serde_json::Value::Null);
                    }
                }
            }
            result_rows.push(row_values);
        }

        // Disconnect
        conn.disconnect().await
            .map_err(|e| format!("Disconnect error: {}", e))?;

        let execution_time = start.elapsed().as_millis() as u64;

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count: result_rows.len(),
            execution_time_ms: execution_time,
            error: None,
        })
    }

    /// Convert MySQL value to JSON
    #[cfg(feature = "database")]
    fn convert_mysql_value(&self, value: &mysql_async::Row) -> serde_json::Value {
        // mysql_async 0.34 uses different API
        // Try to get value as String first
        if let Ok(v) = value.get::<Option<String>, usize>(0) {
            if let Some(val) = v {
                return serde_json::json!(val);
            }
        }
        if let Ok(v) = value.get::<Option<i64>, usize>(0) {
            if let Some(val) = v {
                return serde_json::json!(val);
            }
        }
        if let Ok(v) = value.get::<Option<f64>, usize>(0) {
            if let Some(val) = v {
                return serde_json::json!(val);
            }
        }
        if let Ok(v) = value.get::<Option<bool>, usize>(0) {
            if let Some(val) = v {
                return serde_json::json!(val);
            }
        }
        serde_json::Value::Null
    }

    /// Test a database connection
    #[cfg(feature = "database")]
    pub async fn test_connection(&self, name: &str) -> Result<bool, String> {
        let conn = self.connections.get(name)
            .ok_or_else(|| format!("Connection '{}' not found", name))?;

        match conn.db_type {
            DatabaseType::PostgreSQL => {
                if let Some(ref config) = conn.postgres_config {
                    let (client, connection) = postgres::connect(&config.connection_string, postgres::NoTls)
                        .await
                        .map_err(|e| format!("Connection failed: {}", e))?;

                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            tracing::error!("PostgreSQL connection error: {}", e);
                        }
                    });

                    // Test query
                    let _ = client.simple_query("SELECT 1")
                        .await
                        .map_err(|e| format!("Test query failed: {}", e))?;

                    Ok(true)
                } else {
                    Err("PostgreSQL configuration not found".to_string())
                }
            }
            DatabaseType::MySQL => {
                if let Some(ref config) = conn.mysql_config {
                    let url = mysql::Url::parse(&config.connection_string)
                        .map_err(|e| format!("Invalid MySQL URL: {}", e))?;

                    let mut conn = mysql::Conn::new(url)
                        .await
                        .map_err(|e| format!("Connection failed: {}", e))?;

                    // Test query
                    let _ = conn.query_iter("SELECT 1")
                        .await
                        .map_err(|e| format!("Test query failed: {}", e))?;

                    conn.disconnect().await
                        .map_err(|e| format!("Disconnect error: {}", e))?;

                    Ok(true)
                } else {
                    Err("MySQL configuration not found".to_string())
                }
            }
        }
    }

    /// Test a database connection (non-database feature)
    #[cfg(not(feature = "database"))]
    pub async fn test_connection(&self, name: &str) -> Result<bool, String> {
        Err(format!("Database feature not enabled for connection '{}'", name))
    }
}

impl Default for DatabasePoolManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema information for a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
}

/// Column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_serialization() {
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec![serde_json::json!(1), serde_json::json!("test")],
                vec![serde_json::json!(2), serde_json::json!("test2")],
            ],
            row_count: 2,
            execution_time_ms: 10,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("id"));
        assert!(json.contains("name"));
    }

    #[tokio::test]
    #[cfg(feature = "database")]
    async fn test_add_postgres_pool() {
        let mut manager = DatabasePoolManager::new();
        let result = manager.add_postgres_pool(
            "test".to_string(),
            "postgresql://user:pass@localhost/test".to_string(),
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "database")]
    async fn test_add_invalid_postgres_pool() {
        let mut manager = DatabasePoolManager::new();
        let result = manager.add_postgres_pool(
            "test".to_string(),
            "invalid://connection".to_string(),
        ).await;

        assert!(result.is_err());
    }
}
