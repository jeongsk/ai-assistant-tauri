//! Job runner for scheduled tasks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::path::PathBuf;

/// Job type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobType {
    Skill,
    Recipe,
    Prompt,
    System,
}

/// Job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// For skill/recipe: the ID. For prompt: the prompt text. For system: the task name.
    pub target: String,
    /// Additional parameters
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

/// Scheduled job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub job_type: JobType,
    pub config: JobConfig,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Job execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    pub id: String,
    pub job_id: String,
    pub status: ExecutionStatus,
    pub result: Option<String>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// System tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemTask {
    CleanupOldMessages,
    VacuumDatabase,
    SyncSettings,
}

impl SystemTask {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cleanup_old_messages" => Some(Self::CleanupOldMessages),
            "vacuum_database" => Some(Self::VacuumDatabase),
            "sync_settings" => Some(Self::SyncSettings),
            _ => None,
        }
    }
}

/// Execution context for jobs
#[derive(Clone)]
pub struct ExecutionContext {
    /// Database path for system tasks
    pub db_path: PathBuf,
    /// Agent runtime endpoint for skill/recipe execution
    pub agent_endpoint: Option<String>,
    /// Maximum execution time in seconds
    pub timeout_secs: u64,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("./app.db"),
            agent_endpoint: None,
            timeout_secs: 300, // 5 minutes default
        }
    }
}

/// Result of a job execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub status: ExecutionStatus,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Job executor - handles actual execution of different job types
pub struct JobExecutor {
    context: ExecutionContext,
    running_jobs: Arc<Mutex<HashMap<String, tokio::task::JoinHandle<(String, ExecutionResult)>>>>,
    semaphore: Arc<Semaphore>,
}

impl JobExecutor {
    /// Create a new job executor
    pub fn new(context: ExecutionContext) -> Self {
        // Limit concurrent jobs to 5
        let semaphore = Arc::new(Semaphore::new(5));

        Self {
            context,
            running_jobs: Arc::new(Mutex::new(HashMap::new())),
            semaphore,
        }
    }

