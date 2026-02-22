//! Template Commands - Tauri commands for template import/export and management
//!
//! This module provides Tauri commands that wrap the template_io functionality
//! and integrate with the database for versioning and sharing.

use crate::collaboration::template_io::{
    export_template_to_json, export_templates_to_json,
    import_templates_from_json, validate_template, ConflictResolution, ImportResult,
};
use crate::collaboration::Template;
use tauri::State;

// ============================================================================
// Tauri Commands - Template Import/Export
// ============================================================================

/// Export a single template to JSON format
#[tauri::command]
pub async fn export_template(
    id: String,
    db: State<'_, crate::db::DbState>,
) -> Result<Vec<u8>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Get template from database
    let template = conn
        .query_row(
            "SELECT id, name, category, content, visibility, version, created_at, updated_at
             FROM templates WHERE id = ?1",
            [&id],
            |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    content: row.get(3)?,
                    visibility: row.get(4)?,
                    version: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| format!("Template not found: {}", e))?;

    // Export to JSON
    export_template_to_json(&template)
}

/// Export all templates to JSON format
#[tauri::command]
pub async fn export_all_templates(
    db: State<'_, crate::db::DbState>,
) -> Result<Vec<u8>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Get all templates
    let mut stmt = conn
        .prepare(
            "SELECT id, name, category, content, visibility, version, created_at, updated_at
             FROM templates ORDER BY category, name",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let templates = stmt
        .query_map([], |row| {
            Ok(Template {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                content: row.get(3)?,
                visibility: row.get(4)?,
                version: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to fetch templates: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to parse templates: {}", e))?;

    // Export to JSON
    export_templates_to_json(&templates)
}

/// Import a single template from JSON format
#[tauri::command]
pub async fn import_template(
    data: Vec<u8>,
    resolution: String, // "skip", "overwrite", "rename", "version"
    db: State<'_, crate::db::DbState>,
) -> Result<Template, String> {
    let resolution = match resolution.as_str() {
        "skip" => ConflictResolution::Skip,
        "overwrite" => ConflictResolution::Overwrite,
        "rename" => ConflictResolution::Rename,
        "version" => ConflictResolution::Version,
        _ => return Err("Invalid resolution strategy".to_string()),
    };

    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Import from JSON
    let (imported, result) = import_templates_from_json(&data, resolution)?;

    if !result.success {
        return Err(format!("Import failed: {} errors", result.error_count));
    }

    // Save to database
    if let Some(template) = imported.first() {
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO templates (id, name, category, content, visibility, version, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                 name = ?2, category = ?3, content = ?4, visibility = ?5, updated_at = ?8",
            [
                &template.id,
                &template.name,
                &template.category,
                &template.content,
                &template.visibility,
                &template.version,
                &now,
                &now,
            ],
        )
        .map_err(|e| format!("Failed to save template: {}", e))?;

        Ok(template.clone())
    } else {
        Err("No templates imported".to_string())
    }
}

/// Import multiple templates from JSON format
#[tauri::command]
pub async fn import_templates(
    data: Vec<u8>,
    resolution: String,
    db: State<'_, crate::db::DbState>,
) -> Result<ImportResult, String> {
    let resolution = match resolution.as_str() {
        "skip" => ConflictResolution::Skip,
        "overwrite" => ConflictResolution::Overwrite,
        "rename" => ConflictResolution::Rename,
        "version" => ConflictResolution::Version,
        _ => return Err("Invalid resolution strategy".to_string()),
    };

    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Import from JSON
    let (imported, result) = import_templates_from_json(&data, resolution)?;

    // Save all to database
    let now = chrono::Utc::now().to_rfc3339();

    for template in &imported {
        conn.execute(
            "INSERT INTO templates (id, name, category, content, visibility, version, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                 name = ?2, category = ?3, content = ?4, visibility = ?5, updated_at = ?8",
            [
                &template.id,
                &template.name,
                &template.category,
                &template.content,
                &template.visibility,
                &template.version,
                &template.created_at,
                &now,
            ],
        )
        .map_err(|e| format!("Failed to save template {}: {}", template.id, e))?;
    }

    Ok(result)
}

/// Validate a template structure without importing
#[tauri::command]
pub async fn validate_template_data(data: serde_json::Value) -> Result<bool, String> {
    // Convert to JSON bytes for validation
    let json_bytes = serde_json::to_vec(&data)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    // Try to parse as Template
    let template: Template = serde_json::from_slice(&json_bytes)
        .map_err(|e| format!("Invalid template format: {}", e))?;

    // Validate
    validate_template(&template)?;
    Ok(true)
}

// ============================================================================
// Tauri Commands - Template Versioning
// ============================================================================

/// Template version information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateVersion {
    pub id: i64,
    pub template_id: String,
    pub version: i32,
    pub content: String,
    pub notes: Option<String>,
    pub created_at: String,
}

/// Get version history for a template
#[tauri::command]
pub async fn get_template_versions(
    id: String,
    db: State<'_, crate::db::DbState>,
) -> Result<Vec<TemplateVersion>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Check if table exists (for backward compatibility)
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='template_versions'",
            [],
            |row| Ok(row.get::<_, i32>(0)? > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        return Ok(vec![]);
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, template_id, version, content, notes, created_at
                 FROM template_versions
                 WHERE template_id = ?1
                 ORDER BY version DESC",
        )
        .map_err(|e| format!("Failed to query versions: {}", e))?;

    let versions = stmt
        .query_map([&id], |row| {
            Ok(TemplateVersion {
                id: row.get(0)?,
                template_id: row.get(1)?,
                version: row.get(2)?,
                content: row.get(3)?,
                notes: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to parse versions: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect versions: {}", e))?;

    Ok(versions)
}

/// Create a new version of a template
#[tauri::command]
pub async fn create_template_version(
    id: String,
    notes: String,
    db: State<'_, crate::db::DbState>,
) -> Result<i64, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Ensure table exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_versions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            content TEXT NOT NULL,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(template_id, version)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_versions table: {}", e))?;

    // Get current template
    let template: Template = conn
        .query_row(
            "SELECT id, name, category, content, visibility, version, created_at, updated_at
             FROM templates WHERE id = ?1",
            [&id],
            |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    category: row.get(2)?,
                    content: row.get(3)?,
                    visibility: row.get(4)?,
                    version: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| format!("Template not found: {}", e))?;

    // Get next version number
    let version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM template_versions WHERE template_id = ?1",
            [&id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get next version: {}", e))?;

    // Serialize template
    let content = serde_json::to_string(&template)
        .map_err(|e| format!("Failed to serialize template: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO template_versions (template_id, version, content, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
        [&id, &version.to_string(), &content, &notes, &now],
    )
    .map_err(|e| format!("Failed to create version: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// Rollback a template to a specific version
#[tauri::command]
pub async fn rollback_template(
    id: String,
    version_id: i64,
    db: State<'_, crate::db::DbState>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Get version content
    let (_template_id, content): (String, String) = conn
        .query_row(
            "SELECT template_id, content FROM template_versions WHERE id = ?1",
            [version_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Version not found: {}", e))?;

    // Parse template
    let template: Template = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    // Update main template
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE templates SET name = ?1, category = ?2, content = ?3, visibility = ?4, updated_at = ?5 WHERE id = ?6",
        [&template.name, &template.category, &template.content, &template.visibility, &now, &id],
    )
    .map_err(|e| format!("Failed to rollback template: {}", e))?;

    Ok(())
}

// ============================================================================
// Tauri Commands - Team Sharing
// ============================================================================

/// Share a template to a team
#[tauri::command]
pub async fn share_template_to_team(
    id: String,
    team_id: String,
    permissions: serde_json::Value, // JSON: {read: bool, write: bool, execute: bool}
    db: State<'_, crate::db::DbState>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Verify template exists
    conn
        .query_row(
            "SELECT id FROM templates WHERE id = ?1",
            [&id],
            |_| Ok(()),
        )
        .map_err(|_| format!("Template not found: {}", id))?;

    // Parse permissions
    let permissions_json = serde_json::to_string(&permissions)
        .map_err(|e| format!("Invalid permissions: {}", e))?;

    // Ensure table exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_shares (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id TEXT NOT NULL,
            team_id TEXT NOT NULL,
            permissions TEXT NOT NULL,
            shared_by TEXT NOT NULL,
            shared_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(template_id, team_id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_shares table: {}", e))?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO template_shares (template_id, team_id, permissions, shared_by, shared_at)
             VALUES (?1, ?2, ?3, 'system', ?4)",
        [&id, &team_id, &permissions_json, &now],
    )
    .map_err(|e| format!("Failed to share template: {}", e))?;

    Ok(())
}

/// Get templates shared to a team
#[tauri::command]
pub async fn get_team_templates(
    team_id: String,
    db: State<'_, crate::db::DbState>,
) -> Result<Vec<Template>, String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    // Ensure table exists
    let table_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='template_shares'",
            [],
            |row| Ok(row.get::<_, i32>(0)? > 0),
        )
        .unwrap_or(false);

    if !table_exists {
        return Ok(vec![]);
    }

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.category, t.content, t.visibility, t.version, t.created_at, t.updated_at
                 FROM templates t
                 INNER JOIN template_shares s ON t.id = s.template_id
                 WHERE s.team_id = ?1
                 ORDER BY t.name",
        )
        .map_err(|e| format!("Failed to query team templates: {}", e))?;

    let templates = stmt
        .query_map([&team_id], |row| {
            Ok(Template {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                content: row.get(3)?,
                visibility: row.get(4)?,
                version: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to parse templates: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect templates: {}", e))?;

    Ok(templates)
}

/// Revoke template access from a team
#[tauri::command]
pub async fn revoke_template_access(
    id: String,
    team_id: String,
    db: State<'_, crate::db::DbState>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("DB lock failed: {}", e))?;

    conn.execute(
        "DELETE FROM template_shares WHERE template_id = ?1 AND team_id = ?2",
        [&id, &team_id],
    )
    .map_err(|e| format!("Failed to revoke access: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_parsing() {
        // Test resolution string parsing
        let resolutions = vec!["skip", "overwrite", "rename", "version"];
        for res in resolutions {
            match res {
                "skip" => assert!(matches!(ConflictResolution::Skip, ConflictResolution::Skip)),
                "overwrite" => assert!(matches!(ConflictResolution::Overwrite, ConflictResolution::Overwrite)),
                "rename" => assert!(matches!(ConflictResolution::Rename, ConflictResolution::Rename)),
                "version" => assert!(matches!(ConflictResolution::Version, ConflictResolution::Version)),
                _ => panic!("Invalid resolution"),
            }
        }
    }
}
