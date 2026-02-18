/**
 * Sub-agent Manager - Manages sub-agent lifecycle and task execution
 */

import { logger } from '../utils/logger.js';
import type {
  SubAgent,
  SubAgentType,
  SubAgentStatus,
  SubAgentConfig,
  SubAgentCreateInput,
  SubAgentUpdateInput,
  SubAgentTask,
  DEFAULT_AGENT_CONFIGS,
  DEFAULT_SYSTEM_PROMPTS,
} from './types.js';

export class SubAgentManager {
  private agents: Map<string, SubAgent> = new Map();
  private tasks: Map<string, SubAgentTask> = new Map();

  /**
   * Create a new sub-agent
   */
  async createAgent(input: SubAgentCreateInput): Promise<SubAgent> {
    const agent: SubAgent = {
      id: input.id,
      name: input.name,
      type: input.type,
      status: 'idle',
      systemPrompt: input.systemPrompt || this.getDefaultPrompt(input.type),
      tools: input.tools || [],
      config: input.config || this.getDefaultConfig(input.type),
      createdAt: new Date().toISOString(),
    };

    this.agents.set(agent.id, agent);
    logger.info('Created sub-agent', { id: agent.id, type: agent.type });

    return agent;
  }

  /**
   * Get an agent by ID
   */
  getAgent(id: string): SubAgent | undefined {
    return this.agents.get(id);
  }

  /**
   * List all agents
   */
  listAgents(): SubAgent[] {
    return Array.from(this.agents.values());
  }

  /**
   * Update an agent
   */
  async updateAgent(input: SubAgentUpdateInput): Promise<SubAgent | undefined> {
    const agent = this.agents.get(input.id);
    if (!agent) return undefined;

    if (input.name !== undefined) agent.name = input.name;
    if (input.systemPrompt !== undefined) agent.systemPrompt = input.systemPrompt;
    if (input.tools !== undefined) agent.tools = input.tools;
    if (input.config !== undefined) agent.config = input.config;
    if (input.status !== undefined) agent.status = input.status;
    if (input.task !== undefined) agent.task = input.task;
    if (input.result !== undefined) agent.result = input.result;
    if (input.error !== undefined) agent.error = input.error;

    logger.debug('Updated sub-agent', { id: agent.id });
    return agent;
  }

  /**
   * Delete an agent
   */
  async deleteAgent(id: string): Promise<boolean> {
    const deleted = this.agents.delete(id);
    if (deleted) {
      logger.info('Deleted sub-agent', { id });
    }
    return deleted;
  }

  /**
   * Assign a task to an agent
   */
  async assignTask(agentId: string, task: string, context?: Record<string, any>): Promise<SubAgentTask> {
    const agent = this.agents.get(agentId);
    if (!agent) {
      throw new Error(`Agent not found: ${agentId}`);
    }

    if (agent.status === 'running') {
      throw new Error(`Agent is busy: ${agentId}`);
    }

    // Update agent status
    agent.status = 'running';
    agent.task = task;

    // Create task record
    const taskId = `${agentId}-${Date.now()}`;
    const agentTask: SubAgentTask = {
      id: taskId,
      agentId,
      prompt: task,
      context,
      status: 'running',
      startedAt: new Date().toISOString(),
    };

    this.tasks.set(taskId, agentTask);
    logger.info('Assigned task to sub-agent', { agentId, taskId });

    return agentTask;
  }

  /**
   * Get task result
   */
  async getTaskResult(taskId: string): Promise<SubAgentTask | undefined> {
    return this.tasks.get(taskId);
  }

  /**
   * Complete a task
   */
  async completeTask(taskId: string, result: any): Promise<void> {
    const task = this.tasks.get(taskId);
    if (!task) return;

    task.status = 'completed';
    task.result = result;
    task.completedAt = new Date().toISOString();

    // Update agent
    const agent = this.agents.get(task.agentId);
    if (agent) {
      agent.status = 'idle';
      agent.result = result;
      agent.completedAt = new Date().toISOString();
    }

    logger.info('Task completed', { taskId, agentId: task.agentId });
  }

  /**
   * Fail a task
   */
  async failTask(taskId: string, error: string): Promise<void> {
    const task = this.tasks.get(taskId);
    if (!task) return;

    task.status = 'failed';
    task.error = error;
    task.completedAt = new Date().toISOString();

    // Update agent
    const agent = this.agents.get(task.agentId);
    if (agent) {
      agent.status = 'failed';
      agent.error = error;
      agent.completedAt = new Date().toISOString();
    }

    logger.error('Task failed', { taskId, agentId: task.agentId, error });
  }

  /**
   * Get default config for agent type
   */
  private getDefaultConfig(type: SubAgentType): SubAgentConfig {
    const configs: Record<SubAgentType, SubAgentConfig> = {
      'code-reviewer': { maxIterations: 10, timeout: 120000, temperature: 0.3, maxTokens: 4096 },
      researcher: { maxIterations: 15, timeout: 180000, temperature: 0.5, maxTokens: 8192 },
      executor: { maxIterations: 5, timeout: 60000, temperature: 0.1, maxTokens: 2048 },
      planner: { maxIterations: 8, timeout: 90000, temperature: 0.7, maxTokens: 4096 },
    };
    return configs[type];
  }

  /**
   * Get default prompt for agent type
   */
  private getDefaultPrompt(type: SubAgentType): string {
    const prompts: Record<SubAgentType, string> = {
      'code-reviewer': 'You are a code reviewer sub-agent. Analyze code for bugs, security issues, and best practices.',
      researcher: 'You are a research sub-agent. Gather and synthesize information on the given topic.',
      executor: 'You are an executor sub-agent. Execute specific actions and report results accurately.',
      planner: 'You are a planning sub-agent. Break down complex tasks into actionable steps.',
    };
    return prompts[type];
  }

  /**
   * Shutdown manager
   */
  async shutdown(): Promise<void> {
    logger.info('Shutting down SubAgentManager');
    this.agents.clear();
    this.tasks.clear();
  }
}

// Singleton instance
let instance: SubAgentManager | null = null;

export function getSubAgentManager(): SubAgentManager {
  if (!instance) {
    instance = new SubAgentManager();
  }
  return instance;
}
