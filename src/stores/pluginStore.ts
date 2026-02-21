/**
 * Plugin Store - Zustand store for plugin management
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { Plugin, PluginManifest } from '../types/plugin';

interface PluginState {
  plugins: Plugin[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadPlugins: () => Promise<void>;
  getPlugin: (id: string) => Promise<Plugin | undefined>;
  installPlugin: (manifest: PluginManifest) => Promise<void>;
  enablePlugin: (id: string) => Promise<void>;
  disablePlugin: (id: string) => Promise<void>;
  uninstallPlugin: (id: string) => Promise<void>;
  clearError: () => void;
}

export const usePluginStore = create<PluginState>((set, get) => ({
  plugins: [],
  isLoading: false,
  error: null,

  loadPlugins: async () => {
    set({ isLoading: true, error: null });
    try {
      const rawPlugins = await invoke<Array<{
        id: string;
        name: string;
        version: string;
        manifest: string;
        permissions: string;
        enabled: number;
        installed_at: string;
        updated_at: string;
      }>>('list_plugins');

      const plugins: Plugin[] = rawPlugins.map(p => ({
        id: p.id,
        name: p.name,
        version: p.version,
        manifest: p.manifest, // Keep as string (JSON serialized)
        permissions: JSON.parse(p.permissions),
        enabled: p.enabled === 1,
        installedAt: p.installed_at,
        updatedAt: p.updated_at,
      }));

      set({ plugins, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  getPlugin: async (id: string) => {
    try {
      const rawPlugin = await invoke<{
        id: string;
        name: string;
        version: string;
        manifest: string;
        permissions: string;
        enabled: number;
        installed_at: string;
        updated_at: string;
      }>('get_plugin', { id });

      return {
        id: rawPlugin.id,
        name: rawPlugin.name,
        version: rawPlugin.version,
        manifest: rawPlugin.manifest, // Keep as string
        permissions: JSON.parse(rawPlugin.permissions),
        enabled: rawPlugin.enabled === 1,
        installedAt: rawPlugin.installed_at,
        updatedAt: rawPlugin.updated_at,
      };
    } catch {
      return undefined;
    }
  },

  installPlugin: async (manifest: PluginManifest) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('install_plugin', {
        id: manifest.id,
        name: manifest.name,
        version: manifest.version,
        manifest: JSON.stringify(manifest),
        permissions: JSON.stringify(manifest.permissions || []),
      });

      // Reload plugins list after installation
      await get().loadPlugins();
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  enablePlugin: async (id: string) => {
    set({ error: null });
    try {
      await invoke('enable_plugin', { id });

      set(state => ({
        plugins: state.plugins.map(p =>
          p.id === id ? { ...p, enabled: true, updatedAt: new Date().toISOString() } : p
        ),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  disablePlugin: async (id: string) => {
    set({ error: null });
    try {
      await invoke('disable_plugin', { id });

      set(state => ({
        plugins: state.plugins.map(p =>
          p.id === id ? { ...p, enabled: false, updatedAt: new Date().toISOString() } : p
        ),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  uninstallPlugin: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('uninstall_plugin', { id });

      // Reload plugins list after uninstallation
      await get().loadPlugins();
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
