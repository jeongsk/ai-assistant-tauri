/**
 * Workflow Store - Zustand state management for v0.6 workflow features
 *
 * Handles workflow CRUD, execution, and trigger management.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type {
  Workflow,
  WorkflowExecution,
  ExecutionStatus,
  TriggerHandle,
  Trigger,
  TriggerTypeEnum,
  HttpMethod,
  FsEvent,
  ExecutionResult,
  NodePosition,
} from '../types/workflow';

interface WorkflowState {
  // Data
  workflows: Workflow[];
  executions: Record<string, WorkflowExecution[]>;
  triggers: TriggerHandle[];

  // Loading states
  loading: boolean;
  executing: boolean;

  // Error state
  error: string | null;

  // Workflow actions
  loadWorkflows: () => Promise<void>;
  createWorkflow: (
    id: string,
    name: string,
    description?: string,
    entryPoint?: string,
    isActive?: boolean
  ) => Promise<string>;
  getWorkflow: (id: string) => Promise<Workflow | null>;
  updateWorkflow: (
    id: string,
    name?: string,
    description?: string,
    entryPoint?: string,
    isActive?: boolean
  ) => Promise<void>;
  deleteWorkflow: (id: string) => Promise<void>;
  loadActiveWorkflows: () => Promise<void>;

  // Node actions
  addNode: (
    workflowId: string,
    nodeId: string,
    nodeType: string,
    x: number,
    y: number,
    data?: unknown,
    label?: string
  ) => Promise<void>;
  addConnection: (
    workflowId: string,
    source: string,
    sourceOutput: string,
    target: string,
    targetInput: string,
    condition?: string
  ) => Promise<void>;

  // Execution actions
  executeWorkflow: (id: string, input?: unknown) => Promise<ExecutionResult>;
  createExecution: (
    id: string,
    workflowId: string,
    triggerType?: string
  ) => Promise<string>;
  getExecution: (id: string) => Promise<WorkflowExecution | null>;
  getExecutions: (workflowId: string) => Promise<WorkflowExecution[]>;
  updateExecution: (
    id: string,
    status: ExecutionStatus,
    result?: unknown,
    error?: string
  ) => Promise<void>;

  // Trigger actions
  registerTrigger: (
    triggerId: string,
    workflowId: string,
    triggerType: TriggerTypeEnum,
    config?: Record<string, unknown>
  ) => Promise<void>;
  unregisterTrigger: (triggerId: string) => Promise<void>;
  listTriggers: () => Promise<TriggerHandle[]>;
  getTriggerCount: () => Promise<number>;
}

export const useWorkflowStore = create<WorkflowState>((set, get) => ({
  // Initial state
  workflows: [],
  executions: {},
  triggers: [],
  loading: false,
  executing: false,
  error: null,

  // Workflow actions
  loadWorkflows: async () => {
    set({ loading: true, error: null });
    try {
      const workflows = await invoke<Workflow[]>('workflow_list');
      set({ workflows, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
      throw error;
    }
  },

  createWorkflow: async (
    id: string,
    name: string,
    description?: string,
    entryPoint?: string,
    isActive?: boolean
  ) => {
    set({ loading: true, error: null });
    try {
      const workflowId = await invoke<string>('workflow_create', {
        id,
        name,
        description,
        entryPoint: entryPoint || id,
        isActive,
      });

      // Reload workflows
      await get().loadWorkflows();
      set({ loading: false });
      return workflowId;
    } catch (error) {
      set({ error: String(error), loading: false });
      throw error;
    }
  },

  getWorkflow: async (id: string) => {
    try {
      const workflow = await invoke<Workflow>('workflow_get', { id });
      return workflow;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateWorkflow: async (
    id: string,
    name?: string,
    description?: string,
    entryPoint?: string,
    isActive?: boolean
  ) => {
    set({ loading: true, error: null });
    try {
      await invoke('workflow_update', {
        id,
        name,
        description,
        entryPoint,
        isActive,
      });

      // Reload workflows
      await get().loadWorkflows();
      set({ loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
      throw error;
    }
  },

  deleteWorkflow: async (id: string) => {
    set({ loading: true, error: null });
    try {
      await invoke('workflow_delete', { id });

      set((state) => ({
        workflows: state.workflows.filter((w) => w.id !== id),
        loading: false,
      }));
    } catch (error) {
      set({ error: String(error), loading: false });
      throw error;
    }
  },

  loadActiveWorkflows: async () => {
    set({ loading: true, error: null });
    try {
      const workflows = await invoke<Workflow[]>('workflow_list_active');
      set({ workflows, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
      throw error;
    }
  },

  // Node actions
  addNode: async (
    workflowId: string,
    nodeId: string,
    nodeType: string,
    x: number,
    y: number,
    data?: unknown,
    label?: string
  ) => {
    try {
      await invoke('workflow_add_node', {
        workflowId,
        nodeId,
        nodeType,
        x,
        y,
        data,
        label,
      });

      // Reload workflows
      await get().loadWorkflows();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  addConnection: async (
    workflowId: string,
    source: string,
    sourceOutput: string,
    target: string,
    targetInput: string,
    condition?: string
  ) => {
    try {
      await invoke('workflow_add_connection', {
        workflowId,
        source,
        sourceOutput,
        target,
        targetInput,
        condition,
      });

      // Reload workflows
      await get().loadWorkflows();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  // Execution actions
  executeWorkflow: async (id: string, input?: unknown) => {
    set({ executing: true, error: null });
    try {
      const result = await invoke<ExecutionResult>('workflow_execute', {
        id,
        input,
      });
      set({ executing: false });
      return result;
    } catch (error) {
      set({ error: String(error), executing: false });
      throw error;
    }
  },

  createExecution: async (
    id: string,
    workflowId: string,
    triggerType?: string
  ) => {
    try {
      const executionId = await invoke<string>('workflow_create_execution', {
        id,
        workflowId,
        triggerType,
      });

      // Load executions for this workflow
      await get().getExecutions(workflowId);
      return executionId;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getExecution: async (id: string) => {
    try {
      const execution = await invoke<WorkflowExecution>('workflow_get_execution', { id });
      return execution;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getExecutions: async (workflowId: string) => {
    try {
      const executions = await invoke<WorkflowExecution[]>('workflow_get_executions', {
        workflowId,
      });

      set((state) => ({
        executions: {
          ...state.executions,
          [workflowId]: executions,
        },
      }));

      return executions;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateExecution: async (
    id: string,
    status: ExecutionStatus,
    result?: unknown,
    error?: string
  ) => {
    try {
      await invoke('workflow_update_execution', {
        id,
        status,
        result,
        error,
      });
    } catch (err) {
      set({ error: String(err) });
      throw err;
    }
  },

  // Trigger actions
  registerTrigger: async (
    triggerId: string,
    workflowId: string,
    triggerType: TriggerTypeEnum,
    config?: Record<string, unknown>
  ) => {
    try {
      await invoke('workflow_register_trigger', {
        triggerId,
        workflowId,
        triggerType,
        config,
      });

      // Reload triggers
      await get().listTriggers();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  unregisterTrigger: async (triggerId: string) => {
    try {
      await invoke('workflow_unregister_trigger', { triggerId });

      set((state) => ({
        triggers: state.triggers.filter((t) => t.triggerId !== triggerId),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  listTriggers: async () => {
    try {
      const triggers = await invoke<TriggerHandle[]>('workflow_list_triggers');
      set({ triggers });
      return triggers;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getTriggerCount: async () => {
    try {
      const count = await invoke<number>('workflow_trigger_count');
      return count;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
