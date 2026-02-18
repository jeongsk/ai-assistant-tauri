/**
 * API Types for Tauri ↔ Agent Runtime Communication
 * These types must match the Rust side structures
 */

export type MessageRole = 'user' | 'assistant' | 'system';

export interface ApiMessage {
  role: MessageRole;
  content: string;
}

export interface ApiChatOptions {
  provider?: string;
  model?: string;
  stream?: boolean;
}

export interface ApiChatRequest {
  messages: ApiMessage[];
  options?: ApiChatOptions;
}

export interface ApiChatResponse {
  content: string;
  error?: string;
}

export interface ApiToolCallRequest {
  tool: string;
  args: Record<string, unknown>;
}

export interface ApiToolCallResponse {
  result: unknown;
  error?: string;
}

export interface ApiInitializeRequest {
  config: {
    providers: Array<{
      name: string;
      api_key?: string;
      base_url?: string;
    }>;
    active_provider: string;
    mcp_servers?: Array<Record<string, unknown>>;
    memory_path?: string;
  };
}

export interface ApiInitializeResponse {
  status: 'initialized' | 'ready';
}
