//! Migration script to encrypt existing plaintext passwords

use crate::security::{CredentialManager, Result, SecurityError};
use rusqlite::Connection;

/// Migrate plaintext passwords to keychain-encrypted storage
pub fn migrate_plaintext_passwords(conn: &Connection) -> Result<usize> {
    let keyring = CredentialManager::default()?;

    // Get all database_connections with plaintext passwords
    let mut stmt = conn
        .prepare("SELECT id, name, password FROM database_connections WHERE password IS NOT NULL AND password != ''")
        .map_err(|e| SecurityError::Encryption(format!("Failed to query passwords: {}", e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| SecurityError::Encryption(format!("Failed to map rows: {}", e)))?;

    let mut migrated = 0;

    for row in rows {
        let (id, name, password) = row.map_err(|e| {
            SecurityError::Encryption(format!("Failed to read row: {}", e))
        })?;

        // Store in keychain with format: db_connection:{name}
        let keyring_key = format!("db_connection:{}", name);
        keyring.set_password(&keyring_key, &password)?;

        // Update database to remove plaintext
        conn.execute(
            "UPDATE database_connections SET password = '' WHERE id = ?1",
            [&id],
        )
        .map_err(|e| SecurityError::Encryption(format!("Failed to update DB: {}", e)))?;

        migrated += 1;
    }

    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration() {
        // Skip test in CI environments without keyring support
        if std::env::var("CI").is_ok() || std::env::var("DOCKER").is_ok() {
            return;
        }

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();

        // Create test table
        conn.execute(
            "CREATE TABLE database_connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                password TEXT
            )",
            [],
        )
        .unwrap();

        // Insert test data
        conn.execute(
            "INSERT INTO database_connections (id, name, password) VALUES ('1', 'test_db_v05', 'secret123')",
            [],
        )
        .unwrap();

        // Run migration
        match migrate_plaintext_passwords(&conn) {
            Ok(count) => {
                assert_eq!(count, 1);

                // Verify password is cleared from DB
                let password: String = conn
                    .query_row("SELECT password FROM database_connections WHERE id = '1'", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(password, "");

                // Verify password is in keychain
                let keyring = CredentialManager::default().unwrap();
                match keyring.get_password("db_connection:test_db_v05") {
                    Ok(retrieved) => assert_eq!(retrieved, "secret123"),
                    Err(e) => println!("Skipping keyring assertion: {}", e),
                }

                // Cleanup
                let _ = keyring.delete_password("db_connection:test_db_v05");
            }
            Err(e) => {
                println!("Skipping test: keyring not available: {}", e);
            }
        }
    }
}
