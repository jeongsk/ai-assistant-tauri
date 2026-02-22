//! Workflow Persistence Layer
//! 
//! Manages storage and retrieval of workflows and their executions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Workflow definition (JSON)
    pub definition: WorkflowDefinition,
    /// Version number
    pub version: i64,
    /// Active status
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Workflow definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Entry point node ID
    pub entry_point: String,
    /// All nodes in the workflow
    pub nodes: HashMap<String, WorkflowNode>,
    /// Connections between nodes
    pub connections: Vec<NodeConnection>,
}

/// A node in the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// Node ID
    pub id: String,
    /// Node type
    pub node_type: String,
    /// Node position for visual editor
    pub position: NodePosition,
    /// Node-specific data
    pub data: serde_json::Value,
    /// Node label
    pub label: Option<String>,
}

/// Node position in visual editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

/// Connection between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConnection {
    /// Source node ID
    pub source: String,
    /// Source output port
    pub source_output: String,
    /// Target node ID
    pub target: String,
    /// Target input port
    pub target_input: String,
    /// Optional condition for conditional connections
    pub condition: Option<String>,
}

/// Workflow execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Execution ID
    pub id: String,
    /// Workflow ID
    pub workflow_id: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Trigger type that started this execution
    pub trigger_type: Option<String>,
    /// Execution start time
    pub started_at: Option<String>,
    /// Execution completion time
    pub completed_at: Option<String>,
    /// Execution result
    pub result: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Workflow store trait for persistence
pub trait WorkflowStore: Send + Sync {
    /// Create a new workflow
    fn create(&mut self, workflow: Workflow) -> Result<String, String>;
    
    /// Get workflow by ID
    fn get(&self, id: &str) -> Result<Option<Workflow>, String>;
    
    /// Update workflow
    fn update(&mut self, workflow: Workflow) -> Result<(), String>;
    
    /// Delete workflow
    fn delete(&mut self, id: &str) -> Result<(), String>;
    
    /// List all workflows
    fn list(&self) -> Result<Vec<Workflow>, String>;
    
    /// List active workflows
    fn list_active(&self) -> Result<Vec<Workflow>, String>;
    
    /// Create execution record
    fn create_execution(&mut self, execution: WorkflowExecution) -> Result<String, String>;
    
    /// Update execution status
    fn update_execution(&mut self, execution: WorkflowExecution) -> Result<(), String>;
    
    /// Get execution by ID
    fn get_execution(&self, id: &str) -> Result<Option<WorkflowExecution>, String>;
    
    /// Get executions for workflow
    fn get_executions(&self, workflow_id: &str) -> Result<Vec<WorkflowExecution>, String>;
}

/// In-memory workflow store for testing
pub struct InMemoryWorkflowStore {
    workflows: HashMap<String, Workflow>,
    executions: HashMap<String, WorkflowExecution>,
}

impl InMemoryWorkflowStore {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            executions: HashMap::new(),
        }
    }
}

impl Default for InMemoryWorkflowStore {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowStore for InMemoryWorkflowStore {
    fn create(&mut self, workflow: Workflow) -> Result<String, String> {
        let id = workflow.id.clone();
        self.workflows.insert(id.clone(), workflow);
        Ok(id)
    }
    
    fn get(&self, id: &str) -> Result<Option<Workflow>, String> {
        Ok(self.workflows.get(id).cloned())
    }
    
    fn update(&mut self, workflow: Workflow) -> Result<(), String> {
        let id = workflow.id.clone();
        if self.workflows.contains_key(&id) {
            self.workflows.insert(id, workflow);
            Ok(())
        } else {
            Err("Workflow not found".to_string())
        }
    }
    
    fn delete(&mut self, id: &str) -> Result<(), String> {
        self.workflows.remove(id)
            .map(|_| ())
            .ok_or_else(|| "Workflow not found".to_string())
    }
    
    fn list(&self) -> Result<Vec<Workflow>, String> {
        Ok(self.workflows.values().cloned().collect())
    }
    
    fn list_active(&self) -> Result<Vec<Workflow>, String> {
        Ok(self.workflows.values()
            .filter(|w| w.is_active)
            .cloned()
            .collect())
    }
    
    fn create_execution(&mut self, execution: WorkflowExecution) -> Result<String, String> {
        let id = execution.id.clone();
        self.executions.insert(id.clone(), execution);
        Ok(id)
    }
    
    fn update_execution(&mut self, execution: WorkflowExecution) -> Result<(), String> {
        let id = execution.id.clone();
        if self.executions.contains_key(&id) {
            self.executions.insert(id, execution);
            Ok(())
        } else {
            Err("Execution not found".to_string())
        }
    }
    
    fn get_execution(&self, id: &str) -> Result<Option<WorkflowExecution>, String> {
        Ok(self.executions.get(id).cloned())
    }
    
    fn get_executions(&self, workflow_id: &str) -> Result<Vec<WorkflowExecution>, String> {
        Ok(self.executions.values()
            .filter(|e| e.workflow_id == workflow_id)
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_workflow() -> Workflow {
        Workflow {
            id: "test-1".to_string(),
            name: "Test Workflow".to_string(),
            description: Some("A test workflow".to_string()),
            definition: WorkflowDefinition {
                entry_point: "node-1".to_string(),
                nodes: HashMap::new(),
                connections: vec![],
            },
            version: 1,
            is_active: true,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_create_and_get_workflow() {
        let mut store = InMemoryWorkflowStore::new();
        let workflow = make_test_workflow();
        let id = store.create(workflow.clone()).unwrap();
        
        assert_eq!(id, "test-1");
        
        let retrieved = store.get(&id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test Workflow");
    }

    #[test]
    fn test_update_workflow() {
        let mut store = InMemoryWorkflowStore::new();
        let mut workflow = make_test_workflow();
        store.create(workflow.clone()).unwrap();
        
        workflow.name = "Updated Name".to_string();
        store.update(workflow).unwrap();
        
        let retrieved = store.get("test-1").unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
    }

    #[test]
    fn test_delete_workflow() {
        let mut store = InMemoryWorkflowStore::new();
        let workflow = make_test_workflow();
        store.create(workflow).unwrap();
        
        store.delete("test-1").unwrap();
        
        let retrieved = store.get("test-1").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_list_active_workflows() {
        let mut store = InMemoryWorkflowStore::new();
        
        let mut active = make_test_workflow();
        active.id = "active-1".to_string();
        active.is_active = true;
        
        let mut inactive = make_test_workflow();
        inactive.id = "inactive-1".to_string();
        inactive.is_active = false;
        
        store.create(active).unwrap();
        store.create(inactive).unwrap();
        
        let active_list = store.list_active().unwrap();
        assert_eq!(active_list.len(), 1);
    }

    #[test]
    fn test_execution_crud() {
        let mut store = InMemoryWorkflowStore::new();
        
        let execution = WorkflowExecution {
            id: "exec-1".to_string(),
            workflow_id: "test-1".to_string(),
            status: ExecutionStatus::Pending,
            trigger_type: Some("manual".to_string()),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        };
        
        store.create_execution(execution).unwrap();
        
        let retrieved = store.get_execution("exec-1").unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Pending);
    }
}