    /// Execute a job asynchronously
    pub async fn execute_job(&self, job: ScheduledJob) -> String {
        let execution_id = format!("exec-{}", uuid::Uuid::new_v4());
        let job_id = job.id.clone();
        let execution_id_clone = execution_id.clone();
        let context = self.context.clone();
        let semaphore = self.semaphore.clone();

        // Spawn the job execution task
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let result = match job.job_type {
                JobType::System => Self::execute_system_task(&job, &context).await,
                JobType::Skill => Self::execute_skill(&job, &context).await,
                JobType::Recipe => Self::execute_recipe(&job, &context).await,
                JobType::Prompt => Self::execute_prompt(&job, &context).await,
            };

            // Store the result (in a real implementation, this would update the database)
            tracing::info!(
                "Job {} execution {} completed with status: {:?}",
                job_id,
                execution_id_clone,
                result.status
            );

            (execution_id_clone, result)
        });

        // Store the handle
        let mut running = self.running_jobs.lock().await;
        running.insert(execution_id.clone(), handle);

        execution_id
    }

    /// Execute a system task
    async fn execute_system_task(job: &ScheduledJob, context: &ExecutionContext) -> ExecutionResult {
        let task_name = &job.config.target;

        let system_task = match SystemTask::from_str(task_name) {
            Some(task) => task,
            None => {
                return ExecutionResult {
                    status: ExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("Unknown system task: {}", task_name)),
                };
            }
        };

        match system_task {
            SystemTask::CleanupOldMessages => {
                Self::cleanup_old_messages(context, job).await
            }
            SystemTask::VacuumDatabase => {
                Self::vacuum_database(context).await
            }
            SystemTask::SyncSettings => {
                // Settings sync is a placeholder for now
                ExecutionResult {
                    status: ExecutionStatus::Completed,
                    output: Some("Settings synced".to_string()),
                    error: None,
                }
            }
        }
    }

    /// Execute a skill job
    async fn execute_skill(job: &ScheduledJob, _context: &ExecutionContext) -> ExecutionResult {
        let skill_id = &job.config.target;

        // In a real implementation, this would call the agent runtime
        // For now, we'll simulate the execution
        tracing::info!("Executing skill: {}", skill_id);

        // Placeholder - would make JSON-RPC call to agent runtime
        ExecutionResult {
            status: ExecutionStatus::Completed,
            output: Some(format!("Skill '{}' executed successfully", skill_id)),
            error: None,
        }
    }

    /// Execute a recipe job
    async fn execute_recipe(job: &ScheduledJob, _context: &ExecutionContext) -> ExecutionResult {
        let recipe_id = &job.config.target;

        // In a real implementation, this would call the recipe executor
        tracing::info!("Executing recipe: {}", recipe_id);

        // Placeholder - would execute recipe steps
        ExecutionResult {
            status: ExecutionStatus::Completed,
            output: Some(format!("Recipe '{}' executed successfully", recipe_id)),
            error: None,
        }
    }

    /// Execute a prompt job
    async fn execute_prompt(job: &ScheduledJob, _context: &ExecutionContext) -> ExecutionResult {
        let prompt = &job.config.target;

        // In a real implementation, this would send the prompt to the LLM
        tracing::info!("Executing prompt job: {}", &job.name);

        // Placeholder - would send to agent runtime
        ExecutionResult {
            status: ExecutionStatus::Completed,
            output: Some(format!("Prompt executed: {}", prompt)),
            error: None,
        }
    }

    /// Cleanup old messages (system task)
    async fn cleanup_old_messages(context: &ExecutionContext, job: &ScheduledJob) -> ExecutionResult {
        // Get the retention period from params (default 30 days)
        let retention_days = job.config.params
            .get("retention_days")
            .and_then(|v| v.as_u64())
            .unwrap_or(30) as i64;

        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);

        // Open the database
        let conn = rusqlite::Connection::open(&context.db_path);

        match conn {
            Ok(conn) => {
                let deleted = conn.execute(
                    "DELETE FROM messages WHERE created_at < ?",
                    [&cutoff_date.to_rfc3339()],
                );

                match deleted {
                    Ok(count) => {
                        ExecutionResult {
                            status: ExecutionStatus::Completed,
                            output: Some(format!("Deleted {} old messages", count)),
                            error: None,
                        }
                    }
                    Err(e) => {
                        ExecutionResult {
                            status: ExecutionStatus::Failed,
                            output: None,
                            error: Some(format!("Failed to delete messages: {}", e)),
                        }
                    }
                }
            }
            Err(e) => {
                ExecutionResult {
                    status: ExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("Failed to open database: {}", e)),
                }
            }
        }
    }

    /// Vacuum the database (system task)
    async fn vacuum_database(context: &ExecutionContext) -> ExecutionResult {
        let conn = rusqlite::Connection::open(&context.db_path);

        match conn {
            Ok(conn) => {
                match conn.execute("VACUUM", []) {
                    Ok(_) => {
                        ExecutionResult {
                            status: ExecutionStatus::Completed,
                            output: Some("Database vacuumed successfully".to_string()),
                            error: None,
                        }
                    }
                    Err(e) => {
                        ExecutionResult {
                            status: ExecutionStatus::Failed,
                            output: None,
                            error: Some(format!("Failed to vacuum database: {}", e)),
                        }
                    }
                }
            }
            Err(e) => {
                ExecutionResult {
                    status: ExecutionStatus::Failed,
                    output: None,
                    error: Some(format!("Failed to open database: {}", e)),
                }
            }
        }
    }

    /// Cancel a running job
    pub async fn cancel_job(&self, execution_id: &str) -> bool {
        let mut running = self.running_jobs.lock().await;
        if let Some(handle) = running.remove(execution_id) {
            handle.abort();
            true
        } else {
            false
        }
    }

    /// Get count of currently running jobs
    pub async fn running_count(&self) -> usize {
        self.running_jobs.lock().await.len()
    }

    /// Clean up completed jobs
    pub async fn cleanup_completed(&self) {
        let mut running = self.running_jobs.lock().await;
        let mut to_remove = Vec::new();

        for (id, handle) in running.iter() {
            if handle.is_finished() {
                to_remove.push(id.clone());
            }
        }

        for id in to_remove {
            running.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_task_from_str() {
        assert!(matches!(
            SystemTask::from_str("cleanup_old_messages"),
            Some(SystemTask::CleanupOldMessages)
        ));
        assert!(matches!(
            SystemTask::from_str("vacuum_database"),
            Some(SystemTask::VacuumDatabase)
        ));
        assert!(matches!(
            SystemTask::from_str("sync_settings"),
            Some(SystemTask::SyncSettings)
        ));
        assert!(SystemTask::from_str("unknown_task").is_none());
    }

    #[test]
    fn test_execution_context_default() {
        let ctx = ExecutionContext::default();
        assert_eq!(ctx.timeout_secs, 300);
    }
}
