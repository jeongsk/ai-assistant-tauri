/**
 * Base Provider Interface
 */

export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

export interface ProviderConfig {
  type: string;
  apiKey?: string;
  baseUrl?: string;
  model: string;
  enabled: boolean;
}

export interface ChatOptions {
  maxTokens?: number;
  temperature?: number;
  stream?: boolean;
}

export interface ChatResponse {
  content: string;
  toolCalls?: any[];
  usage?: {
    promptTokens: number;
    completionTokens: number;
  };
}

export abstract class BaseProvider {
  protected config: ProviderConfig;

  constructor(config: ProviderConfig) {
    this.config = config;
  }

  abstract chat(messages: Message[], options?: ChatOptions): Promise<ChatResponse>;
  abstract chatStream(messages: Message[], options?: ChatOptions): AsyncIterable<string>;
}
