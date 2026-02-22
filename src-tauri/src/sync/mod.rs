//! Cloud Synchronization Module
//! 
//! Provides synchronization between local data and cloud storage:
//! - Settings sync
//! - Conversation history sync
//! - Template sync
//! - Offline support with conflict resolution

pub mod manager;
pub mod conflict;
pub mod offline;

pub use manager::{SyncManager, SyncOperation, SyncEntity, SyncResult};
pub use conflict::{ConflictResolver, ConflictStrategy, SyncConflict};
pub use offline::{OfflineQueue, PendingOperation};
