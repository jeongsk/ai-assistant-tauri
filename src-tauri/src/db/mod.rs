// Database Module - SQLite persistence

pub mod schema;

use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

/// Database state managed by Tauri
pub struct DbState(pub Mutex<Connection>);

impl DbState {
    pub fn new(app_handle: &tauri::AppHandle) -> SqliteResult<Self> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data directory");

        std::fs::create_dir_all(&app_dir).ok();
        let db_path = PathBuf::from(&app_dir).join("assistant.db");

        let conn = Connection::open(db_path)?;
        schema::run_migrations(&conn)?;

        Ok(Self(Mutex::new(conn)))
    }
}

/// Conversation model
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Message model
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub metadata: Option<String>,
    pub created_at: String,
}

/// Folder permission model
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FolderPermission {
    pub id: String,
    pub path: String,
    pub level: String,
    pub created_at: String,
}

// Tauri commands for database operations

#[tauri::command]
pub fn load_conversations(db: tauri::State<'_, DbState>) -> Result<Vec<Conversation>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC"
        )
        .map_err(|e| e.to_string())?;

    let conversations = stmt
        .query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(conversations)
}

#[tauri::command]
pub fn save_conversation(
    db: tauri::State<'_, DbState>,
    id: String,
    title: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO conversations (id, title, created_at, updated_at)
         VALUES (?1, ?2, COALESCE((SELECT created_at FROM conversations WHERE id = ?1), ?3), ?4)",
        [&id, &title, &now, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_conversation(db: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Delete messages first
    conn.execute("DELETE FROM messages WHERE conversation_id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    // Delete conversation
    conn.execute("DELETE FROM conversations WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn load_messages(
    db: tauri::State<'_, DbState>,
    conversation_id: String,
) -> Result<Vec<Message>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, conversation_id, role, content, metadata, created_at
             FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC"
        )
        .map_err(|e| e.to_string())?;

    let messages = stmt
        .query_map([&conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                metadata: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(messages)
}

#[tauri::command]
pub fn save_message(
    db: tauri::State<'_, DbState>,
    id: String,
    conversation_id: String,
    role: String,
    content: String,
    metadata: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, metadata, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [&id, &conversation_id, &role, &content, &metadata.unwrap_or_default(), &now],
    )
    .map_err(|e| e.to_string())?;

    // Update conversation timestamp
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        [&now, &conversation_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn load_folder_permissions(
    db: tauri::State<'_, DbState>,
) -> Result<Vec<FolderPermission>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, path, level, created_at FROM folder_permissions ORDER BY path")
        .map_err(|e| e.to_string())?;

    let permissions = stmt
        .query_map([], |row| {
            Ok(FolderPermission {
                id: row.get(0)?,
                path: row.get(1)?,
                level: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(permissions)
}

#[tauri::command]
pub fn add_folder_permission(
    db: tauri::State<'_, DbState>,
    id: String,
    path: String,
    level: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO folder_permissions (id, path, level, created_at) VALUES (?1, ?2, ?3, ?4)",
        [&id, &path, &level, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_folder_permission(db: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM folder_permissions WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_folder_permission(
    db: tauri::State<'_, DbState>,
    id: String,
    level: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE folder_permissions SET level = ?1 WHERE id = ?2",
        [&level, &id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
