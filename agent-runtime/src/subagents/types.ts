/**
 * Sub-agent Type Definitions
 */

export type SubAgentType = 'code-reviewer' | 'researcher' | 'executor' | 'planner';

export type SubAgentStatus = 'idle' | 'running' | 'paused' | 'completed' | 'failed';

export interface SubAgent {
  id: string;
  name: string;
  type: SubAgentType;
  status: SubAgentStatus;
  systemPrompt?: string;
  tools: string[];
  config: SubAgentConfig;
  task?: string;
  result?: any;
  error?: string;
  createdAt: string;
  completedAt?: string;
}

export interface SubAgentConfig {
  maxIterations?: number;
  timeout?: number;
  temperature?: number;
  maxTokens?: number;
}

export interface SubAgentTask {
  id: string;
  agentId: string;
  prompt: string;
  context?: Record<string, any>;
  status: 'pending' | 'running' | 'completed' | 'failed';
  result?: any;
  error?: string;
  startedAt?: string;
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
  result?: any;
  error?: string;
}

// Default configurations per agent type
export const DEFAULT_AGENT_CONFIGS: Record<SubAgentType, SubAgentConfig> = {
  'code-reviewer': {
    maxIterations: 10,
    timeout: 120000,
    temperature: 0.3,
    maxTokens: 4096,
  },
  researcher: {
    maxIterations: 15,
    timeout: 180000,
    temperature: 0.5,
    maxTokens: 8192,
  },
  executor: {
    maxIterations: 5,
    timeout: 60000,
    temperature: 0.1,
    maxTokens: 2048,
  },
  planner: {
    maxIterations: 8,
    timeout: 90000,
    temperature: 0.7,
    maxTokens: 4096,
  },
};

// Default system prompts per agent type
export const DEFAULT_SYSTEM_PROMPTS: Record<SubAgentType, string> = {
  'code-reviewer': `You are a code reviewer sub-agent. Your task is to analyze code for:
- Bugs and potential issues
- Security vulnerabilities
- Performance concerns
- Code style and best practices
- Documentation completeness

Provide clear, actionable feedback with severity levels.`,
  researcher: `You are a research sub-agent. Your task is to:
- Gather information on the given topic
- Synthesize findings from multiple sources
- Identify key insights and patterns
- Provide well-organized summaries

Be thorough but concise in your research.`,
  executor: `You are an executor sub-agent. Your task is to:
- Execute specific actions as instructed
- Report results accurately
- Handle errors gracefully
- Confirm completion of tasks

Follow instructions precisely and report any issues.`,
  planner: `You are a planning sub-agent. Your task is to:
- Break down complex tasks into steps
- Identify dependencies between steps
- Estimate effort and complexity
- Create actionable plans

Be strategic and consider edge cases.`,
};
