//! Integration tests for v0.6 Sync Module
//!
//! Tests for synchronization, conflict resolution, and offline queue.

#[cfg(test)]
mod tests {
    use ai_assistant_tauri_lib::sync::manager::SyncEntity;
    use ai_assistant_tauri_lib::sync::manager::SyncOperation;
    use ai_assistant_tauri_lib::sync::manager::SyncManager;
    use ai_assistant_tauri_lib::sync::conflict::ConflictStrategy;
    use ai_assistant_tauri_lib::sync::conflict::ConflictResolution;
    use ai_assistant_tauri_lib::sync::conflict::ConflictResolver;
    use ai_assistant_tauri_lib::sync::conflict::SyncConflict;
    use ai_assistant_tauri_lib::sync::offline::OfflineQueue;
    use ai_assistant_tauri_lib::sync::offline::PendingOperation;
    use chrono::Utc;

    #[test]
    fn test_sync_entity_variants() {
        // Test SyncEntity variants
        let _ = SyncEntity::Settings;
        let _ = SyncEntity::Conversation;
        let _ = SyncEntity::Template;
        let _ = SyncEntity::Skill;
        let _ = SyncEntity::Recipe;
    }

    #[test]
    fn test_sync_operation_variants() {
        // Test SyncOperation variants
        let _ = SyncOperation::Upload;
        let _ = SyncOperation::Download;
        let _ = SyncOperation::Delete;
    }

    #[test]
    fn test_conflict_strategy_variants() {
        // Test ConflictStrategy variants
        let _ = ConflictStrategy::ClientWins;
        let _ = ConflictStrategy::ServerWins;
        let _ = ConflictStrategy::Merge;
        let _ = ConflictStrategy::Manual;
    }

