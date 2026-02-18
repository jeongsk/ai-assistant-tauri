// Permission Store - Zustand

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { FolderPermission, PermissionLevel, FileNode } from '../types/permission';
import { generateId, hasPermission } from '../types/permission';

interface PermissionStore {
  // State
  permissions: FolderPermission[];
  fileTree: FileNode[];
  isLoading: boolean;
  error: string | null;

  // Actions
  addPermission: (path: string, level: PermissionLevel) => Promise<void>;
  removePermission: (id: string) => Promise<void>;
  updatePermission: (id: string, level: PermissionLevel) => Promise<void>;
  loadPermissions: () => Promise<void>;
  checkAccess: (path: string, requiredLevel: PermissionLevel) => boolean;
  loadDirectory: (path: string) => Promise<FileNode[]>;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const usePermissionStore = create<PermissionStore>((set, get) => ({
  // Initial State
  permissions: [],
  fileTree: [],
  isLoading: false,
  error: null,

  // Add new folder permission
  addPermission: async (path, level) => {
    set({ isLoading: true, error: null });
    try {
      // Validate path exists
      const isValid = await invoke<boolean>('validate_folder_path', { path });
      if (!isValid) {
        throw new Error('Invalid folder path');
      }

      const permission: FolderPermission = {
        id: generateId(),
        path,
        level,
        createdAt: new Date(),
      };

      // Save to Rust backend
      await invoke('add_folder_permission', { permission });

      set((state) => ({
        permissions: [...state.permissions, permission],
        isLoading: false,
      }));
    } catch (error) {
      set({ isLoading: false, error: String(error) });
      throw error;
    }
  },

  // Remove folder permission
  removePermission: async (id) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('remove_folder_permission', { id });
      set((state) => ({
        permissions: state.permissions.filter((p) => p.id !== id),
        isLoading: false,
      }));
    } catch (error) {
      set({ isLoading: false, error: String(error) });
      throw error;
    }
  },

  // Update permission level
  updatePermission: async (id, level) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('update_folder_permission', { id, level });
      set((state) => ({
        permissions: state.permissions.map((p) =>
          p.id === id ? { ...p, level } : p
        ),
        isLoading: false,
      }));
    } catch (error) {
      set({ isLoading: false, error: String(error) });
      throw error;
    }
  },

  // Load permissions from storage
  loadPermissions: async () => {
    set({ isLoading: true, error: null });
    try {
      const permissions = await invoke<FolderPermission[]>('load_folder_permissions');
      set({ permissions: permissions || [], isLoading: false });
    } catch (error) {
      set({ isLoading: false, error: String(error) });
    }
  },

  // Check if path has required permission
  checkAccess: (path, requiredLevel) => {
    const { permissions } = get();
    return hasPermission(path, permissions, requiredLevel);
  },

  // Load directory contents
  loadDirectory: async (path) => {
    const { checkAccess } = get();

    // Check read permission
    if (!checkAccess(path, 'read')) {
      throw new Error('No read permission for this directory');
    }

    try {
      const entries = await invoke<string[]>('list_directory', { path });
      const nodes: FileNode[] = entries.map((entry) => {
        const [name, type] = entry.split(':');
        return {
          name,
          path: `${path}/${name}`,
          type: type as 'file' | 'dir',
          permission: checkAccess(`${path}/${name}`, 'readwrite') ? 'readwrite' : 'read',
        };
      });

      // Sort: directories first, then files
      nodes.sort((a, b) => {
        if (a.type === b.type) return a.name.localeCompare(b.name);
        return a.type === 'dir' ? -1 : 1;
      });

      return nodes;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  // Loading state
  setLoading: (loading) => set({ isLoading: loading }),

  // Error state
  setError: (error) => set({ error }),
}));
