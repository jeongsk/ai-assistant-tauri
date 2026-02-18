/**
 * Memory System Type Definitions
 */

export type MemoryType = 'episodic' | 'semantic' | 'procedural';

export interface Memory {
  id: string;
  type: MemoryType;
  content: string;
  embedding?: number[];
  metadata?: Record<string, any>;
  importance: number;
  createdAt: string;
  lastAccessed?: string;
}

export interface UserPattern {
  id: string;
  patternType: string;
  patternData: Record<string, any>;
  confidence: number;
  sampleCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface MemoryCreateInput {
  id: string;
  type: MemoryType;
  content: string;
  metadata?: Record<string, any>;
  importance?: number;
}

export interface MemorySearchQuery {
  query: string;
  type?: MemoryType;
  limit?: number;
  threshold?: number;
}

export interface MemorySearchResult {
  memory: Memory;
  score: number;
}

export interface ContextCompressionResult {
  summary: string;
  keyPoints: string[];
  tokensSaved: number;
}

// Memory configuration
export interface MemoryConfig {
  maxMemories: number;
  maxAge: number; // in milliseconds
  importanceThreshold: number;
  compressionEnabled: boolean;
  embeddingDimension: number;
}

export const DEFAULT_MEMORY_CONFIG: MemoryConfig = {
  maxMemories: 10000,
  maxAge: 30 * 24 * 60 * 60 * 1000, // 30 days
  importanceThreshold: 0.3,
  compressionEnabled: true,
  embeddingDimension: 384, // all-MiniLM-L6-v2 dimension
};

// Pattern types for user learning
export type PatternType =
  | 'preference'
  | 'workflow'
  | 'communication_style'
  | 'task_frequency'
  | 'context_preference';

export interface PatternInput {
  id: string;
  patternType: PatternType;
  data: Record<string, any>;
}
