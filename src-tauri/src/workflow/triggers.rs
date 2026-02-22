//! Workflow Trigger System
//! 
//! Manages triggers that start workflow executions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trigger types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    /// Schedule-based trigger (cron)
    Schedule,
    /// Webhook trigger
    Webhook,
    /// File system event trigger
    FileSystem,
    /// Voice command trigger
    Voice,
    /// Manual trigger
    Manual,
}

/// Trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trigger {
    /// Schedule trigger with cron expression
    Schedule {
        cron: String,
        timezone: String,
    },
    /// Webhook trigger
    Webhook {
        path: String,
        method: HttpMethod,
    },
    /// File system trigger
    FileSystem {
        path: String,
        events: Vec<FsEvent>,
    },
    /// Voice command trigger
    Voice {
        pattern: String,
        language: String,
    },
    /// Manual trigger (no config needed)
    Manual,
}

/// HTTP methods for webhook triggers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

/// File system events to watch
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FsEvent {
    Create,
    Modify,
    Delete,
    Rename,
}

/// Handle to an active trigger
#[derive(Debug, Clone)]
pub struct TriggerHandle {
    pub trigger_id: String,
    pub workflow_id: String,
    pub trigger_type: TriggerType,
}

/// Trigger manager for coordinating workflow triggers
pub struct TriggerManager {
    /// Active triggers by ID
    active_triggers: Arc<RwLock<HashMap<String, TriggerHandle>>>,
}

impl TriggerManager {
    /// Create a new trigger manager
    pub fn new() -> Self {
        Self {
            active_triggers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a trigger for a workflow
    pub async fn register(
        &self,
        trigger_id: String,
        workflow_id: String,
        trigger: &Trigger,
    ) -> Result<(), String> {
        let trigger_type = match trigger {
            Trigger::Schedule { .. } => TriggerType::Schedule,
            Trigger::Webhook { .. } => TriggerType::Webhook,
            Trigger::FileSystem { .. } => TriggerType::FileSystem,
            Trigger::Voice { .. } => TriggerType::Voice,
            Trigger::Manual => TriggerType::Manual,
        };
        
        let handle = TriggerHandle {
            trigger_id: trigger_id.clone(),
            workflow_id,
            trigger_type,
        };
        
        let mut triggers = self.active_triggers.write().await;
        triggers.insert(trigger_id, handle);
        
        Ok(())
    }
    
    /// Unregister a trigger
    pub async fn unregister(&self, trigger_id: &str) -> Result<(), String> {
        let mut triggers = self.active_triggers.write().await;
        triggers.remove(trigger_id)
            .map(|_| ())
            .ok_or_else(|| "Trigger not found".to_string())
    }
    
    /// Get all active triggers
    pub async fn list_active(&self) -> Vec<TriggerHandle> {
        let triggers = self.active_triggers.read().await;
        triggers.values().cloned().collect()
    }
    
    /// Check if a trigger exists
    pub async fn exists(&self, trigger_id: &str) -> bool {
        let triggers = self.active_triggers.read().await;
        triggers.contains_key(trigger_id)
    }
    
    /// Get trigger count
    pub async fn count(&self) -> usize {
        let triggers = self.active_triggers.read().await;
        triggers.len()
    }
}

impl Default for TriggerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trigger_manager_register() {
        let manager = TriggerManager::new();
        let trigger = Trigger::Schedule {
            cron: "0 0 * * *".to_string(),
            timezone: "UTC".to_string(),
        };
        
        manager.register("trigger-1".to_string(), "workflow-1".to_string(), &trigger).await.unwrap();
        
        assert_eq!(manager.count().await, 1);
    }

    #[tokio::test]
    async fn test_trigger_manager_unregister() {
        let manager = TriggerManager::new();
        let trigger = Trigger::Manual;
        
        manager.register("trigger-1".to_string(), "workflow-1".to_string(), &trigger).await.unwrap();
        manager.unregister("trigger-1").await.unwrap();
        
        assert_eq!(manager.count().await, 0);
    }

    #[tokio::test]
    async fn test_trigger_types() {
        let schedule = Trigger::Schedule {
            cron: "* * * * *".to_string(),
            timezone: "UTC".to_string(),
        };
        let webhook = Trigger::Webhook {
            path: "/api/trigger".to_string(),
            method: HttpMethod::Post,
        };
        let fs = Trigger::FileSystem {
            path: "/watch/dir".to_string(),
            events: vec![FsEvent::Modify],
        };
        let voice = Trigger::Voice {
            pattern: "start workflow".to_string(),
            language: "en".to_string(),
        };
        
        // Just verify they can be created
        assert!(matches!(schedule, Trigger::Schedule { .. }));
        assert!(matches!(webhook, Trigger::Webhook { .. }));
        assert!(matches!(fs, Trigger::FileSystem { .. }));
        assert!(matches!(voice, Trigger::Voice { .. }));
    }
}
