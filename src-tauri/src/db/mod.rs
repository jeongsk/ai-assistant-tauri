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

// ============================================================================
// Skill Model and Commands
// ============================================================================

/// Skill model
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub tools: String, // JSON array
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn list_skills(db: tauri::State<'_, DbState>) -> Result<Vec<Skill>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, description, prompt, tools, created_at, updated_at FROM skills ORDER BY name")
        .map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt: row.get(3)?,
                tools: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(skills)
}

#[tauri::command]
pub fn get_skill(db: tauri::State<'_, DbState>, id: String) -> Result<Skill, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let skill = conn
        .query_row(
            "SELECT id, name, description, prompt, tools, created_at, updated_at FROM skills WHERE id = ?1",
            [&id],
            |row| {
                Ok(Skill {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    prompt: row.get(3)?,
                    tools: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(skill)
}

#[tauri::command]
pub fn create_skill(
    db: tauri::State<'_, DbState>,
    id: String,
    name: String,
    description: String,
    prompt: String,
    tools: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Validate limits
    if description.len() > 500 {
        return Err("Description must be 500 characters or less".to_string());
    }
    if prompt.len() > 10240 {
        return Err("Prompt must be 10KB or less".to_string());
    }

    // Check skill count limit
    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM skills", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if count >= 100 {
        return Err("Maximum skill limit (100) reached".to_string());
    }

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO skills (id, name, description, prompt, tools, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        [&id, &name, &description, &prompt, &tools, &now, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_skill(
    db: tauri::State<'_, DbState>,
    id: String,
    name: String,
    description: String,
    prompt: String,
    tools: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Validate limits
    if description.len() > 500 {
        return Err("Description must be 500 characters or less".to_string());
    }
    if prompt.len() > 10240 {
        return Err("Prompt must be 10KB or less".to_string());
    }

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE skills SET name = ?1, description = ?2, prompt = ?3, tools = ?4, updated_at = ?5 WHERE id = ?6",
        [&name, &description, &prompt, &tools, &now, &id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_skill(db: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM skills WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn search_skills(
    db: tauri::State<'_, DbState>,
    query: String,
) -> Result<Vec<Skill>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, prompt, tools, created_at, updated_at
             FROM skills WHERE name LIKE ?1 OR description LIKE ?1 ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([&pattern], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                prompt: row.get(3)?,
                tools: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(skills)
}

// ============================================================================
// Recipe Model and Commands
// ============================================================================

/// Recipe model
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub steps: String, // JSON array
    pub variables: Option<String>, // JSON object
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Recipe execution model
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RecipeExecution {
    pub id: String,
    pub recipe_id: String,
    pub status: String,
    pub variables: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[tauri::command]
pub fn list_recipes(db: tauri::State<'_, DbState>) -> Result<Vec<Recipe>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, version, steps, variables, is_builtin, created_at, updated_at
             FROM recipes ORDER BY is_builtin, name",
        )
        .map_err(|e| e.to_string())?;

    let recipes = stmt
        .query_map([], |row| {
            Ok(Recipe {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                version: row.get(3)?,
                steps: row.get(4)?,
                variables: row.get(5)?,
                is_builtin: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(recipes)
}

#[tauri::command]
pub fn get_recipe(db: tauri::State<'_, DbState>, id: String) -> Result<Recipe, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let recipe = conn
        .query_row(
            "SELECT id, name, description, version, steps, variables, is_builtin, created_at, updated_at
             FROM recipes WHERE id = ?1",
            [&id],
            |row| {
                Ok(Recipe {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    version: row.get(3)?,
                    steps: row.get(4)?,
                    variables: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(recipe)
}

#[tauri::command]
pub fn create_recipe(
    db: tauri::State<'_, DbState>,
    id: String,
    name: String,
    description: Option<String>,
    version: String,
    steps: String,
    variables: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO recipes (id, name, description, version, steps, variables, is_builtin, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
        [&id, &name, &description.unwrap_or_default(), &version, &steps, &variables.unwrap_or_default(), &now, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_recipe(
    db: tauri::State<'_, DbState>,
    id: String,
    name: String,
    description: Option<String>,
    version: String,
    steps: String,
    variables: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE recipes SET name = ?1, description = ?2, version = ?3, steps = ?4, variables = ?5, updated_at = ?6 WHERE id = ?7",
        [&name, &description.unwrap_or_default(), &version, &steps, &variables.unwrap_or_default(), &now, &id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_recipe(db: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Check if builtin
    let is_builtin: bool = conn
        .query_row("SELECT is_builtin FROM recipes WHERE id = ?1", [&id], |row| {
            Ok(row.get::<_, i32>(0)? != 0)
        })
        .map_err(|e| e.to_string())?;

    if is_builtin {
        return Err("Cannot delete built-in recipes".to_string());
    }

    conn.execute("DELETE FROM recipes WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn create_recipe_execution(
    db: tauri::State<'_, DbState>,
    id: String,
    recipe_id: String,
    variables: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO recipe_executions (id, recipe_id, status, variables, started_at)
         VALUES (?1, ?2, 'running', ?3, ?4)",
        [&id, &recipe_id, &variables.unwrap_or_default(), &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_recipe_execution(
    db: tauri::State<'_, DbState>,
    id: String,
    status: String,
    result: Option<String>,
    error: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE recipe_executions SET status = ?1, result = ?2, error = ?3, completed_at = ?4 WHERE id = ?5",
        [&status, &result.unwrap_or_default(), &error.unwrap_or_default(), &now, &id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn list_recipe_executions(
    db: tauri::State<'_, DbState>,
    recipe_id: Option<String>,
) -> Result<Vec<RecipeExecution>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let sql = match &recipe_id {
        Some(_) => "SELECT id, recipe_id, status, variables, result, error, started_at, completed_at
                     FROM recipe_executions WHERE recipe_id = ?1 ORDER BY started_at DESC",
        None => "SELECT id, recipe_id, status, variables, result, error, started_at, completed_at
                 FROM recipe_executions ORDER BY started_at DESC",
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let map_row = |row: &rusqlite::Row| {
        Ok(RecipeExecution {
            id: row.get(0)?,
            recipe_id: row.get(1)?,
            status: row.get(2)?,
            variables: row.get(3)?,
            result: row.get(4)?,
            error: row.get(5)?,
            started_at: row.get(6)?,
            completed_at: row.get(7)?,
        })
    };

    let executions = match &recipe_id {
        Some(rid) => stmt.query_map([&rid], map_row),
        None => stmt.query_map([], map_row),
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(executions)
}

// ============================================================================
// Sub-agent Model and Commands (v0.3)
// ============================================================================

/// Sub-agent model
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SubAgent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub system_prompt: Option<String>,
    pub tools: String,
    pub config: String,
    pub status: String,
    pub task: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[tauri::command]
pub fn list_sub_agents(db: tauri::State<'_, DbState>) -> Result<Vec<SubAgent>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, role, system_prompt, tools, config, status, task, result, error, created_at, completed_at
             FROM sub_agents ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let agents = stmt
        .query_map([], |row| {
            Ok(SubAgent {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                system_prompt: row.get(3)?,
                tools: row.get(4)?,
                config: row.get(5)?,
                status: row.get(6)?,
                task: row.get(7)?,
                result: row.get(8)?,
                error: row.get(9)?,
                created_at: row.get(10)?,
                completed_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(agents)
}

#[tauri::command]
pub fn create_sub_agent(
    db: tauri::State<'_, DbState>,
    id: String,
    name: String,
    agent_type: String,
    system_prompt: Option<String>,
    tools: String,
    config: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO sub_agents (id, name, role, system_prompt, tools, config, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'idle', ?7)",
        [&id, &name, &agent_type, &system_prompt.unwrap_or_default(), &tools, &config, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_sub_agent(
    db: tauri::State<'_, DbState>,
    id: String,
    name: Option<String>,
    system_prompt: Option<String>,
    tools: Option<String>,
    config: Option<String>,
    status: Option<String>,
    task: Option<String>,
    result: Option<String>,
    error: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Build dynamic update query
    let mut updates = Vec::new();
    let mut params: Vec<String> = Vec::new();

    if let Some(n) = name {
        updates.push("name = ?");
        params.push(n);
    }
    if let Some(sp) = system_prompt {
        updates.push("system_prompt = ?");
        params.push(sp);
    }
    if let Some(t) = tools {
        updates.push("tools = ?");
        params.push(t);
    }
    if let Some(c) = config {
        updates.push("config = ?");
        params.push(c);
    }
    if let Some(s) = status {
        updates.push("status = ?");
        params.push(s);
    }
    if let Some(t) = task {
        updates.push("task = ?");
        params.push(t);
    }
    if let Some(r) = result {
        updates.push("result = ?");
        params.push(r);
    }
    if let Some(e) = error {
        updates.push("error = ?");
        params.push(e);
    }

    if updates.is_empty() {
        return Ok(());
    }

    params.push(id.clone());
    let sql = format!("UPDATE sub_agents SET {} WHERE id = ?", updates.join(", "));

    conn.execute(&sql, rusqlite::params_from_iter(params.iter()))
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_sub_agent(db: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM sub_agents WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn assign_sub_agent_task(
    db: tauri::State<'_, DbState>,
    id: String,
    task: String,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE sub_agents SET status = 'running', task = ?1, error = NULL, result = NULL WHERE id = ?2 AND status = 'idle'",
        [&task, &id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Cron Job Model and Commands (v0.3)
// ============================================================================

/// Cron job model
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub job_type: String,
    pub config: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Job execution model
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct JobExecution {
    pub id: String,
    pub job_id: String,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[tauri::command]
pub fn list_cron_jobs(db: tauri::State<'_, DbState>) -> Result<Vec<CronJob>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, schedule, job_type, config, enabled, last_run, next_run, created_at, updated_at
             FROM cron_jobs ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let jobs = stmt
        .query_map([], |row| {
            Ok(CronJob {
                id: row.get(0)?,
                name: row.get(1)?,
                schedule: row.get(2)?,
                job_type: row.get(3)?,
                config: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                last_run: row.get(6)?,
                next_run: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(jobs)
}

#[tauri::command]
pub fn create_cron_job(
    db: tauri::State<'_, DbState>,
    id: String,
    name: String,
    schedule: String,
    job_type: String,
    config: String,
    enabled: i32,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO cron_jobs (id, name, schedule, job_type, config, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        [&id, &name, &schedule, &job_type, &config, &enabled.to_string(), &now, &now],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_cron_job(
    db: tauri::State<'_, DbState>,
    id: String,
    name: Option<String>,
    schedule: Option<String>,
    config: Option<String>,
    enabled: Option<i32>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    let mut updates = vec!["updated_at = ?"];
    let mut params: Vec<String> = vec![now.clone()];

    if let Some(n) = name {
        updates.push("name = ?");
        params.push(n);
    }
    if let Some(s) = schedule {
        updates.push("schedule = ?");
        params.push(s);
    }
    if let Some(c) = config {
        updates.push("config = ?");
        params.push(c);
    }
    if let Some(e) = enabled {
        updates.push("enabled = ?");
        params.push(e.to_string());
    }

    params.push(id.clone());
    let sql = format!("UPDATE cron_jobs SET {} WHERE id = ?", updates.join(", "));

    conn.execute(&sql, rusqlite::params_from_iter(params.iter()))
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_cron_job(db: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM cron_jobs WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn run_cron_job_now(db: tauri::State<'_, DbState>, id: String) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let execution_id = format!("exec-{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    // Create execution record
    conn.execute(
        "INSERT INTO job_executions (id, job_id, status, started_at)
         VALUES (?1, ?2, 'running', ?3)",
        [&execution_id, &id, &now],
    )
    .map_err(|e| e.to_string())?;

    // Update job's last_run
    conn.execute(
        "UPDATE cron_jobs SET last_run = ?1 WHERE id = ?2",
        [&now, &id],
    )
    .map_err(|e| e.to_string())?;

    // TODO: Actually execute the job (would need async execution)
    // For now, just mark as completed
    let completed_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE job_executions SET status = 'completed', result = 'Job executed', completed_at = ?1 WHERE id = ?2",
        [&completed_at, &execution_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(execution_id)
}

#[tauri::command]
pub fn list_job_executions(
    db: tauri::State<'_, DbState>,
    job_id: Option<String>,
) -> Result<Vec<JobExecution>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let sql = match &job_id {
        Some(_) => "SELECT id, job_id, status, result, error, started_at, completed_at
                     FROM job_executions WHERE job_id = ?1 ORDER BY started_at DESC LIMIT 100",
        None => "SELECT id, job_id, status, result, error, started_at, completed_at
                 FROM job_executions ORDER BY started_at DESC LIMIT 100",
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let map_row = |row: &rusqlite::Row| {
        Ok(JobExecution {
            id: row.get(0)?,
            job_id: row.get(1)?,
            status: row.get(2)?,
            result: row.get(3)?,
            error: row.get(4)?,
            started_at: row.get(5)?,
            completed_at: row.get(6)?,
        })
    };

    let executions = match &job_id {
        Some(jid) => stmt.query_map([&jid], map_row),
        None => stmt.query_map([], map_row),
    }
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(executions)
}
