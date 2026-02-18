/**
 * Settings Store with SQLite persistence for folder permissions
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";
import { invoke } from "@tauri-apps/api/core";

export interface ProviderConfig {
  type: "openai" | "anthropic" | "ollama";
  apiKey?: string;
  baseUrl?: string;
  model: string;
  enabled: boolean;
}

export interface FolderPermission {
  id: string;
  path: string;
  level: "read" | "readwrite";
}

interface SettingsState {
  // Providers
  providers: Record<string, ProviderConfig>;
  activeProvider: string;

  // Folder permissions
  folderPermissions: FolderPermission[];
  folderPermissionsLoaded: boolean;

  // UI
  theme: "light" | "dark" | "system";

  // Actions
  setProvider: (id: string, config: ProviderConfig) => void;
  setActiveProvider: (id: string) => void;
  addFolderPermission: (permission: FolderPermission) => void;
  removeFolderPermission: (id: string) => void;
  setTheme: (theme: "light" | "dark" | "system") => void;

  // DB Actions
  loadFolderPermissions: () => Promise<void>;

  // Agent Runtime sync
  syncProvidersToAgent: () => Promise<void>;
}

const DEFAULT_PROVIDERS: Record<string, ProviderConfig> = {
  openai: {
    type: "openai",
    model: "gpt-4o",
    enabled: true,
  },
  anthropic: {
    type: "anthropic",
    model: "claude-sonnet-4-20250514",
    enabled: true,
  },
  ollama: {
    type: "ollama",
    baseUrl: "http://localhost:11434",
    model: "llama3.2",
    enabled: false,
  },
};

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      // Initial state
      providers: DEFAULT_PROVIDERS,
      activeProvider: "anthropic",
      folderPermissions: [],
      folderPermissionsLoaded: false,
      theme: "system",

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

      // Add folder permission (with DB persistence)
      addFolderPermission: (permission) => {
        // Save to DB
        invoke("add_folder_permission", {
          id: permission.id,
          path: permission.path,
          level: permission.level,
        }).catch((error) =>
          console.error("Failed to save folder permission:", error)
        );

        set((state) => ({
          folderPermissions: [...state.folderPermissions, permission],
        }));
      },

      // Remove folder permission (with DB persistence)
      removeFolderPermission: (id) => {
        // Delete from DB
        invoke("remove_folder_permission", { id }).catch((error) =>
          console.error("Failed to remove folder permission:", error)
        );

        set((state) => ({
          folderPermissions: state.folderPermissions.filter((p) => p.id !== id),
        }));
      },

      // Set theme
      setTheme: (theme) => {
        set({ theme });
      },

      // Load folder permissions from DB
      loadFolderPermissions: async () => {
        try {
          const permissions = await invoke<Array<{
            id: string;
            path: string;
            level: string;
            created_at: string;
          }>>("load_folder_permissions");

          set({
            folderPermissions: permissions.map((p) => ({
              id: p.id,
              path: p.path,
              level: p.level as "read" | "readwrite",
            })),
            folderPermissionsLoaded: true,
          });
        } catch (error) {
          console.error("Failed to load folder permissions:", error);
          set({ folderPermissionsLoaded: true });
        }
      },

      // Sync providers to Agent Runtime
      syncProvidersToAgent: async () => {
        const state = get();
        const { providers, activeProvider } = state;
        const providerArray = Object.entries(providers).map(([id, config]) => ({
          ...config,
          id,
        }));

        try {
          await invoke("configure_providers", {
            providers: providerArray,
            activeProvider,
          });
        } catch (error) {
          console.error("Failed to sync providers to agent:", error);
        }
      },
    }),
    {
      name: "ai-assistant-settings",
      // Only persist providers, activeProvider, and theme
      // Folder permissions are stored in SQLite
      partialize: (state) => ({
        providers: state.providers,
        activeProvider: state.activeProvider,
        theme: state.theme,
      }),
    },
  ),
);
