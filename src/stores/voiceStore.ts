/**
 * Voice Store - Zustand store for voice settings
 */

import { create } from 'zustand';
import { VoiceSettings, VoiceCommand, DEFAULT_VOICE_SETTINGS } from '../types/voice';

interface VoiceState {
  settings: VoiceSettings;
  isListening: boolean;
  isProcessing: boolean;
  commands: VoiceCommand[];
  error: string | null;

  // Actions
  loadSettings: () => Promise<void>;
  updateSettings: (updates: Partial<VoiceSettings>) => Promise<void>;
  startListening: () => void;
  stopListening: () => void;
  addCommand: (command: VoiceCommand) => void;
  clearCommands: () => void;
  clearError: () => void;
}

export const useVoiceStore = create<VoiceState>((set, get) => ({
  settings: {
    id: 'default',
    ...DEFAULT_VOICE_SETTINGS,
    updatedAt: new Date().toISOString(),
  },
  isListening: false,
  isProcessing: false,
  commands: [],
  error: null,

  loadSettings: async () => {
    set({ error: null });
    try {
      // In production, load from Tauri
      const settings: VoiceSettings = {
        id: 'default',
        ...DEFAULT_VOICE_SETTINGS,
        updatedAt: new Date().toISOString(),
      };
      set({ settings });
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  updateSettings: async (updates) => {
    set({ error: null });
    try {
      set(state => ({
        settings: {
          ...state.settings,
          ...updates,
          updatedAt: new Date().toISOString(),
        },
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  startListening: () => set({ isListening: true }),
  stopListening: () => set({ isListening: false }),

  addCommand: (command) => {
    set(state => ({
      commands: [...state.commands, command],
    }));
  },

  clearCommands: () => set({ commands: [] }),
  clearError: () => set({ error: null }),
}));
