/**
 * Voice Store - Zustand store for voice settings
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { VoiceSettings, VoiceCommand, DEFAULT_VOICE_SETTINGS } from '../types/voice';

interface VoiceState {
  settings: VoiceSettings;
  isListening: boolean;
  isProcessing: boolean;
  commands: VoiceCommand[];
  sttInitialized: boolean;
  ttsInitialized: boolean;
  availableModels: string[];
  availableVoices: Array<{ id: string; name: string; language: string; gender: string }>;
  error: string | null;

  // Actions
  loadSettings: () => Promise<void>;
  updateSettings: (updates: Partial<VoiceSettings>) => Promise<void>;
  initStt: () => Promise<void>;
  initTts: () => Promise<void>;
  transcribe: (audioData: ArrayBuffer) => Promise<{ text: string; confidence: number; language: string }>;
  synthesize: (text: string) => Promise<ArrayBuffer>;
  loadAvailableModels: () => Promise<void>;
  loadAvailableVoices: () => Promise<void>;
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
  sttInitialized: false,
  ttsInitialized: false,
  availableModels: [],
  availableVoices: [],
  error: null,

  loadSettings: async () => {
    set({ error: null });
    try {
      const settings = await invoke<{
        id: string;
        enabled: number;
        stt_model: string;
        tts_voice: string;
        language: string;
        wake_word: string | null;
        vad_sensitivity: number;
        updated_at: string;
      } | null>('get_voice_settings');

      if (settings) {
        set({
          settings: {
            id: settings.id,
            enabled: settings.enabled === 1,
            sttModel: settings.stt_model,
            ttsVoice: settings.tts_voice,
            language: settings.language,
            wakeWord: settings.wake_word ?? undefined,
            vadSensitivity: settings.vad_sensitivity,
            updatedAt: settings.updated_at, // Use string from DB
          },
        });
      }
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  updateSettings: async (updates) => {
    set({ error: null });
    try {
      const current = get().settings;

      await invoke('update_voice_settings', {
        id: current.id,
        enabled: updates.enabled ?? current.enabled,
        sttModel: updates.sttModel ?? current.sttModel,
        ttsVoice: updates.ttsVoice ?? current.ttsVoice,
        language: updates.language ?? current.language,
        wakeWord: updates.wakeWord ?? current.wakeWord ?? null,
        vadSensitivity: updates.vadSensitivity ?? current.vadSensitivity,
      });

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

  initStt: async () => {
    set({ error: null });
    try {
      const result = await invoke<string>('init_stt', { model: get().settings.sttModel });
      set({ sttInitialized: true });
      console.log('[Voice] STT initialized:', result);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  initTts: async () => {
    set({ error: null });
    try {
      const result = await invoke<string>('init_tts', { voice: get().settings.ttsVoice });
      set({ ttsInitialized: true });
      console.log('[Voice] TTS initialized:', result);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  transcribe: async (audioData: ArrayBuffer) => {
    set({ error: null, isProcessing: true });
    try {
      const result = await invoke<{
        text: string;
        confidence: number;
        language: string;
        duration_ms: number;
      }>('voice_transcribe', {
        audioData: Array.from(new Uint8Array(audioData)),
        language: get().settings.language,
      });

      return {
        text: result.text,
        confidence: result.confidence,
        language: result.language,
      };
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isProcessing: false });
    }
  },

  synthesize: async (text: string) => {
    set({ error: null, isProcessing: true });
    try {
      const result = await invoke<{
        audio_data: number[];
        sample_rate: number;
        duration_ms: number;
      }>('voice_synthesize', {
        text,
        language: get().settings.language,
      });

      // Convert number array back to ArrayBuffer
      const audioData = new Uint8Array(result.audio_data).buffer;
      return audioData;
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ isProcessing: false });
    }
  },

  loadAvailableModels: async () => {
    try {
      const models = await invoke<string[]>('get_available_models');
      set({ availableModels: models });
    } catch (error) {
      console.error('[Voice] Failed to load models:', error);
    }
  },

  loadAvailableVoices: async () => {
    try {
      const voices = await invoke<Array<{
        id: string;
        name: string;
        language: string;
        gender: string;
      }>>('voice_get_available_voices');
      set({ availableVoices: voices });
    } catch (error) {
      console.error('[Voice] Failed to load voices:', error);
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
