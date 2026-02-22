/**
 * Collaboration Type Definitions
 */

export type Visibility = 'private' | 'public' | 'team';

export type ConflictResolution = 'skip' | 'overwrite' | 'rename' | 'version';

export interface Template {
  id: string;
  name: string;
  category: string;
  content: string;
  visibility: Visibility;
  version: string;
  createdAt: string;
  updatedAt: string;
}

export interface SharedWorkflow {
  id: string;
  name: string;
  description?: string;
  steps: WorkflowStep[];
  ownerId?: string;
  visibility: Visibility;
  createdAt: string;
  updatedAt: string;
}

export interface WorkflowStep {
  id: string;
  name: string;
  type: 'skill' | 'recipe' | 'prompt';
  config: Record<string, unknown>;
  order: number;
}

export interface ExportOptions {
  format: 'json' | 'markdown' | 'html';
  includeMetadata: boolean;
  includeTimestamps: boolean;
  prettyPrint: boolean;
}

export interface ConversationExport {
  id: string;
  title: string;
  createdAt: string;
  messages: Array<{
    role: string;
    content: string;
    createdAt: string;
  }>;
}

// Template Versioning (v0.5)
export interface TemplateVersion {
  id: number;
  templateId: string;
  version: number;
  content: string;
  notes?: string;
  createdAt: string;
}

// Template Import/Export (v0.5)
export interface ImportResult {
  success: boolean;
  imported: number;
  skipped: number;
  errorCount: number;
  errors: Array<{
    template: string;
    error: string;
  }>;
}

// Template Sharing (v0.5)
export interface TemplateShare {
  id: number;
  templateId: string;
  teamId: string;
  permissions: {
    read: boolean;
    write: boolean;
    execute: boolean;
  };
  sharedBy: string;
  sharedAt: string;
}

export interface TemplateShareRequest {
  templateId: string;
  teamId: string;
  permissions: {
    read: boolean;
    write: boolean;
    execute: boolean;
  };
}
