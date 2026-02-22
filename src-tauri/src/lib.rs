// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod db;
mod sidecar;
mod voice;
mod plugins;
mod collaboration;
mod scheduler;
mod marketplace;
mod integration;
mod security;

use scheduler::JobScheduler;
use security::CredentialManager;
use plugins::PluginExecutor;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

/// Message structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Chat request
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub provider: Option<String>,
}

/// Chat response
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub error: Option<String>,
}

/// Folder permission
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderPermission {
    pub id: String,
    pub path: String,
    pub level: String, // "read" or "readwrite"
}

/// Simple greeting command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Get app version
#[tauri::command]
fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Validate folder path
#[tauri::command]
fn validate_folder_path(path: &str) -> Result<bool, String> {
    let path_buf = PathBuf::from(path);
    
    if !path_buf.exists() {
        return Err("Path does not exist".to_string());
    }
    
    if !path_buf.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    Ok(true)
}

/// Check if folder is accessible
#[tauri::command]
fn check_folder_access(path: &str, permissions: Vec<FolderPermission>) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    
    for perm in permissions {
        let perm_path = PathBuf::from(&perm.path);
        if path_buf.starts_with(&perm_path) {
            return Ok(perm.level);
        }
    }
    
    Err("No permission to access this folder".to_string())
}

