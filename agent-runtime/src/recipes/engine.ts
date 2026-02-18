/**
 * Recipe Engine - Execute recipes step by step
 */

import type {
  Recipe,
  RecipeStep,
  RecipeExecutionContext,
  RecipeResult,
} from './types.js';
import { RECIPE_LIMITS } from './types.js';
import { logger } from '../utils/logger.js';

export class RecipeEngine {
  private provider: any;
  private mcpClient: any;

  constructor(provider: any, mcpClient: any) {
    this.provider = provider;
    this.mcpClient = mcpClient;
  }

  async execute(
    recipe: Recipe,
    variables: Record<string, unknown> = {}
  ): Promise<RecipeResult> {
    // Validate step count
    if (recipe.steps.length > RECIPE_LIMITS.maxSteps) {
      throw new Error(
        `Recipe exceeds maximum steps (${RECIPE_LIMITS.maxSteps})`
      );
    }

    const context: RecipeExecutionContext = {
      recipeId: recipe.id,
      executionId: crypto.randomUUID(),
      variables: { ...recipe.variables, ...variables },
      stepResults: new Map(),
      depth: 0,
    };

    try {
      await this.executeSteps(recipe.steps, context);

      return {
        success: true,
        results: Object.fromEntries(context.stepResults),
      };
    } catch (error) {
      return {
        success: false,
        results: Object.fromEntries(context.stepResults),
        error: String(error),
      };
    }
  }

  private async executeSteps(
    steps: RecipeStep[],
    context: RecipeExecutionContext
  ): Promise<void> {
    for (const step of steps) {
      await this.executeStep(step, context);
    }
  }

  private async executeStep(
    step: RecipeStep,
    context: RecipeExecutionContext
  ): Promise<void> {
    logger.debug(`Executing step: ${step.name}`, { stepId: step.id });

    try {
      let result: unknown;

      switch (step.type) {
        case 'prompt':
          result = await this.executePromptStep(step, context);
          break;
        case 'tool_call':
          result = await this.executeToolCallStep(step, context);
          break;
        case 'condition':
          await this.executeConditionStep(step, context);
          return; // No output variable for condition
        case 'loop':
          result = await this.executeLoopStep(step, context);
          break;
        case 'parallel':
          result = await this.executeParallelStep(step, context);
          break;
        default:
          throw new Error(`Unknown step type: ${step.type}`);
      }

      // Store result in output variable
      if (step.outputVariable && result !== undefined) {
        context.stepResults.set(step.outputVariable, result);
      }
    } catch (error) {
      if (step.onError === 'continue') {
        logger.error(`Step ${step.name} failed, continuing`, error);
      } else if (step.onError === 'retry' && step.retryCount) {
        for (let i = 0; i < step.retryCount; i++) {
          try {
            await this.executeStep(step, context);
            return;
          } catch (retryError) {
            logger.warn(`Retry ${i + 1} failed for step ${step.name}`);
          }
        }
      } else {
        throw error;
      }
    }
  }

  private async executePromptStep(
    step: RecipeStep,
    context: RecipeExecutionContext
  ): Promise<string> {
    if (!step.prompt) {
      throw new Error(`Prompt step ${step.name} has no prompt`);
    }

    const resolvedPrompt = this.substituteVariables(step.prompt, context);

    if (!this.provider) {
      // Return mock result for testing
      return `Mock response for: ${resolvedPrompt.slice(0, 100)}...`;
    }

    const response = await this.provider.chat([
      { role: 'user', content: resolvedPrompt },
    ]);

    return response.content;
  }

