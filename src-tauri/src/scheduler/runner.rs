//! Job runner for scheduled tasks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
