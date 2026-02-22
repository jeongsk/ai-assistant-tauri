/**
 * Sync Types for v0.6
 *
 * Types for cloud synchronization, conflict resolution, and offline queue.
 */

// ============================================================================
// Sync Manager Types
// ============================================================================

export enum SyncEntity {
  Settings = 'settings',
  Conversation = 'conversation',
  Template = 'template',
  Skill = 'skill',
  Recipe = 'recipe',
}

export enum SyncOperation {
  Upload = 'upload',
  Download = 'download',
  Delete = 'delete',
}

export interface SyncResult {
  success: boolean;
  uploaded: number;
  downloaded: number;
  conflicts: SyncConflict[];
  errors: string[];
  durationMs: number;
}

// ============================================================================
// Conflict Resolution Types
// ============================================================================

export enum ConflictStrategy {
  ClientWins = 'client_wins',
  ServerWins = 'server_wins',
  Merge = 'merge',
  Manual = 'manual',
}

export enum ConflictResolution {
  KeepLocal = 'keep_local',
  KeepRemote = 'keep_remote',
  Merged = 'merged',
}

export interface SyncConflict {
  entity: SyncEntity;
  id: string;
  localVersion: string;
  remoteVersion: string;
  localData: number[];
  remoteData: number[];
  resolution?: ConflictResolution;
}

// ============================================================================
// Offline Queue Types
// ============================================================================

export interface PendingOperation {
  id: string;
  entity: SyncEntity;
  entityId: string;
  operation: SyncOperation;
  data?: number[];
  attempts: number;
  maxAttempts: number;
  lastError?: string;
  createdAt: string;
  lastAttempt?: string;
}