/// Read file content
#[tauri::command]
fn read_file_content(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

/// Write file content
#[tauri::command]
fn write_file_content(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

/// List directory contents
#[tauri::command]
fn list_directory(path: &str) -> Result<Vec<String>, String> {
    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut result = Vec::new();
    for entry in entries.flatten() {
        if let Ok(name) = entry.file_name().into_string() {
            let file_type = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                "dir"
            } else {
                "file"
            };
            result.push(format!("{}:{}", name, file_type));
        }
    }
    
    Ok(result)
}

// ============================================================================
// Scheduler Commands (JobScheduler)
// ============================================================================

/// Scheduler status response
#[derive(Debug, Serialize, Deserialize)]
pub struct SchedulerStatus {
    pub running: bool,
    pub job_count: usize,
    pub running_count: usize,
}

/// Start the job scheduler
#[tauri::command]
async fn scheduler_start(
    scheduler: tauri::State<'_, Arc<tokio::sync::Mutex<JobScheduler>>>,
) -> Result<(), String> {
    let scheduler = scheduler.lock().await;
    scheduler.start().await
}

/// Stop the job scheduler
#[tauri::command]
async fn scheduler_stop(
    scheduler: tauri::State<'_, Arc<tokio::sync::Mutex<JobScheduler>>>,
) -> Result<(), String> {
    let scheduler = scheduler.lock().await;
    scheduler.stop().await;
    Ok(())
}

/// Get scheduler status
#[tauri::command]
async fn scheduler_status(
    scheduler: tauri::State<'_, Arc<tokio::sync::Mutex<JobScheduler>>>,
) -> Result<SchedulerStatus, String> {
    let scheduler = scheduler.lock().await;
    let running = scheduler.is_running().await;
    let job_count = scheduler.get_jobs().await.len();
    let running_count = scheduler.running_count().await;

    Ok(SchedulerStatus {
        running,
        job_count,
        running_count,
    })
}

/// Execute a scheduled job immediately
#[tauri::command]
async fn scheduler_execute_job(
    scheduler: tauri::State<'_, Arc<tokio::sync::Mutex<JobScheduler>>>,
    job_id: String,
) -> Result<String, String> {
    let scheduler = scheduler.lock().await;
    scheduler.execute_now(&job_id).await
}

/// Cancel a running job execution
#[tauri::command]
async fn scheduler_cancel_execution(
    scheduler: tauri::State<'_, Arc<tokio::sync::Mutex<JobScheduler>>>,
    execution_id: String,
) -> Result<bool, String> {
    let scheduler = scheduler.lock().await;
    Ok(scheduler.cancel_execution(&execution_id).await)
}

// ============================================================================
// Marketplace Commands
// ============================================================================

/// List marketplace items
#[tauri::command]
async fn marketplace_list_items(
    filters: Option<marketplace::MarketplaceFilters>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<marketplace::MarketplaceItem>, String> {
    let store = marketplace::MarketplaceStore::default_marketplace();
    let filters = filters.unwrap_or_default();
    store.list_items(&filters, page.unwrap_or(1), page_size.unwrap_or(20)).await
}

/// Get marketplace item details
#[tauri::command]
async fn marketplace_get_item(
    item_id: String,
) -> Result<marketplace::MarketplaceItem, String> {
    let store = marketplace::MarketplaceStore::default_marketplace();
    store.get_item(&item_id).await
}

/// Search marketplace items
#[tauri::command]
async fn marketplace_search_items(
    query: String,
    filters: Option<marketplace::MarketplaceFilters>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Vec<marketplace::MarketplaceItem>, String> {
    let store = marketplace::MarketplaceStore::default_marketplace();
    let filters = filters.unwrap_or_default();
    store.search_items(&query, &filters, page.unwrap_or(1), page_size.unwrap_or(20)).await
}

/// Get marketplace categories
#[tauri::command]
async fn marketplace_get_categories() -> Result<Vec<marketplace::MarketplaceCategory>, String> {
    let store = marketplace::MarketplaceStore::default_marketplace();
    store.get_categories().await
}

/// Install marketplace item
#[tauri::command]
async fn marketplace_install_item(
    item_id: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let store = marketplace::MarketplaceStore::default_marketplace();
    let item = store.get_item(&item_id).await?;

    let install_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let marketplace_dir = install_dir.join("marketplace");
    let mut installer = marketplace::MarketplaceInstaller::new(marketplace_dir)?;

    installer.install(&item).await
}

/// Uninstall marketplace item
#[tauri::command]
async fn marketplace_uninstall_item(
    item_id: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let install_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let marketplace_dir = install_dir.join("marketplace");
    let mut installer = marketplace::MarketplaceInstaller::new(marketplace_dir)?;

    installer.uninstall(&item_id).await
}

/// Check for marketplace updates
#[tauri::command]
async fn marketplace_check_updates(
    app_handle: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let install_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let marketplace_dir = install_dir.join("marketplace");
    let installer = marketplace::MarketplaceInstaller::new(marketplace_dir)?;

    installer.check_updates().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize database
            let db_state = db::DbState::new(app.handle())
                .expect("Failed to initialize database");
            let db_path = db_state.db_path.clone();

            app.manage(db_state);

            // Initialize sidecar state
            let sidecar_state = std::sync::Mutex::new(sidecar::SidecarState::new());
            app.manage(sidecar_state);

            // Initialize credential manager
            let credential_manager = CredentialManager::default()
                .expect("Failed to initialize credential manager");
            app.manage(std::sync::Mutex::new(credential_manager));

            // Initialize plugin executor
            let plugin_executor = PluginExecutor::new();
            app.manage(std::sync::Mutex::new(plugin_executor));

            // Initialize job scheduler
            let scheduler_config = scheduler::SchedulerConfig {
                check_interval_secs: 60,
                db_path: db_path.clone(),
                max_concurrent_jobs: 5,
            };
            let job_scheduler = Arc::new(tokio::sync::Mutex::new(JobScheduler::new(scheduler_config)));
            app.manage(job_scheduler);

            // Load jobs from database and start scheduler
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                // Load jobs from DB
                if let Ok(jobs) = db::load_scheduled_jobs(&app_handle) {
                    let scheduler = app_handle.try_state::<Arc<tokio::sync::Mutex<JobScheduler>>>();
                    if let Some(scheduler) = scheduler {
                        let scheduler = scheduler.lock().await;
                        if let Err(e) = scheduler.load_jobs(jobs).await {
                            tracing::error!("Failed to load scheduled jobs: {}", e);
                        } else {
                            scheduler.refresh_schedule().await;
                        }
                    }
                }

                // Start the scheduler
                let scheduler = app_handle.try_state::<Arc<tokio::sync::Mutex<JobScheduler>>>();
                if let Some(scheduler) = scheduler {
                    let scheduler = scheduler.lock().await;
                    if let Err(e) = scheduler.start().await {
                        tracing::error!("Failed to start scheduler: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_version,
            validate_folder_path,
            check_folder_access,
            read_file_content,
            write_file_content,
            list_directory,
            sidecar::init_agent,
            sidecar::agent_chat,
            sidecar::get_tools,
            sidecar::configure_providers,
            sidecar::shutdown_agent,
            sidecar::execute_recipe,
            sidecar::execute_skill,
            sidecar::execute_prompt,
            // Database commands
            db::load_conversations,
            db::save_conversation,
            db::delete_conversation,
            db::load_messages,
            db::save_message,
            db::load_folder_permissions,
            db::add_folder_permission,
            db::remove_folder_permission,
            db::update_folder_permission,
            // Skill commands
            db::list_skills,
            db::get_skill,
            db::create_skill,
            db::update_skill,
            db::delete_skill,
            db::search_skills,
            // Recipe commands
            db::list_recipes,
            db::get_recipe,
            db::create_recipe,
            db::update_recipe,
            db::delete_recipe,
            db::create_recipe_execution,
            db::update_recipe_execution,
            db::list_recipe_executions,
            // Sub-agent commands (v0.3)
            db::list_sub_agents,
            db::create_sub_agent,
            db::update_sub_agent,
            db::delete_sub_agent,
            db::assign_sub_agent_task,
            // Cron job commands (v0.3)
            db::list_cron_jobs,
            db::create_cron_job,
            db::update_cron_job,
            db::delete_cron_job,
            db::run_cron_job_now,
            db::list_job_executions,
            // Scheduler commands
            scheduler_start,
            scheduler_stop,
            scheduler_status,
            scheduler_execute_job,
            scheduler_cancel_execution,
            // Marketplace commands
            marketplace_list_items,
            marketplace_get_item,
            marketplace_search_items,
            marketplace_get_categories,
            marketplace_install_item,
            marketplace_uninstall_item,
            marketplace_check_updates,
            // Plugin commands (v0.4)
            db::list_plugins,
            db::get_plugin,
            db::install_plugin,
            db::uninstall_plugin,
            db::enable_plugin,
            db::disable_plugin,
            // Template commands (v0.4)
            db::list_templates,
            db::get_template,
            db::create_template,
            db::update_template,
            db::delete_template,
            db::search_templates,
            // Template import/export commands (v0.5)
            collaboration::template_commands::export_template,
            collaboration::template_commands::export_all_templates,
            collaboration::template_commands::import_template,
            collaboration::template_commands::import_templates,
            collaboration::template_commands::validate_template_data,
            // Template versioning commands (v0.5)
            collaboration::template_commands::get_template_versions,
            collaboration::template_commands::create_template_version,
            collaboration::template_commands::rollback_template,
            // Template sharing commands (v0.5)
            collaboration::template_commands::share_template_to_team,
            collaboration::template_commands::get_team_templates,
            collaboration::template_commands::revoke_template_access,
            // Workflow commands (v0.5)
            collaboration::list_workflows,
            collaboration::get_workflow,
            collaboration::create_workflow,
            collaboration::update_workflow,
            collaboration::delete_workflow,
            // Voice settings commands (v0.4)
            db::get_voice_settings,
            db::update_voice_settings,
            // Voice commands (v0.4)
            voice::stt::init_stt,
            voice::stt::transcribe,
            voice::stt::get_available_models,
            voice::tts::init_tts,
            voice::tts::synthesize,
            voice::tts::get_available_voices,
            // Voice command parsing (v0.5)
            voice::commands::parse_voice_command,
            voice::commands::detect_voice_language,
            voice::commands::validate_voice_command,
            voice::commands::get_voice_command_patterns,
            // Voice conversation commands (v0.5)
            sidecar::execute_voice_command,
            sidecar::start_voice_conversation,
            sidecar::continue_voice_conversation,
            sidecar::end_voice_conversation,
            // Integration commands (v0.4)
            integration::test_database_connection,
            integration::get_database_connection_string,
            integration::validate_git_repository,
            integration::get_git_status,
            integration::get_git_current_commit,
            integration::test_cloud_connection,
            integration::list_cloud_objects,
            integration::get_cloud_endpoint,
            // Security commands (v0.5)
            security::credentials_set_password,
            security::credentials_get_password,
            security::credentials_delete_password,
            security::run_migration,
            // Cloud storage commands (v0.5)
            db::list_cloud_storages,
            db::create_cloud_storage,
            db::delete_cloud_storage,
            // Git repository commands (v0.5)
            db::list_git_repositories,
            db::create_git_repository,
            db::delete_git_repository,
            // Plugin execution commands (v0.5)
            plugins::plugin_execute,
            plugins::plugin_get_resource_usage,
            plugins::plugin_send_message,
            plugins::plugin_get_messages,
            plugins::plugin_stop,
            plugins::plugin_restart,
            plugins::plugin_list_running
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! You've been greeted from Rust!");
    }

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert_eq!(version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_message_serialization() {
        let message = Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            messages: vec![Message {
                role: "user".to_string(),
                content: "Test".to_string(),
            }],
            provider: Some("openai".to_string()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("openai"));
    }

    #[test]
    fn test_folder_permission() {
        let perm = FolderPermission {
            id: "1".to_string(),
            path: "/test/path".to_string(),
            level: "read".to_string(),
        };
        assert_eq!(perm.id, "1");
        assert_eq!(perm.path, "/test/path");
        assert_eq!(perm.level, "read");
    }
}
