// Provider Types

export type ProviderType = 'openai' | 'anthropic' | 'ollama';

export interface ProviderConfig {
  type: ProviderType;
  enabled: boolean;
  apiKey?: string;
  baseUrl?: string;
  model?: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: ProviderType;
  contextWindow: number;
  supportsStreaming: boolean;
  supportsVision: boolean;
}

export const AVAILABLE_MODELS: ModelInfo[] = [
  // OpenAI
  {
    id: 'gpt-4o',
    name: 'GPT-4o',
    provider: 'openai',
    contextWindow: 128000,
    supportsStreaming: true,
    supportsVision: true,
  },
  {
    id: 'gpt-4-turbo',
    name: 'GPT-4 Turbo',
    provider: 'openai',
    contextWindow: 128000,
    supportsStreaming: true,
    supportsVision: true,
  },
  {
    id: 'gpt-3.5-turbo',
    name: 'GPT-3.5 Turbo',
    provider: 'openai',
    contextWindow: 16385,
    supportsStreaming: true,
    supportsVision: false,
  },
  // Anthropic
  {
    id: 'claude-sonnet-4-20250514',
    name: 'Claude Sonnet 4',
    provider: 'anthropic',
    contextWindow: 200000,
    supportsStreaming: true,
    supportsVision: true,
  },
  {
    id: 'claude-3-5-sonnet-20241022',
    name: 'Claude 3.5 Sonnet',
    provider: 'anthropic',
    contextWindow: 200000,
    supportsStreaming: true,
    supportsVision: true,
  },
  {
    id: 'claude-3-opus-20240229',
    name: 'Claude 3 Opus',
    provider: 'anthropic',
    contextWindow: 200000,
    supportsStreaming: true,
    supportsVision: true,
  },
];

export function getModelsByProvider(provider: ProviderType): ModelInfo[] {
  return AVAILABLE_MODELS.filter((m) => m.provider === provider);
}
