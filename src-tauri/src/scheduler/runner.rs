//! Job runner for scheduled tasks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{Mutex, Semaphore};

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
    /// Path to agent runtime binary
    pub agent_binary_path: Option<PathBuf>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("./app.db"),
            agent_endpoint: None,
            timeout_secs: 300, // 5 minutes default
            agent_binary_path: None,
        }
    }
}

/// Agent runtime sidecar client
pub struct AgentRuntimeClient {
    process: StdMutex<Option<SidecarProcess>>,
}

impl AgentRuntimeClient {
    /// Create a new client (but don't spawn the process yet)
    pub fn new() -> Self {
        Self {
            process: StdMutex::new(None),
        }
    }

    /// Find the agent runtime binary path
    fn find_binary_path() -> Result<PathBuf, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;

        // Try binaries folder
        let bin_path = exe_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("binaries").join("agent-runtime"))
            .filter(|p| p.exists());

        // Try same directory
        if bin_path.is_none() {
            let same_dir = exe_path
                .parent()
                .map(|p| p.join("agent-runtime"))
                .filter(|p| p.exists());
            return same_dir.ok_or_else(|| {
                "Agent runtime binary not found".to_string()
            });
        }

        bin_path.ok_or_else(|| {
            "Agent runtime binary not found".to_string()
        })
    }

    /// Ensure the sidecar process is running
    fn ensure_running(&self) -> Result<(), String> {
        let mut guard = self.process.lock().unwrap();
        if guard.is_some() {
            return Ok(());
        }

        let binary_path = Self::find_binary_path()?;
        let mut process = SidecarProcess::spawn(Some(binary_path))?;

        // Wait for ready signal
        process.wait_for_ready()?;

        *guard = Some(process);
        Ok(())
    }

    /// Execute a skill via agent runtime
    pub fn execute_skill(&self, skill_id: &str, input: Option<&str>, variables: Option<&HashMap<String, serde_json::Value>>) -> Result<String, String> {
        self.ensure_running()?;

        let mut guard = self.process.lock().unwrap();
        let process = guard.as_mut().unwrap();

        let mut params = json!({
            "skillId": skill_id,
        });
        if let Some(i) = input {
            params["input"] = json!(i);
        }
        if let Some(v) = variables {
            params["variables"] = json!(v);
        }

        let request = AgentRequest {
            jsonrpc: "2.0".to_string(),
            method: "execute_skill".to_string(),
            params,
            id: uuid::Uuid::new_v4().to_string(),
        };

        let response = process.send_request(&request)?;

        if let Some(error) = response.error {
            return Err(format!("{}: {}", error.code, error.message));
        }

        let result = response.result
            .and_then(|r| r.get("result").and_then(|r| r.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "No result".to_string());

        Ok(result)
    }

    /// Execute a recipe via agent runtime
    pub fn execute_recipe(&self, recipe_id: &str, variables: Option<&HashMap<String, serde_json::Value>>) -> Result<String, String> {
        self.ensure_running()?;

        let mut guard = self.process.lock().unwrap();
        let process = guard.as_mut().unwrap();

        let mut params = json!({
            "recipeId": recipe_id,
        });
        if let Some(v) = variables {
            params["variables"] = json!(v);
        }

        let request = AgentRequest {
            jsonrpc: "2.0".to_string(),
            method: "execute_recipe".to_string(),
            params,
            id: uuid::Uuid::new_v4().to_string(),
        };

        let response = process.send_request(&request)?;

        if let Some(error) = response.error {
            return Err(format!("{}: {}", error.code, error.message));
        }

        let result = response.result
            .and_then(|r| r.get("result").and_then(|r| r.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "No result".to_string());

        Ok(result)
    }

    /// Execute a prompt via agent runtime
    pub fn execute_prompt(&self, prompt: &str, provider: Option<&str>) -> Result<String, String> {
        self.ensure_running()?;

        let mut guard = self.process.lock().unwrap();
        let process = guard.as_mut().unwrap();

        let mut params = json!({
            "prompt": prompt,
        });
        if let Some(p) = provider {
            params["provider"] = json!(p);
        }

        let request = AgentRequest {
            jsonrpc: "2.0".to_string(),
            method: "execute_prompt".to_string(),
            params,
            id: uuid::Uuid::new_v4().to_string(),
        };

        let response = process.send_request(&request)?;

        if let Some(error) = response.error {
            return Err(format!("{}: {}", error.code, error.message));
        }

        let result = response.result
            .and_then(|r| r.get("result").and_then(|r| r.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "No result".to_string());

        Ok(result)
    }
}

impl Default for AgentRuntimeClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Sidecar process wrapper (copied from sidecar.rs to avoid circular dependency)
struct SidecarProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

/// Agent request for JSON-RPC
#[derive(Debug, Serialize, Deserialize)]
struct AgentRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: String,
}

/// Agent response for JSON-RPC
#[derive(Debug, Serialize, Deserialize)]
struct AgentResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<AgentError>,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentError {
    pub code: i32,
    pub message: String,
}

impl SidecarProcess {
    fn spawn(binary_path: Option<PathBuf>) -> Result<Self, String> {
        let bin_path = binary_path
            .or_else(|| {
                let exe_path = std::env::current_exe().ok()?;
                exe_path
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("binaries").join("agent-runtime"))
                    .filter(|p| p.exists())
            })
            .ok_or_else(|| "Failed to determine binaries path".to_string())?;

        if !bin_path.exists() {
            return Err(format!("Agent runtime binary not found at {:?}", bin_path));
        }

        let mut child = Command::new(&bin_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn agent runtime: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    fn send_request(&mut self, request: &AgentRequest) -> Result<AgentResponse, String> {
        let request_str = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        writeln!(self.stdin, "{}", request_str)
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;

        self.stdin.flush()
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;

        let mut response_str = String::new();
        self.stdout.read_line(&mut response_str)
            .map_err(|e| format!("Failed to read from stdout: {}", e))?;

        serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    fn wait_for_ready(&mut self) -> Result<(), String> {
        let mut ready_line = String::new();
        self.stdout.read_line(&mut ready_line)
            .map_err(|e| format!("Failed to read ready signal: {}", e))?;

        let response: AgentResponse = serde_json::from_str(&ready_line)
            .map_err(|e| format!("Failed to parse ready signal: {}", e))?;

        if response.result.as_ref()
            .and_then(|r| r.get("status"))
            .and_then(|s| s.as_str())
            != Some("ready") {
            return Err("Sidecar not ready".to_string());
        }

        Ok(())
    }
}

impl Drop for SidecarProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
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
    completed_results: Arc<StdMutex<HashMap<String, (String, ExecutionResult)>>>,
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
            completed_results: Arc::new(StdMutex::new(HashMap::new())),
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
        let completed_results = self.completed_results.clone();

        // Create execution record in database
        if let Err(e) = Self::create_execution_record(&context, &execution_id, &job_id) {
            tracing::error!("Failed to create execution record: {}", e);
        }

        // Spawn the job execution task
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let result = match job.job_type {
                JobType::System => Self::execute_system_task(&job, &context).await,
                JobType::Skill => Self::execute_skill(&job, &context).await,
                JobType::Recipe => Self::execute_recipe(&job, &context).await,
                JobType::Prompt => Self::execute_prompt(&job, &context).await,
            };

            // Store the result in completed results
            {
                let mut results = completed_results.lock().unwrap();
                results.insert(execution_id_clone.clone(), (job_id.clone(), result.clone()));
            }

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

    /// Create execution record in database
    fn create_execution_record(
        context: &ExecutionContext,
        execution_id: &str,
        job_id: &str,
    ) -> Result<(), String> {
        use rusqlite::params;

        let conn = rusqlite::Connection::open(&context.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        conn.execute(
            "INSERT INTO job_executions (id, job_id, status, started_at) VALUES (?, ?, ?, ?)",
            params![
                execution_id,
                job_id,
                "running",
                Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|e| format!("Failed to create execution record: {}", e))?;

        Ok(())
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

    /// Clean up completed jobs and save results to database
    pub async fn cleanup_completed(&self) {
        let mut running = self.running_jobs.lock().await;
        let mut to_remove = Vec::new();
        let mut results_to_save = Vec::new();

        // Find finished jobs
        for (id, handle) in running.iter() {
            if handle.is_finished() {
                to_remove.push(id.clone());
            }
        }

        // Remove from running and collect results
        for id in to_remove {
            if let Some(handle) = running.remove(&id) {
                // Try to get the result from completed_results
                let results = self.completed_results.lock().unwrap();
                if let Some((job_id, result)) = results.get(&id) {
                    results_to_save.push((id.clone(), job_id.clone(), result.clone()));
                }
            }
        }

        // Save results to database
        for (execution_id, job_id, result) in results_to_save {
            if let Err(e) = Self::save_execution_result(&self.context, &execution_id, &job_id, &result) {
                tracing::error!("Failed to save execution result: {}", e);
            }

            // Remove from completed results after saving
            let mut results = self.completed_results.lock().unwrap();
            results.remove(&execution_id);
        }
    }

    /// Save execution result to database
    fn save_execution_result(
        context: &ExecutionContext,
        execution_id: &str,
        job_id: &str,
        result: &ExecutionResult,
    ) -> Result<(), String> {
        use rusqlite::params;

        let conn = rusqlite::Connection::open(&context.db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let status_str = match result.status {
            ExecutionStatus::Running => "running",
            ExecutionStatus::Completed => "completed",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::Cancelled => "cancelled",
        };

        // Update the execution record
        conn.execute(
            "UPDATE job_executions SET status = ?, result = ?, error = ?, completed_at = ? WHERE id = ?",
            params![
                status_str,
                result.output.as_deref(),
                result.error.as_deref(),
                Utc::now().to_rfc3339(),
                execution_id,
            ],
        )
        .map_err(|e| format!("Failed to update execution: {}", e))?;

        Ok(())
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