    #[test]
    fn test_conflict_resolution_variants() {
        // Test ConflictResolution variants
        let _ = ConflictResolution::KeepLocal;
        let _ = ConflictResolution::KeepRemote;
        let _ = ConflictResolution::Merged(vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let manager = SyncManager::new(None);
        assert!(!manager.needs_sync().await);
        assert_eq!(manager.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_sync_manager_queue_operations() {
        let manager = SyncManager::new(None);

        // Queue operations
        manager.queue_upload(SyncEntity::Settings, "test-1".to_string(), vec![1, 2, 3]).await;
        manager.queue_download(SyncEntity::Template, "template-1".to_string()).await;
        manager.queue_delete(SyncEntity::Conversation, "conv-1".to_string()).await;

        assert_eq!(manager.pending_count().await, 3);
        assert!(manager.needs_sync().await);

        // Clear pending
        manager.clear_pending().await;
        assert_eq!(manager.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_offline_queue_creation() {
        let queue = OfflineQueue::new();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn test_offline_queue_operations() {
        let mut queue = OfflineQueue::new();

        let operation = PendingOperation {
            id: "op-1".to_string(),
            entity: SyncEntity::Settings,
            entity_id: "test".to_string(),
            operation: SyncOperation::Upload,
            data: Some(vec![1, 2, 3]),
            attempts: 0,
            max_attempts: 3,
            last_error: None,
            created_at: Utc::now(),
            last_attempt: None,
        };

        // Push
        queue.push(operation.clone()).unwrap();
        assert_eq!(queue.len(), 1);

        // Peek
        let peeked = queue.peek();
        assert!(peeked.is_some());
        assert_eq!(queue.len(), 1); // Not removed

        // Pop ready
        let popped = queue.pop_ready();
        assert!(popped.is_some());
        assert_eq!(queue.len(), 0);
    }

    #[tokio::test]
    async fn test_offline_queue_mark_failed() {
        let mut queue = OfflineQueue::new();

        let operation = PendingOperation {
            id: "op-1".to_string(),
            entity: SyncEntity::Settings,
            entity_id: "test".to_string(),
            operation: SyncOperation::Upload,
            data: Some(vec![1, 2, 3]),
            attempts: 0,
            max_attempts: 3,
            last_error: None,
            created_at: Utc::now(),
            last_attempt: None,
        };

        queue.mark_failed(operation, "Network error".to_string());
        assert_eq!(queue.len(), 1);

        let queued = queue.peek().unwrap();
        assert_eq!(queued.attempts, 1);
        assert_eq!(queued.last_error, Some("Network error".to_string()));
    }

    #[tokio::test]
    async fn test_offline_queue_max_attempts() {
        let mut queue = OfflineQueue::new();

        let mut operation = PendingOperation {
            id: "op-1".to_string(),
            entity: SyncEntity::Settings,
            entity_id: "test".to_string(),
            operation: SyncOperation::Upload,
            data: Some(vec![1, 2, 3]),
            attempts: 2, // Already at max - 1
            max_attempts: 3,
            last_error: None,
            created_at: Utc::now(),
            last_attempt: None,
        };

        queue.mark_failed(operation, "Final error".to_string());
        // Should not be re-added
        assert!(queue.is_empty());
    }

    #[test]
    fn test_conflict_detection() {
        let resolver = ConflictResolver::new(ConflictStrategy::ClientWins);

        // Different versions = conflict
        let conflict = resolver.detect(
            SyncEntity::Settings,
            "settings-1",
            "v1",
            "v2",
        );
        assert!(conflict.is_some());

        // Same versions = no conflict
        let no_conflict = resolver.detect(
            SyncEntity::Settings,
            "settings-1",
            "v1",
            "v1",
        );
        assert!(no_conflict.is_none());
    }

    #[test]
    fn test_conflict_resolution_strategies() {
        // Test client wins
        let client_resolver = ConflictResolver::new(ConflictStrategy::ClientWins);
        let mut conflict = SyncConflict {
            entity: SyncEntity::Settings,
            id: "test".to_string(),
            local_version: "v1".to_string(),
            remote_version: "v2".to_string(),
            local_data: vec![1, 2, 3],
            remote_data: vec![4, 5, 6],
            resolution: None,
        };
        let resolution = client_resolver.resolve(&mut conflict);
        assert_eq!(resolution, ConflictResolution::KeepLocal);

        // Test server wins
        let server_resolver = ConflictResolver::new(ConflictStrategy::ServerWins);
        let mut conflict = SyncConflict {
            entity: SyncEntity::Settings,
            id: "test".to_string(),
            local_version: "v1".to_string(),
            remote_version: "v2".to_string(),
            local_data: vec![1, 2, 3],
            remote_data: vec![4, 5, 6],
            resolution: None,
        };
        let resolution = server_resolver.resolve(&mut conflict);
        assert_eq!(resolution, ConflictResolution::KeepRemote);

        // Test merge
        let merge_resolver = ConflictResolver::new(ConflictStrategy::Merge);
        let mut conflict = SyncConflict {
            entity: SyncEntity::Settings,
            id: "test".to_string(),
            local_version: "v1".to_string(),
            remote_version: "v2".to_string(),
            local_data: vec![1, 2, 3],
            remote_data: vec![4, 5, 6],
            resolution: None,
        };
        let resolution = merge_resolver.resolve(&mut conflict);
        assert!(matches!(resolution, ConflictResolution::Merged(_)));
    }

    #[test]
    fn test_offline_queue_with_settings() {
        let mut queue = OfflineQueue::new();

        let operation = PendingOperation {
            id: "op-1".to_string(),
            entity: SyncEntity::Settings,
            entity_id: "test".to_string(),
            operation: SyncOperation::Upload,
            data: Some(vec![1, 2, 3]),
            attempts: 0,
            max_attempts: 3,
            last_error: None,
            created_at: Utc::now(),
            last_attempt: None,
        };

        queue.push(operation).unwrap();
        let settings_ops = queue.get_by_entity(&SyncEntity::Settings);
        assert_eq!(settings_ops.len(), 1);
    }

    #[tokio::test]
    async fn test_offline_queue_failed_operations() {
        let mut queue = OfflineQueue::new();

        // Add some operations
        let failed_op = PendingOperation {
            id: "failed-1".to_string(),
            entity: SyncEntity::Settings,
            entity_id: "test".to_string(),
            operation: SyncOperation::Upload,
            data: Some(vec![1, 2, 3]),
            attempts: 3, // Max attempts reached
            max_attempts: 3,
            last_error: Some("Error".to_string()),
            created_at: Utc::now(),
            last_attempt: Some(Utc::now()),
        };

        let ok_op = PendingOperation {
            id: "ok-1".to_string(),
            entity: SyncEntity::Settings,
            entity_id: "test2".to_string(),
            operation: SyncOperation::Upload,
            data: Some(vec![4, 5, 6]),
            attempts: 1,
            max_attempts: 3,
            last_error: None,
            created_at: Utc::now(),
            last_attempt: None,
        };

        queue.push(failed_op.clone()).unwrap();
        queue.push(ok_op).unwrap();

        let failed = queue.get_failed();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].id, "failed-1");
    }
}
