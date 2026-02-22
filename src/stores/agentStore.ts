/**
 * Agent Store - Zustand state management for v0.6 agent features
 *
 * Handles multimodal processing, context management, and agent orchestration.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type {
  Message,
  MessageRole,
  CompressionStrategy,
  CompressionResult,
  SubAgentTask,
  AgentType,
  TaskPriority,
  AggregatedResult,
  ImageAnalysis,
} from '../types/agent';
import { MessagePriority } from '../types/agent';

interface AgentState {
  // Context state
  messages: Message[];
  tokenCount: number;
  isNearLimit: boolean;

  // Orchestrator state
  queueLength: number;
  lastExecutionResult: AggregatedResult | null;

  // Loading states
  contextLoading: boolean;
  orchestratorLoading: boolean;

  // Error state
  error: string | null;

  // Context actions
  addMessage: (role: MessageRole, content: string, priority?: MessagePriority, tokenCount?: number) => Promise<void>;
  getMessages: () => Promise<Message[]>;
  clearContext: () => Promise<void>;
  getTokenCount: () => Promise<number>;
  checkIsNearLimit: () => Promise<boolean>;
  compressContext: () => Promise<CompressionResult>;
  setCompressionStrategy: (strategy: CompressionStrategy, minTokens?: number, targetRatio?: number) => Promise<void>;

  // Multimodal actions
  processMultimodal: (inputType: 'text' | 'image' | 'mixed', text?: string, imageData?: number[], imageFormat?: string) => Promise<unknown>;
  analyzeImage: (imageData: number[], format: string) => Promise<ImageAnalysis>;

  // Orchestrator actions
  addTask: (
    id: string,
    agentType: AgentType,
    description: string,
    data?: unknown,
    priority?: TaskPriority,
    timeoutSeconds?: number
  ) => Promise<void>;
  executeAll: () => Promise<AggregatedResult>;
  getQueueLength: () => Promise<number>;
  clearCompleted: () => Promise<void>;
}

export const useAgentStore = create<AgentState>((set, get) => ({
  // Initial state
  messages: [],
  tokenCount: 0,
  isNearLimit: false,
  queueLength: 0,
  lastExecutionResult: null,
  contextLoading: false,
  orchestratorLoading: false,
  error: null,

  // Context actions
  addMessage: async (role: MessageRole, content: string, priority?: MessagePriority, tokenCount?: number) => {
    set({ contextLoading: true, error: null });
    try {
      await invoke('agent_context_add_message', {
        role,
        content,
        priority: priority || MessagePriority.Normal,
        tokenCount,
      });

      // Refresh messages
      await get().getMessages();
    } catch (error) {
      set({ error: String(error), contextLoading: false });
      throw error;
    }
  },

  getMessages: async () => {
    set({ contextLoading: true, error: null });
    try {
      const rawMessages = await invoke<Array<{
        role: string;
        content: string;
        token_count: number;
        priority: string;
        timestamp: number;
      }>>('agent_context_get_messages');

      const messages: Message[] = rawMessages.map((m) => ({
        role: m.role as MessageRole,
        content: m.content,
        tokenCount: m.token_count,
        priority: m.priority as MessagePriority,
        timestamp: m.timestamp,
      }));

      set({ messages, contextLoading: false });
      return messages;
    } catch (error) {
      set({ error: String(error), contextLoading: false });
      throw error;
    }
  },

  clearContext: async () => {
    set({ contextLoading: true, error: null });
    try {
      await invoke('agent_context_clear');
      set({ messages: [], tokenCount: 0, contextLoading: false });
    } catch (error) {
      set({ error: String(error), contextLoading: false });
      throw error;
    }
  },

  getTokenCount: async () => {
    try {
      const count = await invoke<number>('agent_context_token_count');
      set({ tokenCount: count });
      return count;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  checkIsNearLimit: async () => {
    try {
      const near = await invoke<boolean>('agent_context_is_near_limit');
      set({ isNearLimit: near });
      return near;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  compressContext: async () => {
    set({ contextLoading: true, error: null });
    try {
      const result = await invoke<CompressionResult>('agent_context_compress');
      await get().getMessages();
      await get().getTokenCount();
      set({ contextLoading: false });
      return result;
    } catch (error) {
      set({ error: String(error), contextLoading: false });
      throw error;
    }
  },

  setCompressionStrategy: async (strategy: CompressionStrategy, minTokens?: number, targetRatio?: number) => {
    try {
      await invoke('agent_context_set_strategy', {
        strategy,
        minTokens,
        targetRatio,
      });
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  // Multimodal actions
  processMultimodal: async (inputType: 'text' | 'image' | 'mixed', text?: string, imageData?: number[], imageFormat?: string) => {
    set({ contextLoading: true, error: null });
    try {
      const result = await invoke('agent_multimodal_process', {
        inputType,
        text,
        imageData,
        imageFormat,
      });
      set({ contextLoading: false });
      return result;
    } catch (error) {
      set({ error: String(error), contextLoading: false });
      throw error;
    }
  },

  analyzeImage: async (imageData: number[], format: string) => {
    set({ contextLoading: true, error: null });
    try {
      const result = await invoke<ImageAnalysis>('agent_analyze_image', {
        imageData,
        format,
      });
      set({ contextLoading: false });
      return result;
    } catch (error) {
      set({ error: String(error), contextLoading: false });
      throw error;
    }
  },

  // Orchestrator actions
  addTask: async (
    id: string,
    agentType: AgentType,
    description: string,
    data?: unknown,
    priority?: TaskPriority,
    timeoutSeconds?: number
  ) => {
    set({ orchestratorLoading: true, error: null });
    try {
      await invoke('agent_orchestrator_add_task', {
        id,
        agentType,
        description,
        data,
        priority,
        timeoutSeconds,
      });

      // Refresh queue length
      await get().getQueueLength();
      set({ orchestratorLoading: false });
    } catch (error) {
      set({ error: String(error), orchestratorLoading: false });
      throw error;
    }
  },

  executeAll: async () => {
    set({ orchestratorLoading: true, error: null });
    try {
      const result = await invoke<AggregatedResult>('agent_orchestrator_execute_all');
      set({ lastExecutionResult: result, orchestratorLoading: false });

      // Refresh queue length
      await get().getQueueLength();
      return result;
    } catch (error) {
      set({ error: String(error), orchestratorLoading: false });
      throw error;
    }
  },

  getQueueLength: async () => {
    try {
      const length = await invoke<number>('agent_orchestrator_queue_length');
      set({ queueLength: length });
      return length;
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  clearCompleted: async () => {
    try {
      await invoke('agent_orchestrator_clear_completed');
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },
}));
