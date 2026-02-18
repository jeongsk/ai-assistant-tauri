/**
 * Collaboration Type Definitions
 */

export type Visibility = 'private' | 'public' | 'team';

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
