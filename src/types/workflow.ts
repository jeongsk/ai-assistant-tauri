/**
 * Workflow Types for v0.6
 *
 * Types for workflow automation, including nodes, triggers, and execution.
 */

// ============================================================================
// Workflow Store Types
// ============================================================================

export interface NodePosition {
  x: number;
  y: number;
}

export interface WorkflowNode {
  id: string;
  nodeType: string;
  position: NodePosition;
  data: unknown;
  label?: string;
}

export interface NodeConnection {
  source: string;
  sourceOutput: string;
  target: string;
  targetInput: string;
  condition?: string;
}

export interface WorkflowDefinition {
  entryPoint: string;
  nodes: Record<string, WorkflowNode>;
  connections: NodeConnection[];
}

export enum ExecutionStatus {
  Pending = 'pending',
  Running = 'running',
  Completed = 'completed',
  Failed = 'failed',
  Cancelled = 'cancelled',
}

export interface Workflow {
  id: string;
  name: string;
  description?: string;
  definition: WorkflowDefinition;
  version: number;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface WorkflowExecution {
  id: string;
  workflowId: string;
  status: ExecutionStatus;
  triggerType?: string;
  startedAt?: string;
  completedAt?: string;
  result?: unknown;
  error?: string;
}

// ============================================================================
// Workflow Engine Types
// ============================================================================

export interface NodeContext {
  workflowId: string;
  variables: Record<string, unknown>;
  input: unknown;
  results: Record<string, unknown>;
}

export type NodeResult =
  | { type: 'success'; output: unknown; nextNode?: string }
  | { type: 'failure'; error: string };

export interface ExecutionResult {
  success: boolean;
  output: unknown;
  executedNodes: string[];
  error?: string;
}

// ============================================================================
// Trigger Types
// ============================================================================

export enum TriggerTypeEnum {
  Schedule = 'schedule',
  Webhook = 'webhook',
  FileSystem = 'filesystem',
  Voice = 'voice',
  Manual = 'manual',
}

export enum HttpMethod {
  Get = 'GET',
  Post = 'POST',
  Put = 'PUT',
  Delete = 'DELETE',
}

export enum FsEvent {
  Create = 'create',
  Modify = 'modify',
  Delete = 'delete',
  Rename = 'rename',
}

export type Trigger =
  | { type: TriggerTypeEnum.Schedule; cron: string; timezone: string }
  | { type: TriggerTypeEnum.Webhook; path: string; method: HttpMethod }
  | { type: TriggerTypeEnum.FileSystem; path: string; events: FsEvent[] }
  | { type: TriggerTypeEnum.Voice; pattern: string; language: string }
  | { type: TriggerTypeEnum.Manual };

export interface TriggerHandle {
  triggerId: string;
  workflowId: string;
  triggerType: TriggerTypeEnum;
}
