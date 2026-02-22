// Collaboration Module

#![allow(dead_code)]

pub mod templates;
pub mod export_mod;
pub mod template_io;
pub mod template_commands;
pub mod marketplace;

use serde::{Deserialize, Serialize};
use tauri::State;

/// Template visibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
    Private,
    Public,
    Team,
}

/// Template model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub category: String,
    pub content: String,
    pub visibility: String,
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Shared workflow model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedWorkflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: String,
    pub owner_id: Option<String>,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Markdown,
    Html,
}

/// Export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_timestamps: bool,
    pub pretty_print: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_metadata: true,
            include_timestamps: true,
            pretty_print: true,
        }
    }
}

// ============================================================================
// Workflow Tauri Commands - CRUD operations for SharedWorkflow
// ============================================================================

/// List all workflows
#[tauri::command]
pub fn list_workflows(db: State<'_, crate::db::DbState>) -> Result<Vec<SharedWorkflow>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, steps, owner_id, visibility, created_at, updated_at
             FROM shared_workflows ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let workflows = stmt
        .query_map([], |row| {
            Ok(SharedWorkflow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                steps: row.get(3)?,
                owner_id: row.get(4)?,
                visibility: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(workflows)
}

/// Get a single workflow by ID
#[tauri::command]
pub fn get_workflow(db: State<'_, crate::db::DbState>, id: String) -> Result<SharedWorkflow, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let workflow = conn
        .query_row(
            "SELECT id, name, description, steps, owner_id, visibility, created_at, updated_at
             FROM shared_workflows WHERE id = ?1",
            [&id],
            |row| {
                Ok(SharedWorkflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    steps: row.get(3)?,
                    owner_id: row.get(4)?,
                    visibility: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(workflow)
}

/// Create a new workflow
#[tauri::command]
pub fn create_workflow(
    db: State<'_, crate::db::DbState>,
    id: String,
    name: String,
    description: Option<String>,
    steps: String,
    owner_id: Option<String>,
    visibility: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO shared_workflows (id, name, description, steps, owner_id, visibility, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [
            &id,
            &name,
            &description.unwrap_or_default(),
            &steps,
            &owner_id.unwrap_or_default(),
            &visibility,
            &now,
            &now,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Update an existing workflow
#[tauri::command]
pub fn update_workflow(
    db: State<'_, crate::db::DbState>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    steps: Option<String>,
    owner_id: Option<String>,
    visibility: Option<String>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    // Build dynamic update query
    let mut updates = vec!["updated_at = ?"];
    let mut params: Vec<String> = vec![now.clone()];

    if let Some(n) = name {
        updates.push("name = ?");
        params.push(n);
    }
    if let Some(d) = description {
        updates.push("description = ?");
        params.push(d);
    }
    if let Some(s) = steps {
        updates.push("steps = ?");
        params.push(s);
    }
    if let Some(o) = owner_id {
        updates.push("owner_id = ?");
        params.push(o);
    }
    if let Some(v) = visibility {
        updates.push("visibility = ?");
        params.push(v);
    }

    if updates.len() == 1 {
        return Ok(()); // No updates to apply
    }

    params.push(id.clone());
    let sql = format!("UPDATE shared_workflows SET {} WHERE id = ?", updates.join(", "));

    conn.execute(&sql, rusqlite::params_from_iter(params.iter()))
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Delete a workflow
#[tauri::command]
pub fn delete_workflow(db: State<'_, crate::db::DbState>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM shared_workflows WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}
