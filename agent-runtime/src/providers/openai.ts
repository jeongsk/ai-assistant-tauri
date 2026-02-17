/**
 * OpenAI Provider
 */

import { BaseProvider, Message, ChatOptions, ChatResponse, ProviderConfig } from './base.js';
import { logger } from '../utils/logger.js';

export class OpenAIProvider extends BaseProvider {
  constructor(config: ProviderConfig) {
    super(config);
    logger.info('OpenAI provider initialized', { model: config.model });
  }

  async chat(messages: Message[], options?: ChatOptions): Promise<ChatResponse> {
    // TODO: Implement OpenAI API call
    logger.debug('OpenAI chat', { messageCount: messages.length });
    
    return {
      content: '[OpenAI response placeholder]',
      usage: { promptTokens: 0, completionTokens: 0 },
    };
  }

  async *chatStream(messages: Message[], options?: ChatOptions): AsyncIterable<string> {
    // TODO: Implement streaming
    yield '[OpenAI stream placeholder]';
  }
}
