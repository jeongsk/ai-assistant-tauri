import { describe, it, expect } from '@jest/globals';
import { BaseProvider, Message, ProviderConfig, ChatOptions, ChatResponse } from './base.js';

// Create a concrete implementation for testing
class TestProvider extends BaseProvider {
  async chat(messages: Message[], options?: ChatOptions): Promise<ChatResponse> {
    return {
      content: 'Test response',
      usage: {
        promptTokens: 10,
        completionTokens: 5,
      },
    };
  }

  async *chatStream(messages: Message[], options?: ChatOptions): AsyncIterable<string> {
    yield 'Test ';
    yield 'stream ';
    yield 'response';
  }
}

describe('BaseProvider', () => {
  const config: ProviderConfig = {
    type: 'test',
    apiKey: 'test-key',
    model: 'test-model',
    enabled: true,
  };

  it('should be instantiated with config', () => {
    const provider = new TestProvider(config);
    expect(provider).toBeInstanceOf(BaseProvider);
  });

  it('should return chat response', async () => {
    const provider = new TestProvider(config);
    const messages: Message[] = [
      { role: 'user', content: 'Hello' },
    ];
    const response = await provider.chat(messages);
    expect(response.content).toBe('Test response');
    expect(response.usage?.promptTokens).toBe(10);
  });

  it('should stream chat response', async () => {
    const provider = new TestProvider(config);
    const messages: Message[] = [
      { role: 'user', content: 'Hello' },
    ];
    const chunks: string[] = [];
    for await (const chunk of provider.chatStream(messages)) {
      chunks.push(chunk);
    }
    expect(chunks.join('')).toBe('Test stream response');
  });
});

describe('Message interface', () => {
  it('should accept valid message roles', () => {
    const userMessage: Message = { role: 'user', content: 'Hello' };
    const assistantMessage: Message = { role: 'assistant', content: 'Hi there' };
    const systemMessage: Message = { role: 'system', content: 'You are helpful' };

    expect(userMessage.role).toBe('user');
    expect(assistantMessage.role).toBe('assistant');
    expect(systemMessage.role).toBe('system');
  });
});

describe('ProviderConfig interface', () => {
  it('should have required fields', () => {
    const config: ProviderConfig = {
      type: 'openai',
      model: 'gpt-4',
      enabled: true,
    };
    expect(config.type).toBe('openai');
    expect(config.model).toBe('gpt-4');
    expect(config.enabled).toBe(true);
  });

  it('should accept optional fields', () => {
    const config: ProviderConfig = {
      type: 'anthropic',
      apiKey: 'sk-test',
      baseUrl: 'https://api.anthropic.com',
      model: 'claude-3',
      enabled: true,
    };
    expect(config.apiKey).toBe('sk-test');
    expect(config.baseUrl).toBe('https://api.anthropic.com');
  });
});
