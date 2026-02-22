/**
 * Agent Core - Main agent logic
 */

import type { BaseProvider, Message } from '../providers/base.js';
import { logger } from '../utils/logger.js';

export interface AgentConfig {
  maxSteps: number;
  timeout: number;
}

// Provider accessor interface for dependency injection
export interface ProviderAccessor {
  getActiveProvider(): BaseProvider;
}

export class AgentCore {
  private config: AgentConfig;
  private getProvider: ProviderAccessor['getActiveProvider'];

  constructor(
    getProvider: ProviderAccessor['getActiveProvider'],
    config?: Partial<AgentConfig>
  ) {
    this.getProvider = getProvider;
    this.config = {
      maxSteps: config?.maxSteps || 10,
      timeout: config?.timeout || 60000,
    };
    logger.info('AgentCore initialized', this.config);
  }

  async plan(task: string): Promise<string[]> {
    logger.debug('Planning task', { task });

    try {
      const provider = this.getProvider();

      const planningPrompt: Message = {
        role: 'user',
        content: `You are a task planning assistant. Break down the following task into a series of executable steps.

Task: ${task}

Respond with a JSON array of step strings. Each step should be a clear, actionable instruction.
Example format: ["Step 1 description", "Step 2 description", "Step 3 description"]

Only respond with the JSON array, no other text.`
      };

      const response = await provider.chat([planningPrompt], {
        maxTokens: 1000,
        temperature: 0.7,
      });

      // Parse the response as JSON
      const steps = JSON.parse(response.content);
      if (!Array.isArray(steps)) {
        throw new Error('Provider response is not an array');
      }

      logger.info('Task planned', { stepCount: steps.length });
      return steps;
    } catch (error) {
      logger.error('Planning failed', error);
      throw new Error(`Failed to plan task: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  async execute(steps: string[]): Promise<any> {
    logger.debug('Executing steps', { count: steps.length });

    const results: Array<{ step: string; result: string }> = [];

    try {
      const provider = this.getProvider();

      for (let i = 0; i < steps.length && i < this.config.maxSteps; i++) {
        const step = steps[i];
        logger.debug(`Executing step ${i + 1}/${steps.length}`, { step });

        const stepPrompt: Message = {
          role: 'user',
          content: `You are executing step ${i + 1} of a larger task.

Step to execute: ${step}

Previous results: ${results.length > 0 ? JSON.stringify(results, null, 2) : 'None'}

Execute this step and provide a clear result. Respond with a concise description of what was accomplished.`
        };

        const response = await provider.chat([stepPrompt], {
          maxTokens: 500,
          temperature: 0.3,
        });

        results.push({
          step,
          result: response.content,
        });

        logger.debug(`Step ${i + 1} completed`, {
          resultLength: response.content.length,
        });
      }

      logger.info('Execution completed', {
        stepsExecuted: results.length,
        totalSteps: steps.length,
      });

      return {
        success: true,
        results,
        metadata: {
          stepsExecuted: results.length,
          totalSteps: steps.length,
        },
      };
    } catch (error) {
      logger.error('Execution failed', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        partialResults: results,
      };
    }
  }
}
