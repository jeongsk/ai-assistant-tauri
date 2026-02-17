/**
 * Settings Store
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface ProviderConfig {
  type: 'openai' | 'anthropic' | 'ollama';
  apiKey?: string;
  baseUrl?: string;
  model: string;
  enabled: boolean;
}

export interface FolderPermission {
  id: string;
  path: string;
  level: 'read' | 'readwrite';
}

interface SettingsState {
  // Providers
  providers: Record<string, ProviderConfig>;
  activeProvider: string;
  
  // Folder permissions
  folderPermissions: FolderPermission[];
  
  // UI
  theme: 'light' | 'dark' | 'system';
  
  // Actions
  setProvider: (id: string, config: ProviderConfig) => void;
  setActiveProvider: (id: string) => void;
  addFolderPermission: (permission: FolderPermission) => void;
  removeFolderPermission: (id: string) => void;
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
}

const DEFAULT_PROVIDERS: Record<string, ProviderConfig> = {
  openai: {
    type: 'openai',
    model: 'gpt-4o',
    enabled: true,
  },
  anthropic: {
    type: 'anthropic',
    model: 'claude-sonnet-4-20250514',
    enabled: true,
  },
  ollama: {
    type: 'ollama',
    baseUrl: 'http://localhost:11434',
    model: 'llama3.2',
    enabled: false,
  },
};

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      // Initial state
      providers: DEFAULT_PROVIDERS,
      activeProvider: 'anthropic',
      folderPermissions: [],
      theme: 'system',

      // Set provider config
      setProvider: (id, config) => {
        set((state) => ({
          providers: { ...state.providers, [id]: config },
        }));
      },

      // Set active provider
      setActiveProvider: (id) => {
        set({ activeProvider: id });
      },

      // Add folder permission
      addFolderPermission: (permission) => {
        set((state) => ({
          folderPermissions: [...state.folderPermissions, permission],
        }));
      },

      // Remove folder permission
      removeFolderPermission: (id) => {
        set((state) => ({
          folderPermissions: state.folderPermissions.filter((p) => p.id !== id),
        }));
      },

      // Set theme
      setTheme: (theme) => {
        set({ theme });
      },
    }),
    {
      name: 'ai-assistant-settings',
    }
  )
);
