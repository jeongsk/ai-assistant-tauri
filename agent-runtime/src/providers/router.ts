/**
 * Provider Router - Multi-LLM routing
 */

import { logger } from '../utils/logger.js';
import { BaseProvider, ProviderConfig, Message } from './base.js';
import { OpenAIProvider } from './openai.js';
import { AnthropicProvider } from './anthropic.js';
import { OllamaProvider } from './ollama.js';

export class ProviderRouter {
  private providers: Map<string, BaseProvider> = new Map();
  private activeProviderId: string | null = null;

  async initialize(configs: Record<string, ProviderConfig>, activeId: string): Promise<void> {
    logger.info('Initializing providers', { activeId });

    for (const [id, config] of Object.entries(configs)) {
      let provider: BaseProvider;

      switch (config.type) {
        case 'openai':
          provider = new OpenAIProvider(config);
          break;
        case 'anthropic':
          provider = new AnthropicProvider(config);
          break;
        case 'ollama':
          provider = new OllamaProvider(config);
          break;
        default:
          logger.warn(`Unknown provider type: ${config.type}`);
          continue;
      }

      this.providers.set(id, provider);
      logger.debug(`Registered provider: ${id}`);
    }

    if (this.providers.has(activeId)) {
      this.activeProviderId = activeId;
    } else {
      throw new Error(`Active provider not found: ${activeId}`);
    }
  }

  getActiveProvider(): BaseProvider {
    if (!this.activeProviderId) {
      throw new Error('No active provider set');
    }

    const provider = this.providers.get(this.activeProviderId);
    if (!provider) {
      throw new Error(`Provider not found: ${this.activeProviderId}`);
    }

    return provider;
  }

  setActiveProvider(id: string): void {
    if (!this.providers.has(id)) {
      throw new Error(`Provider not found: ${id}`);
    }
    this.activeProviderId = id;
    logger.info(`Active provider changed to: ${id}`);
  }

  listProviders(): string[] {
    return Array.from(this.providers.keys());
  }
}
