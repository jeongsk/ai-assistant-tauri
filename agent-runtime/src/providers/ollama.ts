/**
 * Ollama Provider (Local LLM)
 */

import { BaseProvider, Message, ChatOptions, ChatResponse, ProviderConfig } from './base.js';
import { logger } from '../utils/logger.js';

interface OllamaResponse {
  message?: {
    content: string;
  };
  prompt_eval_count?: number;
  eval_count?: number;
  done?: boolean;
}

interface OllamaModel {
  name: string;
}

interface OllamaTagsResponse {
  models?: OllamaModel[];
}

export class OllamaProvider extends BaseProvider {
  private baseUrl: string;

  constructor(config: ProviderConfig) {
    super(config);
    this.baseUrl = config.baseUrl || 'http://localhost:11434';
    logger.info('Ollama provider initialized', { 
      model: config.model, 
      baseUrl: this.baseUrl 
    });
  }

  async chat(messages: Message[], options?: ChatOptions): Promise<ChatResponse> {
    logger.debug('Ollama chat', { messageCount: messages.length, model: this.config.model });

    const response = await fetch(`${this.baseUrl}/api/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: this.config.model,
        messages: messages.map(m => ({
          role: m.role,
          content: m.content,
        })),
        stream: false,
        options: {
          num_predict: options?.maxTokens || 4096,
          temperature: options?.temperature ?? 0.7,
        },
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Ollama API error: ${error}`);
    }

    const data = await response.json() as OllamaResponse;

    return {
      content: data.message?.content || '',
      usage: {
        promptTokens: data.prompt_eval_count || 0,
        completionTokens: data.eval_count || 0,
      },
    };
  }

  async *chatStream(messages: Message[], options?: ChatOptions): AsyncIterable<string> {
    const response = await fetch(`${this.baseUrl}/api/chat`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        model: this.config.model,
        messages: messages.map(m => ({
          role: m.role,
          content: m.content,
        })),
        stream: true,
        options: {
          num_predict: options?.maxTokens || 4096,
          temperature: options?.temperature ?? 0.7,
        },
      }),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Ollama API error: ${error}`);
    }

    const reader = response.body?.getReader();
    if (!reader) throw new Error('No response body');

    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';

      for (const line of lines) {
        if (!line.trim()) continue;
        
        try {
          const data = JSON.parse(line) as OllamaResponse;
          if (data.message?.content) {
            yield data.message.content;
          }
          if (data.done) {
            return;
          }
        } catch {
          // Skip invalid JSON
        }
      }
    }
  }

  /**
   * List available models
   */
  async listModels(): Promise<string[]> {
    const response = await fetch(`${this.baseUrl}/api/tags`);
    
    if (!response.ok) {
      throw new Error('Failed to fetch Ollama models');
    }

    const data = await response.json() as OllamaTagsResponse;
    return data.models?.map((m) => m.name) || [];
  }

  /**
   * Check if Ollama is running
   */
  async isAvailable(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}/api/tags`, {
        method: 'HEAD',
      });
      return response.ok;
    } catch {
      return false;
    }
  }
}
