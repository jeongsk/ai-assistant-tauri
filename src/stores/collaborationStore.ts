/**
 * Collaboration Store - Zustand store for templates and sharing
 */

import { create } from 'zustand';
import { Template, SharedWorkflow, ExportOptions, ConversationExport } from '../types/collaboration';

interface CollaborationState {
  templates: Template[];
  workflows: SharedWorkflow[];
  isLoading: boolean;
  error: string | null;

  // Template Actions
  loadTemplates: () => Promise<void>;
  createTemplate: (template: Omit<Template, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>;
  updateTemplate: (id: string, updates: Partial<Template>) => Promise<void>;
  deleteTemplate: (id: string) => Promise<void>;

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
      // In production, load from Tauri
      const templates: Template[] = [];
      set({ templates, isLoading: false });
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  createTemplate: async (input) => {
    set({ isLoading: true, error: null });
    try {
      const template: Template = {
        id: `tpl-${Date.now()}`,
        ...input,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      set(state => ({
        templates: [...state.templates, template],
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  updateTemplate: async (id, updates) => {
    set({ error: null });
    try {
      set(state => ({
        templates: state.templates.map(t =>
          t.id === id ? { ...t, ...updates, updatedAt: new Date().toISOString() } : t
        ),
      }));
    } catch (error) {
      set({ error: (error as Error).message });
    }
  },

  deleteTemplate: async (id) => {
    set({ isLoading: true, error: null });
    try {
      set(state => ({
        templates: state.templates.filter(t => t.id !== id),
        isLoading: false,
      }));
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
    }
  },

  loadWorkflows: async () => {
    set({ isLoading: true, error: null });
    try {
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
