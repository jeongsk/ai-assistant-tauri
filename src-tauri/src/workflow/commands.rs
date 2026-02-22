//! Tauri Commands for v0.6 Workflow Module
//!
//! Commands for workflow management, execution, and triggers.

use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::store::{
    Workflow, WorkflowDefinition, WorkflowExecution, ExecutionStatus,
    InMemoryWorkflowStore, WorkflowStore
};
use super::engine::{WorkflowExecutor, ExecutionResult};
use super::triggers::{TriggerManager, Trigger, TriggerType};
use super::nodes::{NodeExecutor, TriggerExecutor, ActionExecutor, ConditionExecutor, LoopExecutor, AgentExecutor};

/// Global state for workflow features
pub struct WorkflowState {
    pub store: Arc<RwLock<InMemoryWorkflowStore>>,
    pub executor: Arc<RwLock<WorkflowExecutor>>,
    pub triggers: Arc<RwLock<TriggerManager>>,
}

impl WorkflowState {
    pub fn new() -> Self {
        let executor = Arc::new(RwLock::new(WorkflowExecutor::new()));

        // Register built-in node executors
        let executor_clone = executor.clone();
        tokio::spawn(async move {
            let mut exec = executor_clone.write().await;
            exec.register_executor("trigger", Box::new(TriggerExecutor));
            exec.register_executor("action", Box::new(ActionExecutor));
            exec.register_executor("condition", Box::new(ConditionExecutor));
            exec.register_executor("loop", Box::new(LoopExecutor));
            exec.register_executor("agent", Box::new(AgentExecutor));
        });

        Self {
            store: Arc::new(RwLock::new(InMemoryWorkflowStore::new())),
            executor,
            triggers: Arc::new(RwLock::new(TriggerManager::new())),
        }
    }
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Workflow Store Commands
// ============================================================================

/// Create a new workflow
#[tauri::command]
pub async fn workflow_create(
    state: State<'_, Arc<WorkflowState>>,
    id: String,
    name: String,
    description: Option<String>,
    entry_point: String,
    is_active: Option<bool>,
) -> Result<String, String> {
    let mut store = state.store.write().await;

    let workflow = Workflow {
        id: id.clone(),
        name,
        description,
        definition: WorkflowDefinition {
            entry_point,
            nodes: std::collections::HashMap::new(),
            connections: vec![],
        },
        version: 1,
        is_active: is_active.unwrap_or(true),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    store.create(workflow)
}

/// Get a workflow by ID
#[tauri::command]
pub async fn workflow_get(
    state: State<'_, Arc<WorkflowState>>,
    id: String,
) -> Result<Option<Workflow>, String> {
    let store = state.store.read().await;
    store.get(&id)
}

/// List all workflows
#[tauri::command]
pub async fn workflow_list(
    state: State<'_, Arc<WorkflowState>>,
) -> Result<Vec<Workflow>, String> {
    let store = state.store.read().await;
    store.list()
}

/// List active workflows
#[tauri::command]
pub async fn workflow_list_active(
    state: State<'_, Arc<WorkflowState>>,
) -> Result<Vec<Workflow>, String> {
    let store = state.store.read().await;
    store.list_active()
}

/// Update a workflow
#[tauri::command]
pub async fn workflow_update(
    state: State<'_, Arc<WorkflowState>>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    entry_point: Option<String>,
    is_active: Option<bool>,
) -> Result<(), String> {
    let mut store = state.store.write().await;

    // Get existing workflow
    let mut workflow = store.get(&id)?
        .ok_or_else(|| "Workflow not found".to_string())?;

    // Update fields
    if let Some(name) = name {
        workflow.name = name;
    }
    if let Some(description) = description {
        workflow.description = Some(description);
    }
    if let Some(entry_point) = entry_point {
        workflow.definition.entry_point = entry_point;
    }
    if let Some(is_active) = is_active {
        workflow.is_active = is_active;
    }
    workflow.updated_at = chrono::Utc::now().to_rfc3339();

    store.update(workflow)
}

/// Delete a workflow
#[tauri::command]
pub async fn workflow_delete(
    state: State<'_, Arc<WorkflowState>>,
    id: String,
) -> Result<(), String> {
    let mut store = state.store.write().await;
    store.delete(&id)
}

/// Add a node to a workflow
#[tauri::command]
pub async fn workflow_add_node(
    state: State<'_, Arc<WorkflowState>>,
    workflow_id: String,
    node_id: String,
    node_type: String,
    x: f64,
    y: f64,
    data: Option<serde_json::Value>,
    label: Option<String>,
) -> Result<(), String> {
    let mut store = state.store.write().await;

    let mut workflow = store.get(&workflow_id)?
        .ok_or_else(|| "Workflow not found".to_string())?;

    use super::store::{WorkflowNode, NodePosition};

    let node = WorkflowNode {
        id: node_id.clone(),
        node_type,
        position: NodePosition { x, y },
        data: data.unwrap_or(serde_json::json!({})),
        label,
    };

    workflow.definition.nodes.insert(node_id, node);
    workflow.updated_at = chrono::Utc::now().to_rfc3339();

    store.update(workflow)
}

/// Add a connection between nodes
#[tauri::command]
pub async fn workflow_add_connection(
    state: State<'_, Arc<WorkflowState>>,
    workflow_id: String,
    source: String,
    source_output: String,
    target: String,
    target_input: String,
    condition: Option<String>,
) -> Result<(), String> {
    let mut store = state.store.write().await;

    let mut workflow = store.get(&workflow_id)?
        .ok_or_else(|| "Workflow not found".to_string())?;

    use super::store::NodeConnection;

    let connection = NodeConnection {
        source,
        source_output,
        target,
        target_input,
        condition,
    };

    workflow.definition.connections.push(connection);
    workflow.updated_at = chrono::Utc::now().to_rfc3339();

    store.update(workflow)
}

// ============================================================================
// Workflow Execution Commands
// ============================================================================

/// Execute a workflow
#[tauri::command]
pub async fn workflow_execute(
    state: State<'_, Arc<WorkflowState>>,
    id: String,
    input: Option<serde_json::Value>,
) -> Result<ExecutionResult, String> {
    let store = state.store.read().await;
    let executor = state.executor.read().await;

    let workflow = store.get(&id)?
        .ok_or_else(|| "Workflow not found".to_string())?;

    Ok(executor.execute(&workflow, input.unwrap_or(serde_json::json!(null))))
}

/// Create an execution record
#[tauri::command]
pub async fn workflow_create_execution(
    state: State<'_, Arc<WorkflowState>>,
    id: String,
    workflow_id: String,
    trigger_type: Option<String>,
) -> Result<String, String> {
    let mut store = state.store.write().await;

    let execution = WorkflowExecution {
        id: id.clone(),
        workflow_id,
        status: ExecutionStatus::Pending,
        trigger_type,
        started_at: Some(chrono::Utc::now().to_rfc3339()),
        completed_at: None,
        result: None,
        error: None,
    };

    store.create_execution(execution)
}

/// Get execution by ID
#[tauri::command]
pub async fn workflow_get_execution(
    state: State<'_, Arc<WorkflowState>>,
    id: String,
) -> Result<Option<WorkflowExecution>, String> {
    let store = state.store.read().await;
    store.get_execution(&id)
}

/// Get executions for a workflow
#[tauri::command]
pub async fn workflow_get_executions(
    state: State<'_, Arc<WorkflowState>>,
    workflow_id: String,
) -> Result<Vec<WorkflowExecution>, String> {
    let store = state.store.read().await;
    store.get_executions(&workflow_id)
}

/// Update execution status
#[tauri::command]
pub async fn workflow_update_execution(
    state: State<'_, Arc<WorkflowState>>,
    id: String,
    status: String,
    result: Option<serde_json::Value>,
    error: Option<String>,
) -> Result<(), String> {
    let mut store = state.store.write().await;

    let mut execution = store.get_execution(&id)?
        .ok_or_else(|| "Execution not found".to_string())?;

    execution.status = match status.as_str() {
        "pending" => ExecutionStatus::Pending,
        "running" => ExecutionStatus::Running,
        "completed" => ExecutionStatus::Completed,
        "failed" => ExecutionStatus::Failed,
        "cancelled" => ExecutionStatus::Cancelled,
        _ => return Err("Invalid status".to_string()),
    };

    execution.result = result;
    execution.error = error;

    if matches!(execution.status, ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled) {
        execution.completed_at = Some(chrono::Utc::now().to_rfc3339());
    }

    store.update_execution(execution)
}

// ============================================================================
// Trigger Commands
// ============================================================================

/// Register a trigger for a workflow
#[tauri::command]
pub async fn workflow_register_trigger(
    state: State<'_, Arc<WorkflowState>>,
    trigger_id: String,
    workflow_id: String,
    trigger_type: String,
    config: Option<serde_json::Value>,
) -> Result<(), String> {
    let triggers = state.triggers.read().await;

    let trigger = match trigger_type.as_str() {
        "schedule" => {
            let cron = config
                .as_ref()
                .and_then(|c| c.get("cron"))
                .and_then(|c| c.as_str())
                .unwrap_or("* * * * *")
                .to_string();
            let timezone = config
                .as_ref()
                .and_then(|c| c.get("timezone"))
                .and_then(|c| c.as_str())
                .unwrap_or("UTC")
                .to_string();
            Trigger::Schedule { cron, timezone }
        }
        "webhook" => {
            let path = config
                .as_ref()
                .and_then(|c| c.get("path"))
                .and_then(|c| c.as_str())
                .unwrap_or("/webhook")
                .to_string();
            let method_str = config
                .as_ref()
                .and_then(|c| c.get("method"))
                .and_then(|c| c.as_str())
                .unwrap_or("POST");
            let method = match method_str {
                "GET" => super::triggers::HttpMethod::Get,
                "POST" => super::triggers::HttpMethod::Post,
                "PUT" => super::triggers::HttpMethod::Put,
                "DELETE" => super::triggers::HttpMethod::Delete,
                _ => return Err("Invalid HTTP method".to_string()),
            };
            Trigger::Webhook { path, method }
        }
        "filesystem" => {
            let path = config
                .as_ref()
                .and_then(|c| c.get("path"))
                .and_then(|c| c.as_str())
                .unwrap_or("/")
                .to_string();
            let events_config = config
                .as_ref()
                .and_then(|c| c.get("events"))
                .and_then(|c| c.as_array())
                .cloned()
                .unwrap_or_default();
            let events: Vec<super::triggers::FsEvent> = events_config
                .iter()
                .filter_map(|e| {
                    match e.as_str() {
                        Some("create") => Some(super::triggers::FsEvent::Create),
                        Some("modify") => Some(super::triggers::FsEvent::Modify),
                        Some("delete") => Some(super::triggers::FsEvent::Delete),
                        Some("rename") => Some(super::triggers::FsEvent::Rename),
                        _ => None,
                    }
                })
                .collect();
            Trigger::FileSystem { path, events }
        }
        "voice" => {
            let pattern = config
                .as_ref()
                .and_then(|c| c.get("pattern"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            let language = config
                .as_ref()
                .and_then(|c| c.get("language"))
                .and_then(|c| c.as_str())
                .unwrap_or("en")
                .to_string();
            Trigger::Voice { pattern, language }
        }
        "manual" => Trigger::Manual,
        _ => return Err("Invalid trigger type".to_string()),
    };

    triggers.register(trigger_id, workflow_id, &trigger).await
}

/// Unregister a trigger
#[tauri::command]
pub async fn workflow_unregister_trigger(
    state: State<'_, Arc<WorkflowState>>,
    trigger_id: String,
) -> Result<(), String> {
    let triggers = state.triggers.read().await;
    triggers.unregister(&trigger_id).await
}

/// List active triggers
#[tauri::command]
pub async fn workflow_list_triggers(
    state: State<'_, Arc<WorkflowState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let triggers = state.triggers.read().await;
    let handles = triggers.list_active().await;

    let json_handles: Vec<serde_json::Value> = handles
        .into_iter()
        .map(|h| serde_json::json!({
            "trigger_id": h.trigger_id,
            "workflow_id": h.workflow_id,
            "trigger_type": format!("{:?}", h.trigger_type),
        }))
        .collect();

    Ok(json_handles)
}

/// Get trigger count
#[tauri::command]
pub async fn workflow_trigger_count(
    state: State<'_, Arc<WorkflowState>>,
) -> Result<usize, String> {
    let triggers = state.triggers.read().await;
    Ok(triggers.count().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_state_creation() {
        let state = WorkflowState::new();
        // Just verify it creates without panic
        assert_eq!(state.triggers.read().await.count().await, 0);
    }
}
