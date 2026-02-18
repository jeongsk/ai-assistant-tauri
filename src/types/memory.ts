/**
 * Memory Type Definitions
 */

export type MemoryType = 'episodic' | 'semantic' | 'procedural';

export interface Memory {
  id: string;
  type: MemoryType;
  content: string;
  embedding?: number[];
  metadata?: Record<string, unknown>;
  importance: number;
  createdAt: string;
  lastAccessed?: string;
}

export interface UserPattern {
  id: string;
  patternType: string;
  patternData: Record<string, unknown>;
  confidence: number;
  sampleCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface MemoryCreateInput {
  id: string;
  type: MemoryType;
  content: string;
  metadata?: Record<string, unknown>;
  importance?: number;
}

export interface MemorySearchResult {
  memory: Memory;
  score: number;
}
