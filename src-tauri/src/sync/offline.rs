//! Offline Queue Management
//! 
//! Manages pending operations when offline.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::manager::SyncOperation;
use super::SyncEntity;

/// Pending operation in the offline queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOperation {
    /// Unique ID for this operation
    pub id: String,
    /// Entity being synced
    pub entity: SyncEntity,
    /// Entity ID
    pub entity_id: String,
    /// Operation type
    pub operation: SyncOperation,
    /// Data payload (for uploads)
    pub data: Option<Vec<u8>>,
    /// Number of retry attempts
    pub attempts: usize,
    /// Maximum retry attempts
    pub max_attempts: usize,
    /// Last error message
    pub last_error: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last attempt timestamp
    pub last_attempt: Option<DateTime<Utc>>,
}

/// Offline queue for pending sync operations
pub struct OfflineQueue {
    operations: Vec<PendingOperation>,
    max_size: usize,
    retry_delay_minutes: i64,
}

impl OfflineQueue {
    /// Create a new offline queue
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            max_size: 1000,
            retry_delay_minutes: 5,
        }
    }
    
    /// Create with custom settings
    pub fn with_settings(max_size: usize, retry_delay_minutes: i64) -> Self {
        Self {
            operations: Vec::new(),
            max_size,
            retry_delay_minutes,
        }
    }
    
    /// Add an operation to the queue
    pub fn push(&mut self, operation: PendingOperation) -> Result<(), String> {
        if self.operations.len() >= self.max_size {
            // Remove oldest operation
            self.operations.remove(0);
        }
        
        self.operations.push(operation);
        Ok(())
    }
    
    /// Get next operation ready for retry
    pub fn pop_ready(&mut self) -> Option<PendingOperation> {
        let now = Utc::now();
        let retry_delay = chrono::Duration::minutes(self.retry_delay_minutes);
        
        // Find first operation ready for retry
        let pos = self.operations.iter().position(|op| {
            if op.attempts >= op.max_attempts {
                return false;
            }
            
            match op.last_attempt {
                None => true,
                Some(last) => now - last >= retry_delay,
            }
        })?;
        
        Some(self.operations.remove(pos))
    }
    
    /// Peek at next operation without removing
    pub fn peek(&self) -> Option<&PendingOperation> {
        self.operations.first()
    }
    
    /// Mark an operation as failed (re-add for retry)
    pub fn mark_failed(&mut self, mut operation: PendingOperation, error: String) {
        operation.attempts += 1;
        operation.last_error = Some(error);
        operation.last_attempt = Some(Utc::now());
        
        if operation.attempts < operation.max_attempts {
            self.operations.push(operation);
        }
    }
    
    /// Remove all operations for an entity
    pub fn remove_for_entity(&mut self, entity: &SyncEntity, entity_id: &str) -> usize {
        let before = self.operations.len();
        self.operations.retain(|op| {
            op.entity != *entity || op.entity_id != entity_id
        });
        before - self.operations.len()
    }
    
    /// Get queue length
    pub fn len(&self) -> usize {
        self.operations.len()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
    
    /// Clear all operations
    pub fn clear(&mut self) {
        self.operations.clear();
    }
    
    /// Get operations by entity type
    pub fn get_by_entity(&self, entity: &SyncEntity) -> Vec<&PendingOperation> {
        self.operations.iter()
            .filter(|op| op.entity == *entity)
            .collect()
    }
    
    /// Get failed operations (max attempts reached)
    pub fn get_failed(&self) -> Vec<&PendingOperation> {
        self.operations.iter()
            .filter(|op| op.attempts >= op.max_attempts)
            .collect()
    }
}

impl Default for OfflineQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_operation(id: &str) -> PendingOperation {
        PendingOperation {
            id: id.to_string(),
            entity: SyncEntity::Settings,
            entity_id: "test".to_string(),
            operation: SyncOperation::Upload,
            data: Some(vec![1, 2, 3]),
            attempts: 0,
            max_attempts: 3,
            last_error: None,
            created_at: Utc::now(),
            last_attempt: None,
        }
    }

    #[test]
    fn test_queue_push_pop() {
        let mut queue = OfflineQueue::new();
        
        queue.push(make_operation("op-1")).unwrap();
        queue.push(make_operation("op-2")).unwrap();
        
        assert_eq!(queue.len(), 2);
        
        let op = queue.pop_ready();
        assert!(op.is_some());
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_queue_peek() {
        let mut queue = OfflineQueue::new();
        queue.push(make_operation("op-1")).unwrap();
        
        let op = queue.peek();
        assert!(op.is_some());
        assert_eq!(queue.len(), 1); // Not removed
    }

    #[test]
    fn test_mark_failed() {
        let mut queue = OfflineQueue::new();
        let op = make_operation("op-1");
        
        queue.mark_failed(op, "Network error".to_string());
        
        assert_eq!(queue.len(), 1);
        let queued = queue.peek().unwrap();
        assert_eq!(queued.attempts, 1);
        assert_eq!(queued.last_error, Some("Network error".to_string()));
    }

    #[test]
    fn test_max_attempts() {
        let mut queue = OfflineQueue::new();
        let mut op = make_operation("op-1");
        op.attempts = 2; // Already at max - 1
        
        queue.mark_failed(op, "Final error".to_string());
        
        // Should not be re-added
        assert!(queue.is_empty());
    }

    #[test]
    fn test_remove_for_entity() {
        let mut queue = OfflineQueue::new();
        
        queue.push(make_operation("op-1")).unwrap();
        queue.push(make_operation("op-2")).unwrap();
        
        let removed = queue.remove_for_entity(&SyncEntity::Settings, "test");
        
        assert_eq!(removed, 2);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_get_failed() {
        let mut queue = OfflineQueue::new();
        
        let mut op1 = make_operation("op-1");
        op1.attempts = 3; // Max reached
        
        let op2 = make_operation("op-2");
        
        queue.push(op1).unwrap();
        queue.push(op2).unwrap();
        
        let failed = queue.get_failed();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].id, "op-1");
    }
}
