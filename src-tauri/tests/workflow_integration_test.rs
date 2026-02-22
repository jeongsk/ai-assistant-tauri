//! Integration tests for v0.6 Workflow Module
//!
//! Tests for workflow management, execution, and triggers.

#[cfg(test)]
mod tests {
    use ai_assistant_tauri_lib::workflow::store::ExecutionStatus;
    use ai_assistant_tauri_lib::workflow::store::InMemoryWorkflowStore;
    use ai_assistant_tauri_lib::workflow::store::Workflow;
    use ai_assistant_tauri_lib::workflow::store::WorkflowDefinition;
    use ai_assistant_tauri_lib::workflow::store::WorkflowExecution;
    use ai_assistant_tauri_lib::workflow::store::WorkflowStore;
    use ai_assistant_tauri_lib::workflow::triggers::TriggerType;
    use ai_assistant_tauri_lib::workflow::triggers::HttpMethod;
    use ai_assistant_tauri_lib::workflow::triggers::FsEvent;
    use ai_assistant_tauri_lib::workflow::triggers::TriggerManager;
    use ai_assistant_tauri_lib::workflow::triggers::Trigger;
    use ai_assistant_tauri_lib::workflow::engine::WorkflowExecutor;
    use std::collections::HashMap;

    #[test]
    fn test_workflow_execution_status() {
        // Test ExecutionStatus variants
        let _ = ExecutionStatus::Pending;
        let _ = ExecutionStatus::Running;
        let _ = ExecutionStatus::Completed;
        let _ = ExecutionStatus::Failed;
        let _ = ExecutionStatus::Cancelled;
    }

    #[test]
    fn test_trigger_type_equality() {
        // Test TriggerType variants
        assert_eq!(TriggerType::Manual, TriggerType::Manual);
        assert_ne!(TriggerType::Schedule, TriggerType::Manual);
    }

    #[test]
    fn test_http_method_variants() {
        // Test HttpMethod variants
        let _ = HttpMethod::Get;
        let _ = HttpMethod::Post;
        let _ = HttpMethod::Put;
        let _ = HttpMethod::Delete;
    }

    #[test]
    fn test_fs_event_variants() {
        // Test FsEvent variants
        let _ = FsEvent::Create;
        let _ = FsEvent::Modify;
        let _ = FsEvent::Delete;
        let _ = FsEvent::Rename;
    }

    #[tokio::test]
    async fn test_workflow_store_creation() {
        let store = InMemoryWorkflowStore::new();
        assert_eq!(store.list().unwrap().len(), 0);
        assert_eq!(store.list_active().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_workflow_executor_creation() {
        let _executor = WorkflowExecutor::new();
        // Just verify it creates without panic
    }

    #[tokio::test]
    async fn test_trigger_manager_creation() {
        let manager = TriggerManager::new();
        assert_eq!(manager.count().await, 0);
        assert!(!manager.exists("test-trigger").await);
    }

    #[tokio::test]
    async fn test_workflow_crud() {
        let mut store = InMemoryWorkflowStore::new();

        let workflow = Workflow {
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
        };

        // Create
        let id = store.create(workflow.clone()).unwrap();
        assert_eq!(id, "test-1");

        // Get
        let retrieved = store.get(&id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test Workflow");

        // Update
        let mut updated = retrieved;
        updated.name = "Updated Workflow".to_string();
        store.update(updated).unwrap();
        let updated = store.get(&id).unwrap().unwrap();
        assert_eq!(updated.name, "Updated Workflow");

        // Delete
        store.delete(&id).unwrap();
        assert!(store.get(&id).unwrap().is_none());
    }

    #[tokio::test]
    async fn test_workflow_execution_crud() {
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

        // Create
        let id = store.create_execution(execution.clone()).unwrap();
        assert_eq!(id, "exec-1");

        // Get
        let retrieved = store.get_execution(&id).unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Pending);

        // Update
        let mut updated = retrieved;
        updated.status = ExecutionStatus::Completed;
        store.update_execution(updated).unwrap();

        // Get executions for workflow
        let executions = store.get_executions(&execution.workflow_id).unwrap();
        assert_eq!(executions.len(), 1);
    }

    #[tokio::test]
    async fn test_trigger_registration() {
        let manager = TriggerManager::new();

        // Register a manual trigger
        manager.register("trigger-1".to_string(), "workflow-1".to_string(), &Trigger::Manual).await.unwrap();
        assert_eq!(manager.count().await, 1);
        assert!(manager.exists("trigger-1").await);

        // Unregister
        manager.unregister("trigger-1").await.unwrap();
        assert_eq!(manager.count().await, 0);
    }
}
