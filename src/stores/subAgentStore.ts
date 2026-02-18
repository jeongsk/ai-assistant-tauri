/**
 * Sub-agent Store - Zustand state management for sub-agents
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { SubAgent, SubAgentCreateInput, SubAgentUpdateInput, SubAgentType } from '../types/subagent';

interface SubAgentState {
  agents: SubAgent[];
  loading: boolean;
  error: string | null;

  // Actions
  loadAgents: () => Promise<void>;
  createAgent: (agent: SubAgentCreateInput) => Promise<void>;
  updateAgent: (agent: SubAgentUpdateInput) => Promise<void>;
  deleteAgent: (id: string) => Promise<void>;
  getAgent: (id: string) => SubAgent | undefined;
  assignTask: (id: string, task: string) => Promise<void>;
}

export const useSubAgentStore = create<SubAgentState>((set, get) => ({
  agents: [],
  loading: false,
  error: null,

  loadAgents: async () => {
    set({ loading: true, error: null });
    try {
      const rawAgents = await invoke<Array<{
        id: string;
        name: string;
        type: string;
        status: string;
        system_prompt: string | null;
        tools: string;
        config: string;
        task: string | null;
        result: string | null;
        error: string | null;
        created_at: string;
        completed_at: string | null;
      }>>('list_sub_agents');

      const agents: SubAgent[] = rawAgents.map((a) => ({
        id: a.id,
        name: a.name,
        type: a.type as SubAgentType,
        status: a.status as SubAgent['status'],
        systemPrompt: a.system_prompt || undefined,
        tools: JSON.parse(a.tools || '[]'),
        config: JSON.parse(a.config || '{}'),
        task: a.task || undefined,
        result: a.result || undefined,
        error: a.error || undefined,
        createdAt: a.created_at,
        completedAt: a.completed_at || undefined,
      }));

      set({ agents, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  createAgent: async (input: SubAgentCreateInput) => {
    try {
      await invoke('create_sub_agent', {
        id: input.id,
        name: input.name,
        type: input.type,
        systemPrompt: input.systemPrompt || null,
        tools: JSON.stringify(input.tools || []),
        config: JSON.stringify(input.config || {}),
      });

      await get().loadAgents();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateAgent: async (input: SubAgentUpdateInput) => {
    try {
      await invoke('update_sub_agent', {
        id: input.id,
        name: input.name,
        systemPrompt: input.systemPrompt,
        tools: input.tools ? JSON.stringify(input.tools) : null,
        config: input.config ? JSON.stringify(input.config) : null,
        status: input.status,
        task: input.task,
        result: input.result,
        error: input.error,
      });

      await get().loadAgents();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteAgent: async (id: string) => {
    try {
      await invoke('delete_sub_agent', { id });

      set((state) => ({
        agents: state.agents.filter((a) => a.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getAgent: (id: string) => {
    return get().agents.find((a) => a.id === id);
  },

  assignTask: async (id: string, task: string) => {
    try {
      await invoke('assign_sub_agent_task', { id, task });
      await get().loadAgents();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
