/**
 * Type definitions
 */

export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  isStreaming?: boolean;
}

export interface Conversation {
  id: string;
  title: string;
  messages: Message[];
  createdAt: Date;
  updatedAt: Date;
}

export interface ProviderConfig {
  type: 'openai' | 'anthropic' | 'ollama';
  apiKey?: string;
  baseUrl?: string;
  model: string;
  enabled: boolean;
}

export interface FolderPermission {
  id: string;
  path: string;
  level: 'read' | 'readwrite';
}

export type Theme = 'light' | 'dark' | 'system';
