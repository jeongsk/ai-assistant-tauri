/**
 * Memory Store - Zustand store for memory management
 */

import { create } from 'zustand';
import { Memory, UserPattern, MemoryCreateInput, MemoryType } from '../types/memory';

interface MemoryState {
  memories: Memory[];
  patterns: UserPattern[];
  isLoading: boolean;
  error: string | null;

  // Actions
  loadMemories: () => Promise<void>;
  createMemory: (input: MemoryCreateInput) => Promise<void>;
  deleteMemory: (id: string) => Promise<void>;
  searchMemories: (query: string, type?: MemoryType) => Promise<Memory[]>;

  loadPatterns: () => Promise<void>;
  clearError: () => void;
}

export const useMemoryStore = create<MemoryState>((set, get) => ({
  memories: [],
  patterns: [],
  isLoading: false,
  error: null,

  loadMemories: async () => {
    set({ isLoading: true, error: null });
    try {
      // In production, this would call Tauri commands
      const memories: Memory[] = [];
      set({ memories, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  createMemory: async (input: MemoryCreateInput) => {
    set({ isLoading: true, error: null });
    try {
      const memory: Memory = {
        id: input.id,
        type: input.type,
        content: input.content,
        metadata: input.metadata,
        importance: input.importance ?? 0.5,
        createdAt: new Date().toISOString(),
      };

      set(state => ({
        memories: [...state.memories, memory],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  deleteMemory: async (id: string) => {
    set({ isLoading: true, error: null });
    try {
      set(state => ({
        memories: state.memories.filter(m => m.id !== id),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  searchMemories: async (query: string, type?: MemoryType) => {
    const memories = get().memories;
    const queryLower = query.toLowerCase();

    return memories.filter(m => {
      if (type && m.type !== type) return false;
      return m.content.toLowerCase().includes(queryLower);
    });
  },

  loadPatterns: async () => {
    set({ isLoading: true, error: null });
    try {
      const patterns: UserPattern[] = [];
      set({ patterns, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
