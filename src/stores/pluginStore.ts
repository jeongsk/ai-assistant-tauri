/**
 * Plugin Store - Zustand store for plugin management
 */

import { create } from 'zustand';
import { Plugin, PluginManifest } from '../types/plugin';

interface PluginState {
  plugins: Plugin[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadPlugins: () => Promise<void>;
  installPlugin: (manifest: PluginManifest) => Promise<void>;
  enablePlugin: (id: string) => Promise<void>;
  disablePlugin: (id: string) => Promise<void>;
  uninstallPlugin: (id: string) => Promise<void>;
  clearError: () => void;
}

export const usePluginStore = create<PluginState>((set) => ({
  plugins: [],
  isLoading: false,
  error: null,

  loadPlugins: async () => {
    set({ isLoading: true, error: null });
    try {
      // In production, load from Tauri
      const plugins: Plugin[] = [];
      set({ plugins, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  installPlugin: async (manifest: PluginManifest) => {
    set({ isLoading: true, error: null });
    try {
      const plugin: Plugin = {
        id: manifest.id,
        name: manifest.name,
        version: manifest.version,
        manifest: JSON.stringify(manifest),
        permissions: [],
        enabled: false,
        installedAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      set(state => ({
        plugins: [...state.plugins, plugin],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  enablePlugin: async (id: string) => {
    set({ error: null });
    try {
      set(state => ({
        plugins: state.plugins.map(p =>
          p.id === id ? { ...p, enabled: true, updatedAt: new Date().toISOString() } : p
        ),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  disablePlugin: async (id: string) => {
    set({ error: null });
    try {
      set(state => ({
        plugins: state.plugins.map(p =>
          p.id === id ? { ...p, enabled: false, updatedAt: new Date().toISOString() } : p
        ),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  uninstallPlugin: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      set(state => ({
        plugins: state.plugins.filter(p => p.id !== id),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