  private async executeToolCallStep(
    step: RecipeStep,
    context: RecipeExecutionContext
  ): Promise<unknown> {
    if (!step.tool) {
      throw new Error(`Tool call step ${step.name} has no tool`);
    }

    const resolvedArgs: Record<string, unknown> = {};
    if (step.args) {
      for (const [key, value] of Object.entries(step.args)) {
        if (typeof value === 'string') {
          resolvedArgs[key] = this.substituteVariables(value, context);
        } else {
          resolvedArgs[key] = value;
        }
      }
    }

    if (!this.mcpClient) {
      // Return mock result for testing
      return { tool: step.tool, args: resolvedArgs, result: 'mock' };
    }

    return await this.mcpClient.callTool(step.tool, resolvedArgs);
  }

  private async executeConditionStep(
    step: RecipeStep,
    context: RecipeExecutionContext
  ): Promise<void> {
    if (!step.condition) {
      throw new Error(`Condition step ${step.name} has no condition`);
    }

    const conditionResult = this.evaluateCondition(step.condition, context);

    if (conditionResult && step.thenSteps) {
      await this.executeSteps(step.thenSteps, {
        ...context,
        depth: context.depth + 1,
      });
    } else if (!conditionResult && step.elseSteps) {
      await this.executeSteps(step.elseSteps, {
        ...context,
        depth: context.depth + 1,
      });
    }
  }

  private async executeLoopStep(
    step: RecipeStep,
    context: RecipeExecutionContext
  ): Promise<unknown[]> {
    if (!step.iterateOver || !step.loopSteps) {
      throw new Error(`Loop step ${step.name} is incomplete`);
    }

    const items = this.resolveVariable(step.iterateOver, context);
    if (!Array.isArray(items)) {
      throw new Error(`iterateOver must resolve to an array`);
    }

    const results: unknown[] = [];

    for (const item of items) {
      // Create a new context with the loop variable
      const loopContext: RecipeExecutionContext = {
        ...context,
        stepResults: new Map(context.stepResults),
        depth: context.depth + 1,
      };
      loopContext.variables = { ...context.variables, item };

      await this.executeSteps(step.loopSteps, loopContext);
      results.push(Object.fromEntries(loopContext.stepResults));
    }

    return results;
  }

  private async executeParallelStep(
    step: RecipeStep,
    context: RecipeExecutionContext
  ): Promise<unknown[]> {
    if (!step.parallelSteps) {
      throw new Error(`Parallel step ${step.name} has no parallelSteps`);
    }

    const promises = step.parallelSteps.map((subStep) =>
      this.executeStep(subStep, {
        ...context,
        depth: context.depth + 1,
      })
    );

    return await Promise.all(promises);
  }

  private substituteVariables(
    template: string,
    context: RecipeExecutionContext
  ): string {
    return template.replace(/\$\{(\w+)\}/g, (_, key) => {
      const value = this.resolveVariable(key, context);
      return String(value);
    });
  }

  private resolveVariable(
    key: string,
    context: RecipeExecutionContext
  ): unknown {
    if (key in context.variables) {
      return context.variables[key];
    }
    if (context.stepResults.has(key)) {
      return context.stepResults.get(key);
    }
    throw new Error(`Variable not found: ${key}`);
  }

  private evaluateCondition(
    condition: string,
    context: RecipeExecutionContext
  ): boolean {
    // Simple condition evaluation
    // Supports: ${variable} == "value", ${variable} != "value"
    const equalMatch = condition.match(/\$\{(\w+)\}\s*==\s*"([^"]+)"/);
    if (equalMatch) {
      const [, varName, expectedValue] = equalMatch;
      const actualValue = this.resolveVariable(varName, context);
      return String(actualValue) === expectedValue;
    }

    const notEqualMatch = condition.match(/\$\{(\w+)\}\s*!=\s*"([^"]+)"/);
    if (notEqualMatch) {
      const [, varName, expectedValue] = notEqualMatch;
      const actualValue = this.resolveVariable(varName, context);
      return String(actualValue) !== expectedValue;
    }

    // Default: treat as truthy check
    const value = this.resolveVariable(condition.replace(/\$\{|\}/g, ''), context);
    return Boolean(value);
  }
}
