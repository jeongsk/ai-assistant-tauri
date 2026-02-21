/**
 * Collaboration Store - Zustand store for templates and sharing
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { Template, SharedWorkflow, ExportOptions, ConversationExport } from '../types/collaboration';

interface CollaborationState {
  templates: Template[];
  workflows: SharedWorkflow[];
  isLoading: boolean;
  error: string | null;

  // Template Actions
  loadTemplates: () => Promise<void>;
  getTemplate: (id: string) => Promise<Template | undefined>;
  createTemplate: (template: Omit<Template, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>;
  updateTemplate: (id: string, updates: Partial<Omit<Template, 'id' | 'createdAt' | 'updatedAt'>>) => Promise<void>;
  deleteTemplate: (id: string) => Promise<void>;
  searchTemplates: (query: string) => Promise<Template[]>;

  // Workflow Actions
  loadWorkflows: () => Promise<void>;
  createWorkflow: (workflow: Omit<SharedWorkflow, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>;
  deleteWorkflow: (id: string) => Promise<void>;

  // Export/Import
  exportConversations: (conversations: ConversationExport[], options: ExportOptions) => Promise<Blob>;
  importConversations: (file: File) => Promise<ConversationExport[]>;

  clearError: () => void;
}

export const useCollaborationStore = create<CollaborationState>((set, get) => ({
  templates: [],
  workflows: [],
  isLoading: false,
  error: null,

  loadTemplates: async () => {
    set({ isLoading: true, error: null });
    try {
      const rawTemplates = await invoke<Array<{
        id: string;
        name: string;
        category: string;
        content: string;
        visibility: string;
        version: string;
        created_at: string;
        updated_at: string;
      }>>('list_templates');

      const templates: Template[] = rawTemplates.map(t => ({
        id: t.id,
        name: t.name,
        category: t.category,
        content: t.content,
        visibility: t.visibility as 'private' | 'public' | 'team',
        version: t.version,
        createdAt: t.created_at,
        updatedAt: t.updated_at,
      }));

      set({ templates, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  getTemplate: async (id: string) => {
    try {
      const rawTemplate = await invoke<{
        id: string;
        name: string;
        category: string;
        content: string;
        visibility: string;
        version: string;
        created_at: string;
        updated_at: string;
      }>('get_template', { id });

      return {
        id: rawTemplate.id,
        name: rawTemplate.name,
        category: rawTemplate.category,
        content: rawTemplate.content,
        visibility: rawTemplate.visibility as 'private' | 'public' | 'team',
        version: rawTemplate.version,
        createdAt: rawTemplate.created_at,
        updatedAt: rawTemplate.updated_at,
      };
    } catch {
      return undefined;
    }
  },

  createTemplate: async (input) => {
    set({ isLoading: true, error: null });
    try {
      const id = `tpl-${Date.now()}`;

      await invoke('create_template', {
        id,
        name: input.name,
        category: input.category,
        content: input.content,
        visibility: input.visibility,
      });

      // Reload templates after creation
      await get().loadTemplates();
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  updateTemplate: async (id, updates) => {
    set({ error: null });
    try {
      const current = get().templates.find(t => t.id === id);
      if (!current) {
        throw new Error(`Template ${id} not found`);
      }

      await invoke('update_template', {
        id,
        name: updates.name ?? current.name,
        category: updates.category ?? current.category,
        content: updates.content ?? current.content,
        visibility: updates.visibility ?? current.visibility,
      });

      // Reload templates after update
      await get().loadTemplates();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  deleteTemplate: async (id) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('delete_template', { id });

      // Reload templates after deletion
      await get().loadTemplates();
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  searchTemplates: async (query: string) => {
    try {
      const rawTemplates = await invoke<Array<{
        id: string;
        name: string;
        category: string;
        content: string;
        visibility: string;
        version: string;
        created_at: string;
        updated_at: string;
      }>>('search_templates', { query });

      return rawTemplates.map(t => ({
        id: t.id,
        name: t.name,
        category: t.category,
        content: t.content,
        visibility: t.visibility as 'private' | 'public' | 'team',
        version: t.version,
        createdAt: t.created_at,
        updatedAt: t.updated_at,
      }));
    } catch {
      return [];
    }
  },

  loadWorkflows: async () => {
    set({ isLoading: true, error: null });
    try {
      // Workflows are stored in shared_workflows table but not yet implemented in Tauri
      const workflows: SharedWorkflow[] = [];
      set({ workflows, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  createWorkflow: async (input) => {
    set({ isLoading: true, error: null });
    try {
      const workflow: SharedWorkflow = {
        id: `wf-${Date.now()}`,
        ...input,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      set(state => ({
        workflows: [...state.workflows, workflow],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  deleteWorkflow: async (id) => {
    set(state => ({
      workflows: state.workflows.filter(w => w.id !== id),
    }));
  },

  exportConversations: async (conversations, options) => {
    const data = {
      version: '1.0',
      exportedAt: new Date().toISOString(),
      conversations,
    };

    if (options.format === 'json') {
      const content = options.prettyPrint
        ? JSON.stringify(data, null, 2)
        : JSON.stringify(data);
      return new Blob([content], { type: 'application/json' });
    }

    // Markdown export
    let markdown = '# Conversation Export\n\n';
    for (const conv of conversations) {
      markdown += `## ${conv.title}\n\n`;
      if (options.includeTimestamps) {
        markdown += `*Created: ${conv.createdAt}*\n\n`;
      }
      for (const msg of conv.messages) {
        const role = msg.role === 'user' ? 'You' : 'Assistant';
        markdown += `**${role}**: ${msg.content}\n\n`;
      }
      markdown += '---\n\n';
    }

    return new Blob([markdown], { type: 'text/markdown' });
  },

  importConversations: async (file: File) => {
    const text = await file.text();
    const data = JSON.parse(text);

    if (data.conversations && Array.isArray(data.conversations)) {
      return data.conversations as ConversationExport[];
    }

    throw new Error('Invalid import file format');
  },

  clearError: () => set({ error: null }),
}));
