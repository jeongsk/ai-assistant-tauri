/**
 * Provider Routing - Intelligent LLM selection
 */

import { logger } from '../utils/logger.js';
import { TaskClassifier, TaskClassification, getTaskClassifier } from './classifier.js';

export type TaskType = 'coding' | 'creative' | 'analysis' | 'chat' | 'research' | 'planning';

export interface RoutingRule {
  id: string;
  name: string;
  description?: string;
  condition: RoutingCondition;
  provider: string;
  model?: string;
  priority: number;
  enabled: boolean;
}

export interface RoutingCondition {
  taskTypes?: TaskType[];
  maxTokens?: number;
  minTokens?: number;
  keywords?: string[];
  complexity?: 'simple' | 'medium' | 'complex';
}

export interface ProviderSelection {
  provider: string;
  model?: string;
  reason: string;
  classification: TaskClassification;
  fallbackChain: string[];
}

export interface RouteOptions {
  preferSpeed?: boolean;
  preferQuality?: boolean;
  maxCost?: number;
  excludeProviders?: string[];
}

// Default routing rules
const DEFAULT_RULES: RoutingRule[] = [
  {
    id: 'rule-code',
    name: 'Code Generation',
    description: 'Route coding tasks to capable models',
    condition: { taskTypes: ['coding'] },
    provider: 'anthropic',
    model: 'claude-3-sonnet',
    priority: 100,
    enabled: true,
  },
  {
    id: 'rule-chat-simple',
    name: 'Simple Chat',
    description: 'Route simple chat to fast models',
    condition: { taskTypes: ['chat'], complexity: 'simple', maxTokens: 500 },
    provider: 'openai',
    model: 'gpt-3.5-turbo',
    priority: 90,
    enabled: true,
  },
  {
    id: 'rule-analysis',
    name: 'Analysis Tasks',
    description: 'Route analysis to reasoning models',
    condition: { taskTypes: ['analysis'] },
    provider: 'openai',
    model: 'gpt-4',
    priority: 85,
    enabled: true,
  },
  {
    id: 'rule-creative',
    name: 'Creative Writing',
    description: 'Route creative tasks to Claude',
    condition: { taskTypes: ['creative'] },
    provider: 'anthropic',
    model: 'claude-3-opus',
    priority: 95,
    enabled: true,
  },
  {
    id: 'rule-research',
    name: 'Research Tasks',
    description: 'Route research to capable models',
    condition: { taskTypes: ['research'] },
    provider: 'anthropic',
    model: 'claude-3-sonnet',
    priority: 80,
    enabled: true,
  },
  {
    id: 'rule-planning',
    name: 'Planning Tasks',
    description: 'Route planning to reasoning models',
    condition: { taskTypes: ['planning'] },
    provider: 'openai',
    model: 'gpt-4',
    priority: 85,
    enabled: true,
  },
];

// Default fallback chain
const DEFAULT_FALLBACK_CHAIN = ['anthropic', 'openai', 'ollama'];

export class ProviderRouter {
  private rules: RoutingRule[] = [...DEFAULT_RULES];
  private classifier: TaskClassifier;
  private fallbackChain: string[];

  constructor() {
    this.classifier = getTaskClassifier();
    this.fallbackChain = [...DEFAULT_FALLBACK_CHAIN];
  }

  /**
   * Route a prompt to the best provider
   */
  route(prompt: string, options?: RouteOptions): ProviderSelection {
    // Classify the task
    const classification = this.classifier.classify(prompt);

    logger.debug('Task classified', {
      type: classification.type,
      confidence: classification.confidence,
      complexity: classification.complexity,
    });

    // Find matching rules
    const matchingRules = this.findMatchingRules(classification, options);

    // Sort by priority
    matchingRules.sort((a, b) => b.priority - a.priority);

    // Build fallback chain
    let fallbackChain = [...this.fallbackChain];
    if (options?.excludeProviders) {
      fallbackChain = fallbackChain.filter(p => !options.excludeProviders!.includes(p));
    }

    // Select best rule
    if (matchingRules.length > 0) {
      const bestRule = matchingRules[0];

      // Ensure selected provider is in fallback chain
      if (!fallbackChain.includes(bestRule.provider)) {
        fallbackChain.unshift(bestRule.provider);
      }

      return {
        provider: bestRule.provider,
        model: bestRule.model,
        reason: `Matched rule: ${bestRule.name}`,
        classification,
        fallbackChain,
      };
    }

    // Default routing based on complexity
    let provider = 'openai';
    let model: string | undefined;

    if (classification.complexity === 'simple' && options?.preferSpeed) {
      provider = 'openai';
      model = 'gpt-3.5-turbo';
    } else if (classification.complexity === 'complex' || options?.preferQuality) {
      provider = 'anthropic';
      model = 'claude-3-opus';
    } else {
      provider = 'anthropic';
      model = 'claude-3-sonnet';
    }

    return {
      provider,
      model,
      reason: `Default routing for ${classification.type} task`,
      classification,
      fallbackChain,
    };
  }

  /**
   * Find rules matching the classification
   */
  private findMatchingRules(
    classification: TaskClassification,
    options?: RouteOptions
  ): RoutingRule[] {
    return this.rules.filter(rule => {
      if (!rule.enabled) return false;

      const cond = rule.condition;

      // Check task types
      if (cond.taskTypes && !cond.taskTypes.includes(classification.type)) {
        return false;
      }

      // Check complexity
      if (cond.complexity && cond.complexity !== classification.complexity) {
        return false;
      }

      // Check token limits
      if (cond.maxTokens && classification.estimatedTokens > cond.maxTokens) {
        return false;
      }
      if (cond.minTokens && classification.estimatedTokens < cond.minTokens) {
        return false;
      }

      // Check excluded providers
      if (options?.excludeProviders?.includes(rule.provider)) {
        return false;
      }

      return true;
    });
  }

  /**
   * Add a custom routing rule
   */
  addRule(rule: RoutingRule): void {
    this.rules.push(rule);
    logger.info('Added routing rule', { id: rule.id, name: rule.name });
  }

  /**
   * Remove a routing rule
   */
  removeRule(id: string): boolean {
    const index = this.rules.findIndex(r => r.id === id);
    if (index !== -1) {
      this.rules.splice(index, 1);
      logger.info('Removed routing rule', { id });
      return true;
    }
    return false;
  }

  /**
   * Get all rules
   */
  getRules(): RoutingRule[] {
    return [...this.rules];
  }

  /**
   * Set fallback chain
   */
  setFallbackChain(chain: string[]): void {
    this.fallbackChain = chain;
    logger.info('Updated fallback chain', { chain });
  }

  /**
   * Get fallback chain
   */
  getFallbackChain(): string[] {
    return [...this.fallbackChain];
  }
}

// Singleton instance
let instance: ProviderRouter | null = null;

export function getProviderRouter(): ProviderRouter {
  if (!instance) {
    instance = new ProviderRouter();
  }
  return instance;
}
