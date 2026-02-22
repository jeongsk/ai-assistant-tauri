//! Tauri Commands for v0.6 Sync Module
//!
//! Commands for cloud synchronization, conflict resolution, and offline queue management.

use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::manager::{SyncManager, SyncEntity, SyncOperation, SyncResult, CloudProvider};
use super::conflict::{ConflictResolver, ConflictStrategy, SyncConflict, ConflictResolution};
use super::offline::{OfflineQueue, PendingOperation};

/// Mock cloud provider for testing
struct MockCloudProvider;

impl CloudProvider for MockCloudProvider {
    fn upload(&self, _entity: SyncEntity, _id: &str, _data: &[u8]) -> Result<String, String> {
        Ok("mock-upload-id".to_string())
    }

    fn download(&self, _entity: SyncEntity, _id: &str) -> Result<Vec<u8>, String> {
        Ok(vec![1, 2, 3, 4])
    }

    fn list(&self, _entity: SyncEntity) -> Result<Vec<(String, String)>, String> {
        Ok(vec![("item-1".to_string(), "v1".to_string())])
    }

    fn delete(&self, _entity: SyncEntity, _id: &str) -> Result<(), String> {
        Ok(())
    }

    fn get_last_sync(&self, _entity: SyncEntity, _id: &str) -> Result<Option<String>, String> {
        Ok(Some("2026-01-01T00:00:00Z".to_string()))
    }
}

