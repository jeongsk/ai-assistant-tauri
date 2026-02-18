/**
 * Integrations Store - Zustand store for external integrations
 */

import { create } from 'zustand';
import { DatabaseConfig, GitConfig, CloudStorageConfig, IntegrationStatus } from '../types/integration';

interface IntegrationsState {
  databases: DatabaseConfig[];
  gitRepos: GitConfig[];
  cloudStorages: CloudStorageConfig[];
  statuses: IntegrationStatus[];
  isLoading: boolean;
  error: string | null;

  // Actions
  addDatabase: (config: DatabaseConfig) => Promise<void>;
  removeDatabase: (name: string) => void;
  testDatabase: (config: DatabaseConfig) => Promise<boolean>;

  addGitRepo: (config: GitConfig) => Promise<void>;
  removeGitRepo: (path: string) => void;

  addCloudStorage: (config: CloudStorageConfig) => Promise<void>;
  removeCloudStorage: (bucket: string) => void;

  refreshStatuses: () => Promise<void>;
  clearError: () => void;
}

export const useIntegrationsStore = create<IntegrationsState>((set) => ({
  databases: [],
  gitRepos: [],
  cloudStorages: [],
  statuses: [],
  isLoading: false,
  error: null,

  addDatabase: async (config: DatabaseConfig) => {
    set({ isLoading: true, error: null });
    try {
      set(state => ({
        databases: [...state.databases, config],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  removeDatabase: (name: string) => {
    set(state => ({
      databases: state.databases.filter(d => d.database !== name),
    }));
  },

  testDatabase: async (config: DatabaseConfig) => {
    // In production, test actual connection
    console.log('Testing database:', config);
    return true;
  },

  addGitRepo: async (config: GitConfig) => {
    set({ isLoading: true, error: null });
    try {
      set(state => ({
        gitRepos: [...state.gitRepos, config],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  removeGitRepo: (path: string) => {
    set(state => ({
      gitRepos: state.gitRepos.filter(g => g.repositoryPath !== path),
    }));
  },

  addCloudStorage: async (config: CloudStorageConfig) => {
    set({ isLoading: true, error: null });
    try {
      set(state => ({
        cloudStorages: [...state.cloudStorages, config],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  removeCloudStorage: (bucket: string) => {
    set(state => ({
      cloudStorages: state.cloudStorages.filter(c => c.bucket !== bucket),
    }));
  },

  refreshStatuses: async () => {
    set({ isLoading: true, error: null });
    try {
      // In production, check actual connection status
      const statuses: IntegrationStatus[] = [];
      set({ statuses, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
