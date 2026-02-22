//! Sync Manager
//! 
//! Coordinates synchronization between local and cloud storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use super::conflict::{ConflictResolver, ConflictStrategy, SyncConflict};

/// Entity types that can be synced
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SyncEntity {
    Settings,
    Conversation,
    Template,
    Skill,
    Recipe,
}

/// Sync operation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncOperation {
    /// Upload local changes to cloud
    Upload,
    /// Download cloud changes to local
    Download,
    /// Delete from both
    Delete,
}

/// Sync result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub uploaded: usize,
    pub downloaded: usize,
    pub conflicts: Vec<SyncConflict>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// Cloud provider trait
pub trait CloudProvider: Send + Sync {
    /// Upload data to cloud
    fn upload(&self, entity: SyncEntity, id: &str, data: &[u8]) -> Result<String, String>;
    
    /// Download data from cloud
    fn download(&self, entity: SyncEntity, id: &str) -> Result<Vec<u8>, String>;
    
    /// List entities in cloud
    fn list(&self, entity: SyncEntity) -> Result<Vec<(String, String)>, String>;
    
    /// Delete from cloud
    fn delete(&self, entity: SyncEntity, id: &str) -> Result<(), String>;
    
    /// Get last sync timestamp
    fn get_last_sync(&self, entity: SyncEntity, id: &str) -> Result<Option<String>, String>;
}

/// Sync manager
pub struct SyncManager {
    /// Cloud provider
    provider: Option<Arc<dyn CloudProvider>>,
    /// Conflict resolver
    conflict_resolver: ConflictResolver,
    /// Pending operations
    pending: Arc<RwLock<Vec<PendingSyncOp>>>,
    /// Last sync timestamps (reserved for future auto-sync feature)
    #[allow(dead_code)]
    last_sync: Arc<RwLock<HashMap<(SyncEntity, String), DateTime<Utc>>>>,
    /// Auto sync interval (reserved for future auto-sync feature)
    #[allow(dead_code)]
    sync_interval: Duration,
}

/// Pending sync operation
#[derive(Debug, Clone)]
struct PendingSyncOp {
    entity: SyncEntity,
    id: String,
    operation: SyncOperation,
    data: Option<Vec<u8>>,
    /// Retry count (reserved for future retry logic)
    #[allow(dead_code)]
    retries: usize,
    /// Creation timestamp (reserved for future timeout logic)
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(provider: Option<Arc<dyn CloudProvider>>) -> Self {
        Self {
            provider,
            conflict_resolver: ConflictResolver::new(ConflictStrategy::Manual),
            pending: Arc::new(RwLock::new(Vec::new())),
            last_sync: Arc::new(RwLock::new(HashMap::new())),
            sync_interval: Duration::from_secs(300), // 5 minutes
        }
    }
    
    /// Set conflict resolution strategy
    pub fn set_conflict_strategy(&mut self, strategy: ConflictStrategy) {
        self.conflict_resolver = ConflictResolver::new(strategy);
    }
    
    /// Queue an upload operation
    pub async fn queue_upload(&self, entity: SyncEntity, id: String, data: Vec<u8>) {
        let op = PendingSyncOp {
            entity,
            id,
            operation: SyncOperation::Upload,
            data: Some(data),
            retries: 0,
            created_at: Utc::now(),
        };
        
        let mut pending = self.pending.write().await;
        pending.push(op);
    }
    
    /// Queue a download operation
    pub async fn queue_download(&self, entity: SyncEntity, id: String) {
        let op = PendingSyncOp {
            entity,
            id,
            operation: SyncOperation::Download,
            data: None,
            retries: 0,
            created_at: Utc::now(),
        };
        
        let mut pending = self.pending.write().await;
        pending.push(op);
    }
    
    /// Queue a delete operation
    pub async fn queue_delete(&self, entity: SyncEntity, id: String) {
        let op = PendingSyncOp {
            entity,
            id,
            operation: SyncOperation::Delete,
            data: None,
            retries: 0,
            created_at: Utc::now(),
        };
        
        let mut pending = self.pending.write().await;
        pending.push(op);
    }
    
    /// Perform sync now
    pub async fn sync_now(&self) -> SyncResult {
        let start = Instant::now();
        let provider = match &self.provider {
            Some(p) => p,
            None => {
                return SyncResult {
                    success: false,
                    uploaded: 0,
                    downloaded: 0,
                    conflicts: vec![],
                    errors: vec!["No cloud provider configured".to_string()],
                    duration_ms: 0,
                };
            }
        };
        
        let mut uploaded = 0;
        let mut downloaded = 0;
        let conflicts = Vec::new();
        let mut errors = Vec::new();
        
        // Process pending operations
        let pending_ops: Vec<PendingSyncOp> = {
            let mut pending = self.pending.write().await;
            std::mem::take(&mut *pending)
        };
        
        for op in pending_ops {
            let result = match op.operation {
                SyncOperation::Upload => {
                    if let Some(data) = &op.data {
                        provider.upload(op.entity.clone(), &op.id, data)
                            .map(|_| {
                                uploaded += 1;
                            })
                    } else {
                        Err("No data for upload".to_string())
                    }
                }
                SyncOperation::Download => {
                    provider.download(op.entity.clone(), &op.id)
                        .map(|_| {
                            downloaded += 1;
                        })
                }
                SyncOperation::Delete => {
                    provider.delete(op.entity.clone(), &op.id)
                        .map(|_| {
                            uploaded += 1;
                        })
                }
            };
            
            if let Err(e) = result {
                errors.push(format!("{:?} {} failed: {}", op.entity, op.id, e));
            }
        }
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        SyncResult {
            success: errors.is_empty(),
            uploaded,
            downloaded,
            conflicts,
            errors,
            duration_ms,
        }
    }
    
    /// Get pending operation count
    pub async fn pending_count(&self) -> usize {
        self.pending.read().await.len()
    }
    
    /// Check if sync is needed
    pub async fn needs_sync(&self) -> bool {
        !self.pending.read().await.is_empty()
    }
    
    /// Clear pending operations
    pub async fn clear_pending(&self) {
        let mut pending = self.pending.write().await;
        pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let manager = SyncManager::new(None);
        assert!(!manager.needs_sync().await);
    }

    #[tokio::test]
    async fn test_queue_operations() {
        let manager = SyncManager::new(None);
        
        manager.queue_upload(SyncEntity::Settings, "settings-1".to_string(), vec![1, 2, 3]).await;
        manager.queue_download(SyncEntity::Template, "template-1".to_string()).await;
        manager.queue_delete(SyncEntity::Conversation, "conv-1".to_string()).await;
        
        assert_eq!(manager.pending_count().await, 3);
    }

    #[tokio::test]
    async fn test_clear_pending() {
        let manager = SyncManager::new(None);
        
        manager.queue_upload(SyncEntity::Settings, "settings-1".to_string(), vec![]).await;
        manager.clear_pending().await;
        
        assert_eq!(manager.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_sync_without_provider() {
        let manager = SyncManager::new(None);
        manager.queue_upload(SyncEntity::Settings, "settings-1".to_string(), vec![]).await;
        
        let result = manager.sync_now().await;
        
        assert!(!result.success);
        assert!(result.errors.iter().any(|e| e.contains("No cloud provider")));
    }
}