/// Global state for sync features
pub struct SyncState {
    pub manager: Arc<RwLock<SyncManager>>,
    pub offline_queue: Arc<RwLock<OfflineQueue>>,
    pub conflict_resolver: Arc<RwLock<ConflictResolver>>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(SyncManager::new(None))),
            offline_queue: Arc::new(RwLock::new(OfflineQueue::new())),
            conflict_resolver: Arc::new(RwLock::new(ConflictResolver::new(ConflictStrategy::Manual))),
        }
    }

    pub fn with_mock_provider() -> Self {
        Self {
            manager: Arc::new(RwLock::new(SyncManager::new(Some(Arc::new(MockCloudProvider))))),
            offline_queue: Arc::new(RwLock::new(OfflineQueue::new())),
            conflict_resolver: Arc::new(RwLock::new(ConflictResolver::new(ConflictStrategy::Manual))),
        }
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Sync Manager Commands
// ============================================================================

/// Perform sync now
#[tauri::command]
pub async fn sync_now(
    state: State<'_, Arc<SyncState>>,
) -> Result<SyncResult, String> {
    let manager = state.manager.read().await;
    Ok(manager.sync_now().await)
}

/// Queue an upload operation
#[tauri::command]
pub async fn sync_queue_upload(
    state: State<'_, Arc<SyncState>>,
    entity: String,
    id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let manager = state.manager.read().await;

    let sync_entity = match entity.as_str() {
        "settings" => SyncEntity::Settings,
        "conversation" => SyncEntity::Conversation,
        "template" => SyncEntity::Template,
        "skill" => SyncEntity::Skill,
        "recipe" => SyncEntity::Recipe,
        _ => return Err("Invalid entity type".to_string()),
    };

    manager.queue_upload(sync_entity, id, data).await;
    Ok(())
}

/// Queue a download operation
#[tauri::command]
pub async fn sync_queue_download(
    state: State<'_, Arc<SyncState>>,
    entity: String,
    id: String,
) -> Result<(), String> {
    let manager = state.manager.read().await;

    let sync_entity = match entity.as_str() {
        "settings" => SyncEntity::Settings,
        "conversation" => SyncEntity::Conversation,
        "template" => SyncEntity::Template,
        "skill" => SyncEntity::Skill,
        "recipe" => SyncEntity::Recipe,
        _ => return Err("Invalid entity type".to_string()),
    };

    manager.queue_download(sync_entity, id).await;
    Ok(())
}

/// Queue a delete operation
#[tauri::command]
pub async fn sync_queue_delete(
    state: State<'_, Arc<SyncState>>,
    entity: String,
    id: String,
) -> Result<(), String> {
    let manager = state.manager.read().await;

    let sync_entity = match entity.as_str() {
        "settings" => SyncEntity::Settings,
        "conversation" => SyncEntity::Conversation,
        "template" => SyncEntity::Template,
        "skill" => SyncEntity::Skill,
        "recipe" => SyncEntity::Recipe,
        _ => return Err("Invalid entity type".to_string()),
    };

    manager.queue_delete(sync_entity, id).await;
    Ok(())
}

/// Get pending operation count
#[tauri::command]
pub async fn sync_pending_count(
    state: State<'_, Arc<SyncState>>,
) -> Result<usize, String> {
    let manager = state.manager.read().await;
    Ok(manager.pending_count().await)
}

/// Check if sync is needed
#[tauri::command]
pub async fn sync_needs_sync(
    state: State<'_, Arc<SyncState>>,
) -> Result<bool, String> {
    let manager = state.manager.read().await;
    Ok(manager.needs_sync().await)
}

/// Clear pending operations
#[tauri::command]
pub async fn sync_clear_pending(
    state: State<'_, Arc<SyncState>>,
) -> Result<(), String> {
    let manager = state.manager.read().await;
    manager.clear_pending().await;
    Ok(())
}

/// Set conflict resolution strategy
#[tauri::command]
pub async fn sync_set_conflict_strategy(
    state: State<'_, Arc<SyncState>>,
    strategy: String,
) -> Result<(), String> {
    let mut manager = state.manager.write().await;

    let conflict_strategy = match strategy.as_str() {
        "client_wins" => ConflictStrategy::ClientWins,
        "server_wins" => ConflictStrategy::ServerWins,
        "merge" => ConflictStrategy::Merge,
        "manual" => ConflictStrategy::Manual,
        _ => return Err("Invalid conflict strategy".to_string()),
    };

    manager.set_conflict_strategy(conflict_strategy);
    Ok(())
}

// ============================================================================
// Conflict Resolution Commands
// ============================================================================

/// Detect a conflict
#[tauri::command]
pub async fn sync_detect_conflict(
    state: State<'_, Arc<SyncState>>,
    entity: String,
    id: String,
    local_version: String,
    remote_version: String,
) -> Result<Option<SyncConflict>, String> {
    let resolver = state.conflict_resolver.read().await;

    let sync_entity = match entity.as_str() {
        "settings" => SyncEntity::Settings,
        "conversation" => SyncEntity::Conversation,
        "template" => SyncEntity::Template,
        "skill" => SyncEntity::Skill,
        "recipe" => SyncEntity::Recipe,
        _ => return Err("Invalid entity type".to_string()),
    };

    Ok(resolver.detect(sync_entity, &id, &local_version, &remote_version))
}

/// Resolve a conflict
#[tauri::command]
pub async fn sync_resolve_conflict(
    state: State<'_, Arc<SyncState>>,
    entity: String,
    id: String,
    local_version: String,
    remote_version: String,
    local_data: Vec<u8>,
    remote_data: Vec<u8>,
    resolution: String,
) -> Result<ConflictResolution, String> {
    let resolver = state.conflict_resolver.read().await;

    let sync_entity = match entity.as_str() {
        "settings" => SyncEntity::Settings,
        "conversation" => SyncEntity::Conversation,
        "template" => SyncEntity::Template,
        "skill" => SyncEntity::Skill,
        "recipe" => SyncEntity::Recipe,
        _ => return Err("Invalid entity type".to_string()),
    };

    let mut conflict = SyncConflict {
        entity: sync_entity,
        id,
        local_version,
        remote_version,
        local_data,
        remote_data,
        resolution: None,
    };

    // Pre-set resolution if provided
    if !resolution.is_empty() {
        conflict.resolution = Some(match resolution.as_str() {
            "keep_local" => ConflictResolution::KeepLocal,
            "keep_remote" => ConflictResolution::KeepRemote,
            _ => return Err("Invalid resolution type".to_string()),
        });
    }

    Ok(resolver.resolve(&mut conflict))
}

// ============================================================================
// Offline Queue Commands
// ============================================================================

/// Push an operation to the offline queue
#[tauri::command]
pub async fn sync_offline_push(
    state: State<'_, Arc<SyncState>>,
    id: String,
    entity: String,
    entity_id: String,
    operation: String,
    data: Option<Vec<u8>>,
) -> Result<(), String> {
    let mut queue = state.offline_queue.write().await;

    let sync_entity = match entity.as_str() {
        "settings" => SyncEntity::Settings,
        "conversation" => SyncEntity::Conversation,
        "template" => SyncEntity::Template,
        "skill" => SyncEntity::Skill,
        "recipe" => SyncEntity::Recipe,
        _ => return Err("Invalid entity type".to_string()),
    };

    let sync_operation = match operation.as_str() {
        "upload" => SyncOperation::Upload,
        "download" => SyncOperation::Download,
        "delete" => SyncOperation::Delete,
        _ => return Err("Invalid operation type".to_string()),
    };

    let pending = PendingOperation {
        id,
        entity: sync_entity,
        entity_id,
        operation: sync_operation,
        data,
        attempts: 0,
        max_attempts: 3,
        last_error: None,
        created_at: chrono::Utc::now(),
        last_attempt: None,
    };

    queue.push(pending)
}

/// Pop the next ready operation from the offline queue
#[tauri::command]
pub async fn sync_offline_pop_ready(
    state: State<'_, Arc<SyncState>>,
) -> Result<Option<PendingOperation>, String> {
    let mut queue = state.offline_queue.write().await;
    Ok(queue.pop_ready())
}

/// Peek at the next operation without removing it
#[tauri::command]
pub async fn sync_offline_peek(
    state: State<'_, Arc<SyncState>>,
) -> Result<Option<PendingOperation>, String> {
    let queue = state.offline_queue.read().await;
    Ok(queue.peek().cloned())
}

/// Mark an operation as failed
#[tauri::command]
pub async fn sync_offline_mark_failed(
    state: State<'_, Arc<SyncState>>,
    operation: PendingOperation,
    error: String,
) -> Result<(), String> {
    let mut queue = state.offline_queue.write().await;
    queue.mark_failed(operation, error);
    Ok(())
}

/// Get offline queue length
#[tauri::command]
pub async fn sync_offline_length(
    state: State<'_, Arc<SyncState>>,
) -> Result<usize, String> {
    let queue = state.offline_queue.read().await;
    Ok(queue.len())
}

/// Clear the offline queue
#[tauri::command]
pub async fn sync_offline_clear(
    state: State<'_, Arc<SyncState>>,
) -> Result<(), String> {
    let mut queue = state.offline_queue.write().await;
    queue.clear();
    Ok(())
}

/// Get failed operations from the offline queue
#[tauri::command]
pub async fn sync_offline_get_failed(
    state: State<'_, Arc<SyncState>>,
) -> Result<Vec<PendingOperation>, String> {
    let queue = state.offline_queue.read().await;
    Ok(queue.get_failed().into_iter().cloned().collect())
}

/// Get operations by entity type
#[tauri::command]
pub async fn sync_offline_get_by_entity(
    state: State<'_, Arc<SyncState>>,
    entity: String,
) -> Result<Vec<PendingOperation>, String> {
    let queue = state.offline_queue.read().await;

    let sync_entity = match entity.as_str() {
        "settings" => SyncEntity::Settings,
        "conversation" => SyncEntity::Conversation,
        "template" => SyncEntity::Template,
        "skill" => SyncEntity::Skill,
        "recipe" => SyncEntity::Recipe,
        _ => return Err("Invalid entity type".to_string()),
    };

    Ok(queue.get_by_entity(&sync_entity).into_iter().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_state_creation() {
        let state = SyncState::new();
        // Just verify it creates without panic
        assert_eq!(state.offline_queue.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_sync_state_with_mock() {
        let state = SyncState::with_mock_provider();
        // Just verify it creates without panic
        assert_eq!(state.offline_queue.read().await.len(), 0);
    }
}
