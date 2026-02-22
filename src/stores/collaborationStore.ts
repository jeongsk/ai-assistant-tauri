/**
 * Collaboration Store - Zustand store for templates and sharing
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import {
  Template,
  SharedWorkflow,
  ExportOptions,
  ConversationExport,
  ConflictResolution,
  ImportResult,
  TemplateVersion,
  TemplateShareRequest
} from '../types/collaboration';

interface CollaborationState {
  templates: Template[];
  workflows: SharedWorkflow[];
  templateVersions: TemplateVersion[];
  teamTemplates: Template[];
  isLoading: boolean;
  error: string | null;

  // Template Actions
  loadTemplates: () => Promise<void>;
  getTemplate: (id: string) => Promise<Template | undefined>;
  createTemplate: (template: Omit<Template, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>;
  updateTemplate: (id: string, updates: Partial<Omit<Template, 'id' | 'createdAt' | 'updatedAt'>>) => Promise<void>;
  deleteTemplate: (id: string) => Promise<void>;
  searchTemplates: (query: string) => Promise<Template[]>;

  // Template Import/Export (v0.5)
  exportTemplate: (id: string) => Promise<Uint8Array>;
  exportAllTemplates: () => Promise<Uint8Array>;
  importTemplate: (data: Uint8Array, resolution: ConflictResolution) => Promise<Template>;
  importTemplates: (data: Uint8Array, resolution: ConflictResolution) => Promise<ImportResult>;
  validateTemplateData: (data: unknown) => Promise<boolean>;

  // Template Versioning (v0.5)
  getTemplateVersions: (id: string) => Promise<TemplateVersion[]>;
  createTemplateVersion: (id: string, notes: string) => Promise<number>;
  rollbackTemplate: (id: string, versionId: number) => Promise<void>;

  // Template Sharing (v0.5)
  shareTemplateToTeam: (request: TemplateShareRequest) => Promise<void>;
  getTeamTemplates: (teamId: string) => Promise<Template[]>;
  revokeTemplateAccess: (id: string, teamId: string) => Promise<void>;

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
  templateVersions: [],
  teamTemplates: [],
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

  // Template Import/Export (v0.5)
  exportTemplate: async (id: string) => {
    try {
      const data = await invoke<number[]>('export_template', { id });
      return new Uint8Array(data);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  exportAllTemplates: async () => {
    try {
      const data = await invoke<number[]>('export_all_templates');
      return new Uint8Array(data);
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  importTemplate: async (data: Uint8Array, resolution: ConflictResolution) => {
    set({ isLoading: true, error: null });
    try {
      const rawData = Array.from(data);
      const rawTemplate = await invoke<{
        id: string;
        name: string;
        category: string;
        content: string;
        visibility: string;
        version: string;
        created_at: string;
        updated_at: string;
      }>('import_template', {
        data: rawData,
        resolution,
      });

      const template: Template = {
        id: rawTemplate.id,
        name: rawTemplate.name,
        category: rawTemplate.category,
        content: rawTemplate.content,
        visibility: rawTemplate.visibility as 'private' | 'public' | 'team',
        version: rawTemplate.version,
        createdAt: rawTemplate.created_at,
        updatedAt: rawTemplate.updated_at,
      };

      // Reload templates after import
      await get().loadTemplates();
      return template;
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  importTemplates: async (data: Uint8Array, resolution: ConflictResolution) => {
    set({ isLoading: true, error: null });
    try {
      const rawData = Array.from(data);
      const result = await invoke<{
        success: boolean;
        imported: number;
        skipped: number;
        error_count: number;
        errors: Array<{ template: string; error: string }>;
      }>('import_templates', {
        data: rawData,
        resolution,
      });

      const importResult: ImportResult = {
        success: result.success,
        imported: result.imported,
        skipped: result.skipped,
        errorCount: result.error_count,
        errors: result.errors,
      };

      // Reload templates after import
      if (result.success) {
        await get().loadTemplates();
      }

      return importResult;
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  validateTemplateData: async (data: unknown) => {
    try {
      return await invoke<boolean>('validate_template_data', { data });
    } catch {
      return false;
    }
  },

  // Template Versioning (v0.5)
  getTemplateVersions: async (id: string) => {
    try {
      const rawVersions = await invoke<Array<{
        id: number;
        template_id: string;
        version: number;
        content: string;
        notes: string | null;
        created_at: string;
      }>>('get_template_versions', { id });

      const versions: TemplateVersion[] = rawVersions.map(v => ({
        id: v.id,
        templateId: v.template_id,
        version: v.version,
        content: v.content,
        notes: v.notes ?? undefined,
        createdAt: v.created_at,
      }));

      set({ templateVersions: versions });
      return versions;
    } catch (error) {
      set({ error: (error as Error).message });
      return [];
    }
  },

  createTemplateVersion: async (id: string, notes: string) => {
    try {
      const versionId = await invoke<number>('create_template_version', { id, notes });
      // Reload versions after creation
      await get().getTemplateVersions(id);
      return versionId;
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  rollbackTemplate: async (id: string, versionId: number) => {
    set({ isLoading: true, error: null });
    try {
      await invoke('rollback_template', { id, versionId });
      // Reload templates after rollback
      await get().loadTemplates();
    } catch (error) {
      set({ error: (error as Error).message, isLoading: false });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },

  // Template Sharing (v0.5)
  shareTemplateToTeam: async (request: TemplateShareRequest) => {
    try {
      await invoke('share_template_to_team', {
        id: request.templateId,
        teamId: request.teamId,
        permissions: request.permissions,
      });
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    }
  },

  getTeamTemplates: async (teamId: string) => {
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
      }>>('get_team_templates', { teamId });

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

      set({ teamTemplates: templates });
      return templates;
    } catch (error) {
      set({ error: (error as Error).message });
      return [];
    }
  },

  revokeTemplateAccess: async (id: string, teamId: string) => {
    try {
      await invoke('revoke_template_access', { id, teamId });
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
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
