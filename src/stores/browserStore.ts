/**
 * Browser Store - Zustand state management for browser MCP settings
 */

import { create } from 'zustand';

export interface BrowserSettings {
  enabled: boolean;
  headless: boolean;
  defaultViewport: {
    width: number;
    height: number;
  };
  rateLimit: {
    maxActions: number;
    windowMs: number;
  };
  timeout: number;
  userAgent: string;
}

interface BrowserState {
  settings: BrowserSettings;
  loading: boolean;
  error: string | null;

  // Actions
  loadSettings: () => Promise<void>;
  updateSettings: (settings: Partial<BrowserSettings>) => Promise<void>;
  resetSettings: () => void;
}

const DEFAULT_SETTINGS: BrowserSettings = {
  enabled: true,
  headless: true,
  defaultViewport: {
    width: 1280,
    height: 720,
  },
  rateLimit: {
    maxActions: 100,
    windowMs: 60000, // 1 minute
  },
  timeout: 30000, // 30 seconds
  userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
};

export const useBrowserStore = create<BrowserState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  loading: false,
  error: null,

  loadSettings: async () => {
    set({ loading: true, error: null });
    try {
      // Load from localStorage for now
      // In a full implementation, this would load from Tauri backend
      const stored = localStorage.getItem('browserSettings');
      if (stored) {
        const parsed = JSON.parse(stored);
        set({ settings: { ...DEFAULT_SETTINGS, ...parsed }, loading: false });
      } else {
        set({ settings: DEFAULT_SETTINGS, loading: false });
      }
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  updateSettings: async (newSettings: Partial<BrowserSettings>) => {
    try {
      const currentSettings = get().settings;
      const updatedSettings = { ...currentSettings, ...newSettings };

      // Save to localStorage
      localStorage.setItem('browserSettings', JSON.stringify(updatedSettings));

      set({ settings: updatedSettings });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  resetSettings: () => {
    localStorage.removeItem('browserSettings');
    set({ settings: DEFAULT_SETTINGS });
  },
}));
