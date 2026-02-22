/**
 * Sync Store - Zustand state management for v0.6 sync features
 *
 * Handles cloud synchronization, conflict resolution, and offline queue.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type {
  SyncEntity,
  SyncOperation,
  SyncResult,
  ConflictStrategy,
  SyncConflict,
  ConflictResolution,
  PendingOperation,
} from '../types/sync';

interface SyncState {
  // State
  pendingCount: number;
  needsSync: boolean;
  lastSyncResult: SyncResult | null;
  offlineQueue: PendingOperation[];
  failedOperations: PendingOperation[];

  // Loading state
  syncing: boolean;
  loading: boolean;

  // Error state
  error: string | null;

  // Sync manager actions
  syncNow: () => Promise<SyncResult>;
  queueUpload: (entity: SyncEntity, id: string, data: number[]) => Promise<void>;
  queueDownload: (entity: SyncEntity, id: string) => Promise<void>;
  queueDelete: (entity: SyncEntity, id: string) => Promise<void>;
  getPendingCount: () => Promise<number>;
  checkNeedsSync: () => Promise<boolean>;
  clearPending: () => Promise<void>;
  setConflictStrategy: (strategy: ConflictStrategy) => Promise<void>;

  // Conflict resolution actions
  detectConflict: (
    entity: SyncEntity,
    id: string,
    localVersion: string,
    remoteVersion: string
  ) => Promise<SyncConflict | null>;
  resolveConflict: (
    entity: SyncEntity,
    id: string,
    localVersion: string,
    remoteVersion: string,
    localData: number[],
    remoteData: number[],
    resolution: ConflictResolution
  ) => Promise<ConflictResolution>;

  // Offline queue actions
  pushToQueue: (
    id: string,
    entity: SyncEntity,
    entityId: string,
    operation: SyncOperation,
    data?: number[]
  ) => Promise<void>;
  popReadyFromQueue: () => Promise<PendingOperation | null>;
  peekQueue: () => Promise<PendingOperation | null>;
  markFailed: (operation: PendingOperation, error: string) => Promise<void>;
  getQueueLength: () => Promise<number>;
  clearQueue: () => Promise<void>;
  getFailedOperations: () => Promise<PendingOperation[]>;
  getOperationsByEntity: (entity: SyncEntity) => Promise<PendingOperation[]>;
}

export const useSyncStore = create<SyncState>((set, get) => ({
  // Initial state
  pendingCount: 0,
  needsSync: false,
  lastSyncResult: null,
  offlineQueue: [],
  failedOperations: [],
  syncing: false,
  loading: false,
  error: null,

  // Sync manager actions
  syncNow: async () => {
    set({ syncing: true, error: null });
    try {
      const result = await invoke<SyncResult>('sync_now');
      set({ lastSyncResult: result, syncing: false });

      // Update pending count and needs sync
      await get().getPendingCount();
      await get().checkNeedsSync();

      return result;
    } catch (error) {
      set({ error: String(error), syncing: false });
      throw error;
    }
  },

  queueUpload: async (entity: SyncEntity, id: string, data: number[]) => {
    try {
      await invoke('sync_queue_upload', { entity, id, data });

      // Update state
      await get().getPendingCount();
      await get().checkNeedsSync();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  queueDownload: async (entity: SyncEntity, id: string) => {
    try {
      await invoke('sync_queue_download', { entity, id });

      // Update state
      await get().getPendingCount();
      await get().checkNeedsSync();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  queueDelete: async (entity: SyncEntity, id: string) => {
    try {
      await invoke('sync_queue_delete', { entity, id });

      // Update state
      await get().getPendingCount();
      await get().checkNeedsSync();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getPendingCount: async () => {
    try {
      const count = await invoke<number>('sync_pending_count');
      set({ pendingCount: count });
      return count;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  checkNeedsSync: async () => {
    try {
      const needs = await invoke<boolean>('sync_needs_sync');
      set({ needsSync: needs });
      return needs;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  clearPending: async () => {
    try {
      await invoke('sync_clear_pending');

      set({ pendingCount: 0, needsSync: false });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  setConflictStrategy: async (strategy: ConflictStrategy) => {
    try {
      await invoke('sync_set_conflict_strategy', { strategy });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  // Conflict resolution actions
  detectConflict: async (
    entity: SyncEntity,
    id: string,
    localVersion: string,
    remoteVersion: string
  ) => {
    try {
      const conflict = await invoke<SyncConflict | null>('sync_detect_conflict', {
        entity,
        id,
        localVersion,
        remoteVersion,
      });
      return conflict;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  resolveConflict: async (
    entity: SyncEntity,
    id: string,
    localVersion: string,
    remoteVersion: string,
    localData: number[],
    remoteData: number[],
    resolution: ConflictResolution
  ) => {
    try {
      const result = await invoke<ConflictResolution>('sync_resolve_conflict', {
        entity,
        id,
        localVersion,
        remoteVersion,
        localData,
        remoteData,
        resolution,
      });
      return result;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  // Offline queue actions
  pushToQueue: async (
    id: string,
    entity: SyncEntity,
    entityId: string,
    operation: SyncOperation,
    data?: number[]
  ) => {
    try {
      await invoke('sync_offline_push', {
        id,
        entity,
        entityId,
        operation,
        data,
      });

      // Refresh queue
      await get().getQueueLength();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  popReadyFromQueue: async () => {
    try {
      const operation = await invoke<PendingOperation | null>('sync_offline_pop_ready');

      if (operation) {
        set((state) => ({
          offlineQueue: state.offlineQueue.filter((op) => op.id !== operation.id),
        }));
      }

      // Refresh queue length
      await get().getQueueLength();

      return operation;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  peekQueue: async () => {
    try {
      const operation = await invoke<PendingOperation | null>('sync_offline_peek');
      return operation;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  markFailed: async (operation: PendingOperation, error: string) => {
    try {
      await invoke('sync_offline_mark_failed', { operation, error });

      // Refresh failed operations
      await get().getFailedOperations();
    } catch (err) {
      set({ error: String(err) });
      throw err;
    }
  },

  getQueueLength: async () => {
    try {
      const length = await invoke<number>('sync_offline_length');
      return length;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  clearQueue: async () => {
    try {
      await invoke('sync_offline_clear');
      set({ offlineQueue: [] });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getFailedOperations: async () => {
    try {
      const failed = await invoke<PendingOperation[]>('sync_offline_get_failed');
      set({ failedOperations: failed });
      return failed;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getOperationsByEntity: async (entity: SyncEntity) => {
    try {
      const operations = await invoke<PendingOperation[]>('sync_offline_get_by_entity', { entity });
      return operations;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
