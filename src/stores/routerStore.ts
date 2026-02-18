/**
 * Router Store - Zustand state management for provider routing
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export type TaskType = 'coding' | 'creative' | 'analysis' | 'chat' | 'research' | 'planning';

export interface RoutingCondition {
  taskTypes?: TaskType[];
  maxTokens?: number;
  minTokens?: number;
  keywords?: string[];
  complexity?: 'simple' | 'medium' | 'complex';
}

export interface RoutingRule {
  id: string;
  name: string;
  description?: string;
  condition: RoutingCondition;
  provider: string;
  model?: string;
  priority: number;
  enabled: boolean;
}

interface RouterState {
  rules: RoutingRule[];
  fallbackChain: string[];
  loading: boolean;
  error: string | null;

  // Actions
  loadRules: () => Promise<void>;
  addRule: (rule: RoutingRule) => Promise<void>;
  updateRule: (id: string, updates: Partial<RoutingRule>) => Promise<void>;
  removeRule: (id: string) => Promise<void>;
  setFallbackChain: (chain: string[]) => Promise<void>;
  toggleRule: (id: string) => Promise<void>;
}

export const useRouterStore = create<RouterState>((set, get) => ({
  rules: [],
  fallbackChain: ['anthropic', 'openai', 'ollama'],
  loading: false,
  error: null,

  loadRules: async () => {
    set({ loading: true, error: null });
    try {
      const rawRules = await invoke<Array<{
        id: string;
        name: string;
        description: string | null;
        condition: string;
        provider: string;
        model: string | null;
        priority: number;
        enabled: number;
      }>>('list_routing_rules');

      const rules: RoutingRule[] = rawRules.map((r) => ({
        id: r.id,
        name: r.name,
        description: r.description || undefined,
        condition: JSON.parse(r.condition || '{}'),
        provider: r.provider,
        model: r.model || undefined,
        priority: r.priority,
        enabled: r.enabled !== 0,
      }));

      // Sort by priority
      rules.sort((a, b) => b.priority - a.priority);

      set({ rules, loading: false });
    } catch (error) {
      // If table doesn't exist yet, use defaults
      set({
        rules: [
          { id: 'rule-code', name: 'Code Generation', condition: { taskTypes: ['coding'] }, provider: 'anthropic', model: 'claude-3-sonnet', priority: 100, enabled: true },
          { id: 'rule-chat', name: 'Simple Chat', condition: { taskTypes: ['chat'], complexity: 'simple' }, provider: 'openai', model: 'gpt-3.5-turbo', priority: 90, enabled: true },
          { id: 'rule-analysis', name: 'Analysis', condition: { taskTypes: ['analysis'] }, provider: 'openai', model: 'gpt-4', priority: 85, enabled: true },
        ],
        loading: false
      });
    }
  },

  addRule: async (rule: RoutingRule) => {
    try {
      await invoke('create_routing_rule', {
        id: rule.id,
        name: rule.name,
        description: rule.description || null,
        condition: JSON.stringify(rule.condition),
        provider: rule.provider,
        model: rule.model || null,
        priority: rule.priority,
        enabled: rule.enabled ? 1 : 0,
      });

      const rules = [...get().rules, rule];
      rules.sort((a, b) => b.priority - a.priority);
      set({ rules });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateRule: async (id: string, updates: Partial<RoutingRule>) => {
    try {
      const rule = get().rules.find(r => r.id === id);
      if (!rule) return;

      const updated = { ...rule, ...updates };

      await invoke('update_routing_rule', {
        id,
        name: updated.name,
        description: updated.description || null,
        condition: JSON.stringify(updated.condition),
        provider: updated.provider,
        model: updated.model || null,
        priority: updated.priority,
        enabled: updated.enabled ? 1 : 0,
      });

      const rules = get().rules.map(r => r.id === id ? updated : r);
      rules.sort((a, b) => b.priority - a.priority);
      set({ rules });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  removeRule: async (id: string) => {
    try {
      await invoke('delete_routing_rule', { id });

      set((state) => ({
        rules: state.rules.filter(r => r.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  setFallbackChain: async (chain: string[]) => {
    set({ fallbackChain: chain });
    // Could persist to settings
  },

  toggleRule: async (id: string) => {
    const rule = get().rules.find(r => r.id === id);
    if (rule) {
      await get().updateRule(id, { enabled: !rule.enabled });
    }
  },
}));
