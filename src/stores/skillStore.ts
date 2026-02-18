/**
 * Skill Store - Zustand state management for skills
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Skill, SkillCreateInput, SkillUpdateInput } from '../types/skill';

interface SkillState {
  skills: Skill[];
  loading: boolean;
  error: string | null;

  // Actions
  loadSkills: () => Promise<void>;
  createSkill: (skill: SkillCreateInput) => Promise<void>;
  updateSkill: (skill: SkillUpdateInput) => Promise<void>;
  deleteSkill: (id: string) => Promise<void>;
  searchSkills: (query: string) => Promise<Skill[]>;
  getSkill: (id: string) => Skill | undefined;
}

export const useSkillStore = create<SkillState>((set, get) => ({
  skills: [],
  loading: false,
  error: null,

  loadSkills: async () => {
    set({ loading: true, error: null });
    try {
      const rawSkills = await invoke<Array<{
        id: string;
        name: string;
        description: string;
        prompt: string;
        tools: string;
        created_at: string;
        updated_at: string;
      }>>('list_skills');

      const skills: Skill[] = rawSkills.map((s) => ({
        ...s,
        tools: JSON.parse(s.tools || '[]'),
      }));

      set({ skills, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  createSkill: async (input: SkillCreateInput) => {
    try {
      await invoke('create_skill', {
        id: input.id,
        name: input.name,
        description: input.description,
        prompt: input.prompt,
        tools: JSON.stringify(input.tools),
      });

      // Reload skills
      await get().loadSkills();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateSkill: async (input: SkillUpdateInput) => {
    try {
      await invoke('update_skill', {
        id: input.id,
        name: input.name,
        description: input.description,
        prompt: input.prompt,
        tools: JSON.stringify(input.tools),
      });

      // Reload skills
      await get().loadSkills();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteSkill: async (id: string) => {
    try {
      await invoke('delete_skill', { id });

      set((state) => ({
        skills: state.skills.filter((s) => s.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  searchSkills: async (query: string) => {
    try {
      const rawSkills = await invoke<Array<{
        id: string;
        name: string;
        description: string;
        prompt: string;
        tools: string;
        created_at: string;
        updated_at: string;
      }>>('search_skills', { query });

      return rawSkills.map((s) => ({
        ...s,
        tools: JSON.parse(s.tools || '[]'),
      }));
    } catch (error) {
      set({ error: String(error) });
      return [];
    }
  },

  getSkill: (id: string) => {
    return get().skills.find((s) => s.id === id);
  },
}));
