//! Conflict Resolution
//! 
//! Handles conflicts between local and remote data during sync.

use serde::{Deserialize, Serialize};
use super::manager::SyncEntity;

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictStrategy {
    /// Local changes always win
    ClientWins,
    /// Remote changes always win
    ServerWins,
    /// Merge changes when possible
    Merge,
    /// Require manual resolution
    Manual,
}

/// Sync conflict representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    /// Entity type
    pub entity: SyncEntity,
    /// Entity ID
    pub id: String,
    /// Local version timestamp
    pub local_version: String,
    /// Remote version timestamp
    pub remote_version: String,
    /// Local data
    pub local_data: Vec<u8>,
    /// Remote data
    pub remote_data: Vec<u8>,
    /// Resolution (if resolved)
    pub resolution: Option<ConflictResolution>,
}

/// Resolution of a conflict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolution {
    /// Keep local version
    KeepLocal,
    /// Keep remote version
    KeepRemote,
    /// Use merged version
    Merged(Vec<u8>),
}

/// Conflict resolver
pub struct ConflictResolver {
    strategy: ConflictStrategy,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }
    
    /// Detect if there's a conflict
    pub fn detect(
        &self,
        entity: SyncEntity,
        id: &str,
        local_version: &str,
        remote_version: &str,
    ) -> Option<SyncConflict> {
        if local_version != remote_version {
            Some(SyncConflict {
                entity,
                id: id.to_string(),
                local_version: local_version.to_string(),
                remote_version: remote_version.to_string(),
                local_data: vec![],
                remote_data: vec![],
                resolution: None,
            })
        } else {
            None
        }
    }
    
    /// Resolve a conflict using the configured strategy
    pub fn resolve(&self, conflict: &mut SyncConflict) -> ConflictResolution {
        if let Some(resolution) = &conflict.resolution {
            return resolution.clone();
        }
        
        let resolution = match &self.strategy {
            ConflictStrategy::ClientWins => ConflictResolution::KeepLocal,
            ConflictStrategy::ServerWins => ConflictResolution::KeepRemote,
            ConflictStrategy::Merge => {
                // Attempt to merge
                self.merge(&conflict.local_data, &conflict.remote_data)
            }
            ConflictStrategy::Manual => {
                // Return conflict as unresolved
                return ConflictResolution::KeepLocal; // Default
            }
        };
        
        conflict.resolution = Some(resolution.clone());
        resolution
    }
    
    /// Merge two versions of data
    fn merge(&self, local: &[u8], remote: &[u8]) -> ConflictResolution {
        // Simple merge strategy: combine unique elements
        // In real implementation, this would be data-type specific
        
        if local.is_empty() {
            return ConflictResolution::KeepRemote;
        }
        if remote.is_empty() {
            return ConflictResolution::KeepLocal;
        }
        
        // Placeholder: just concatenate with a marker
        let mut merged = Vec::with_capacity(local.len() + remote.len() + 4);
        merged.extend_from_slice(b"[ML]");
        merged.extend_from_slice(local);
        merged.extend_from_slice(b"[MR]");
        merged.extend_from_slice(remote);
        
        ConflictResolution::Merged(merged)
    }
    
    /// Set the resolution strategy
    pub fn set_strategy(&mut self, strategy: ConflictStrategy) {
        self.strategy = strategy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_client_wins_strategy() {
        let resolver = ConflictResolver::new(ConflictStrategy::ClientWins);
        let mut conflict = SyncConflict {
            entity: SyncEntity::Settings,
            id: "test".to_string(),
            local_version: "v1".to_string(),
            remote_version: "v2".to_string(),
            local_data: vec![1, 2, 3],
            remote_data: vec![4, 5, 6],
            resolution: None,
        };
        
        let resolution = resolver.resolve(&mut conflict);
        assert_eq!(resolution, ConflictResolution::KeepLocal);
    }

    #[test]
    fn test_server_wins_strategy() {
        let resolver = ConflictResolver::new(ConflictStrategy::ServerWins);
        let mut conflict = SyncConflict {
            entity: SyncEntity::Settings,
            id: "test".to_string(),
            local_version: "v1".to_string(),
            remote_version: "v2".to_string(),
            local_data: vec![1, 2, 3],
            remote_data: vec![4, 5, 6],
            resolution: None,
        };
        
        let resolution = resolver.resolve(&mut conflict);
        assert_eq!(resolution, ConflictResolution::KeepRemote);
    }

    #[test]
    fn test_merge_strategy() {
        let resolver = ConflictResolver::new(ConflictStrategy::Merge);
        let mut conflict = SyncConflict {
            entity: SyncEntity::Settings,
            id: "test".to_string(),
            local_version: "v1".to_string(),
            remote_version: "v2".to_string(),
            local_data: vec![1, 2, 3],
            remote_data: vec![4, 5, 6],
            resolution: None,
        };
        
        let resolution = resolver.resolve(&mut conflict);
        assert!(matches!(resolution, ConflictResolution::Merged(_)));
    }

    #[test]
    fn test_pre_set_resolution() {
        let resolver = ConflictResolver::new(ConflictStrategy::ClientWins);
        let mut conflict = SyncConflict {
            entity: SyncEntity::Settings,
            id: "test".to_string(),
            local_version: "v1".to_string(),
            remote_version: "v2".to_string(),
            local_data: vec![],
            remote_data: vec![],
            resolution: Some(ConflictResolution::KeepRemote),
        };
        
        let resolution = resolver.resolve(&mut conflict);
        assert_eq!(resolution, ConflictResolution::KeepRemote);
    }
}
