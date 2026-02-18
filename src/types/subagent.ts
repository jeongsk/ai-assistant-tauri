/**
 * Sub-agent Type Definitions
 */

export type SubAgentType = 'code-reviewer' | 'researcher' | 'executor' | 'planner';

export type SubAgentStatus = 'idle' | 'running' | 'paused' | 'completed' | 'failed';

export interface SubAgentConfig {
  maxIterations?: number;
  timeout?: number;
  temperature?: number;
  maxTokens?: number;
}

export interface SubAgent {
  id: string;
  name: string;
  type: SubAgentType;
  status: SubAgentStatus;
  systemPrompt?: string;
  tools: string[];
  config: SubAgentConfig;
  task?: string;
  result?: string;
  error?: string;
  createdAt: string;
  completedAt?: string;
}

export interface SubAgentCreateInput {
  id: string;
  name: string;
  type: SubAgentType;
  systemPrompt?: string;
  tools?: string[];
  config?: SubAgentConfig;
}

export interface SubAgentUpdateInput {
  id: string;
  name?: string;
  systemPrompt?: string;
  tools?: string[];
  config?: SubAgentConfig;
  status?: SubAgentStatus;
  task?: string;
  result?: string;
  error?: string;
}

export const AGENT_TYPE_LABELS: Record<SubAgentType, string> = {
  'code-reviewer': 'Code Reviewer',
  researcher: 'Researcher',
  executor: 'Executor',
  planner: 'Planner',
};

export const AGENT_STATUS_COLORS: Record<SubAgentStatus, string> = {
  idle: 'gray',
  running: 'blue',
  paused: 'yellow',
  completed: 'green',
  failed: 'red',
};
