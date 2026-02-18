/**
 * Memory Manager - Context and conversation memory with persistence
 */

import { promises as fs } from 'fs';
import { dirname } from 'path';
import { logger } from '../utils/logger.js';
import { Message } from '../providers/base.js';
import { ContextCompressor } from './compressor.js';
import { PatternLearner } from './learner.js';
import {
  Memory,
  MemoryType,
  MemoryConfig,
  DEFAULT_MEMORY_CONFIG,
  MemoryCreateInput,
} from './types.js';

// Local config for short-term memory
export interface ShortTermMemoryConfig {
  maxMessages: number;
  maxTokens: number;
}

export interface MemoryPersistenceConfig {
  enabled: boolean;
  path: string;
  autoSave: boolean;
  saveInterval: number; // milliseconds
  compressOnSave: boolean;
}

export interface MemoryStats {
  messageCount: number;
  maxMessages: number;
  memoryCount: number;
  patternCount: number;
  lastSaved?: string;
}

export interface SerializedMemory {
  version: string;
  timestamp: string;
  shortTermMemory: Message[];
  longTermMemories: Memory[];
  patterns: any[];
  stats: MemoryStats;
}

const MEMORY_FORMAT_VERSION = '1.0';

export class MemoryManager {
  private shortTermMemory: Message[] = [];
  private longTermMemories: Map<string, Memory> = new Map();
  private config: ShortTermMemoryConfig;
  private persistenceConfig: MemoryPersistenceConfig;
  private saveTimer: NodeJS.Timeout | null = null;
  private compressor: ContextCompressor;
  private learner: PatternLearner;
  private lastSaved: string | null = null;

  constructor(
    shortTermConfig?: Partial<ShortTermMemoryConfig>,
    persistenceConfig?: Partial<MemoryPersistenceConfig>
  ) {
    this.config = {
      maxMessages: shortTermConfig?.maxMessages || 50,
      maxTokens: shortTermConfig?.maxTokens || 128000,
    };

    this.persistenceConfig = {
      enabled: persistenceConfig?.enabled ?? false,
      path: persistenceConfig?.path || './memory.json',
      autoSave: persistenceConfig?.autoSave ?? true,
      saveInterval: persistenceConfig?.saveInterval || 60000, // 1 minute
      compressOnSave: persistenceConfig?.compressOnSave ?? true,
    };

    this.compressor = new ContextCompressor();
    this.learner = new PatternLearner();

    logger.info('MemoryManager initialized', {
      shortTerm: this.config,
      persistence: this.persistenceConfig,
    });

    // Start auto-save if enabled
    if (this.persistenceConfig.enabled && this.persistenceConfig.autoSave) {
      this.startAutoSave();
    }
  }

  /**
   * Initialize memory manager, optionally loading from file
   */
  async initialize(memoryPath?: string): Promise<void> {
    if (memoryPath) {
      this.persistenceConfig.path = memoryPath;
      this.persistenceConfig.enabled = true;
    }

    // Load from file if persistence is enabled
    if (this.persistenceConfig.enabled) {
      await this.loadFromFile();
    }

    logger.debug('Memory initialized', { path: memoryPath || this.persistenceConfig.path });
  }

  /**
   * Get context messages for LLM
   */
  async getContext(): Promise<Message[]> {
    // Return most recent messages within limits
    return this.shortTermMemory.slice(-this.config.maxMessages);
  }

  /**
   * Get enriched context with memories and patterns
   */
  async getEnrichedContext(): Promise<{
    messages: Message[];
    memories: Memory[];
    patterns: Record<string, any>;
  }> {
    const messages = await this.getContext();

    // Get relevant long-term memories
    const memories = this.getRecentMemories(10);

    // Get learned patterns/preferences
    const preferences = this.learner.getUserPreferences();

    return {
      messages,
      memories,
      patterns: preferences,
    };
  }

  /**
   * Add messages to memory
   */
  async addMessages(
    messages: Message[],
    response?: { content: string }
  ): Promise<void> {
    // Add user/assistant messages
    this.shortTermMemory.push(...messages);

    if (response) {
      this.shortTermMemory.push({
        role: 'assistant',
        content: response.content,
      });
    }

    // Trim if exceeds max
    if (this.shortTermMemory.length > this.config.maxMessages) {
      const excess = this.shortTermMemory.length - this.config.maxMessages;

      // Compress old messages before removing
      if (this.persistenceConfig.compressOnSave) {
        const compressed = this.compressor.compressConversation(
          this.shortTermMemory.slice(0, excess)
        );

        // Store as episodic memory
        await this.createMemory({
          id: `compressed-${Date.now()}`,
          type: 'episodic',
          content: compressed,
          importance: 0.5,
        });
      }

      this.shortTermMemory = this.shortTermMemory.slice(excess);
      logger.debug(`Trimmed ${excess} messages from memory`);
    }

    // Learn from patterns
    for (const message of messages) {
      if (message.role === 'user') {
        this.learner.recordEvent({
          type: 'communication_style',
          data: {
            messageLength: message.content.length,
            hasQuestion: message.content.includes('?'),
            hasCode: message.content.includes('```') || /<code>/.test(message.content),
          },
        });
      }
    }

    logger.debug(`Memory size: ${this.shortTermMemory.length} messages`);

    // Auto-save if enabled
    if (this.persistenceConfig.enabled && this.persistenceConfig.autoSave) {
      await this.saveToFile();
    }
  }

  /**
   * Create a long-term memory
   */
  async createMemory(input: MemoryCreateInput): Promise<Memory> {
    const memory: Memory = {
      id: input.id,
      type: input.type,
      content: input.content,
      metadata: input.metadata || {},
      importance: input.importance ?? 0.5,
      createdAt: new Date().toISOString(),
      lastAccessed: new Date().toISOString(),
    };

    this.longTermMemories.set(memory.id, memory);
    logger.debug('Created memory', { id: memory.id, type: memory.type });

    // Auto-save if enabled
    if (this.persistenceConfig.enabled && this.persistenceConfig.autoSave) {
      await this.saveToFile();
    }

    return memory;
  }

  /**
   * Get a memory by ID
   */
  getMemory(id: string): Memory | undefined {
    const memory = this.longTermMemories.get(id);
    if (memory) {
      memory.lastAccessed = new Date().toISOString();
    }
    return memory;
  }

  /**
   * Get recent memories
   */
  getRecentMemories(limit: number = 20): Memory[] {
    return Array.from(this.longTermMemories.values())
      .sort((a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
      )
      .slice(0, limit);
  }

  /**
   * Get memories by type
   */
  getMemoriesByType(type: MemoryType): Memory[] {
    return Array.from(this.longTermMemories.values())
      .filter(m => m.type === type)
      .sort((a, b) => b.importance - a.importance);
  }

  /**
   * Search memories by content
   */
  searchMemories(query: string, limit: number = 10): Memory[] {
    const lowerQuery = query.toLowerCase();

    return Array.from(this.longTermMemories.values())
      .filter(m =>
        m.content.toLowerCase().includes(lowerQuery) ||
        JSON.stringify(m.metadata).toLowerCase().includes(lowerQuery)
      )
      .sort((a, b) => b.importance - a.importance)
      .slice(0, limit);
  }

  /**
   * Delete a memory
   */
  async deleteMemory(id: string): Promise<boolean> {
    const deleted = this.longTermMemories.delete(id);
    if (deleted) {
      logger.debug('Deleted memory', { id });

      // Auto-save if enabled
      if (this.persistenceConfig.enabled && this.persistenceConfig.autoSave) {
        await this.saveToFile();
      }
    }
    return deleted;
  }

  /**
   * Clear short-term memory
   */
  async clear(): Promise<void> {
    this.shortTermMemory = [];
    logger.info('Short-term memory cleared');

    // Auto-save if enabled
    if (this.persistenceConfig.enabled && this.persistenceConfig.autoSave) {
      await this.saveToFile();
    }
  }

  /**
   * Clear all memory (short-term and long-term)
   */
  async clearAll(): Promise<void> {
    this.shortTermMemory = [];
    this.longTermMemories.clear();
    this.learner.clearPatterns();
    logger.info('All memory cleared');

    // Delete persistence file if exists
    if (this.persistenceConfig.enabled) {
      try {
        await fs.unlink(this.persistenceConfig.path);
        logger.info('Deleted memory file', { path: this.persistenceConfig.path });
      } catch {
        // File might not exist, ignore
      }
    }
  }

  /**
   * Get memory stats
   */
  getStats(): MemoryStats {
    return {
      messageCount: this.shortTermMemory.length,
      maxMessages: this.config.maxMessages,
      memoryCount: this.longTermMemories.size,
      patternCount: this.learner.getPatterns().length,
      lastSaved: this.lastSaved || undefined,
    };
  }

  /**
   * Compress short-term memory
   */
  async compress(): Promise<string> {
    const result = this.compressor.compressConversation(this.shortTermMemory);

    // Store compression result as episodic memory
    await this.createMemory({
      id: `compression-${Date.now()}`,
      type: 'episodic',
      content: result,
      importance: 0.3,
    });

    return result;
  }

  /**
   * Get learned patterns
   */
  getPatterns() {
    return this.learner.getPatterns();
  }

  /**
   * Get user preferences
   */
  getUserPreferences(): Record<string, any> {
    return this.learner.getUserPreferences();
  }

  /**
   * Record feedback for learning
   */
  async recordFeedback(feedback: {
    category: string;
    preference: string;
    value: any;
    explicit: boolean;
  }): Promise<void> {
    this.learner.learnFromFeedback(feedback);

    // Auto-save if enabled
    if (this.persistenceConfig.enabled && this.persistenceConfig.autoSave) {
      await this.saveToFile();
    }
  }

  /**
   * Save memory to file
   */
  async saveToFile(): Promise<void> {
    if (!this.persistenceConfig.enabled) {
      return;
    }

    try {
      // Ensure directory exists
      const dir = dirname(this.persistenceConfig.path);
      await fs.mkdir(dir, { recursive: true });

      // Prepare serialized data
      const data: SerializedMemory = {
        version: MEMORY_FORMAT_VERSION,
        timestamp: new Date().toISOString(),
        shortTermMemory: this.shortTermMemory,
        longTermMemories: Array.from(this.longTermMemories.values()),
        patterns: this.learner.getPatterns(),
        stats: this.getStats(),
      };

      // Write to file
      await fs.writeFile(
        this.persistenceConfig.path,
        JSON.stringify(data, null, 2),
        'utf-8'
      );

      this.lastSaved = new Date().toISOString();
      logger.debug('Memory saved to file', {
        path: this.persistenceConfig.path,
        stats: data.stats
      });
    } catch (error: any) {
      logger.error('Failed to save memory to file', {
        path: this.persistenceConfig.path,
        error: error.message
      });
    }
  }

  /**
   * Load memory from file
   */
  async loadFromFile(): Promise<void> {
    if (!this.persistenceConfig.enabled) {
      return;
    }

    try {
      const content = await fs.readFile(this.persistenceConfig.path, 'utf-8');
      const data: SerializedMemory = JSON.parse(content);

      // Version check
      if (data.version !== MEMORY_FORMAT_VERSION) {
        logger.warn('Memory format version mismatch', {
          file: data.version,
          current: MEMORY_FORMAT_VERSION,
        });
      }

      // Restore short-term memory
      this.shortTermMemory = data.shortTermMemory || [];

      // Restore long-term memories
      this.longTermMemories.clear();
      for (const memory of data.longTermMemories || []) {
        this.longTermMemories.set(memory.id, memory);
      }

      // Restore patterns
      if (data.patterns && Array.isArray(data.patterns)) {
        for (const pattern of data.patterns) {
          // Pattern restoration would need PatternLearner to support import
          // For now, just log
          logger.debug('Found pattern in saved memory', { id: pattern.id });
        }
      }

      this.lastSaved = data.timestamp;
      logger.info('Memory loaded from file', {
        path: this.persistenceConfig.path,
        messages: this.shortTermMemory.length,
        memories: this.longTermMemories.size,
      });
    } catch (error: any) {
      if (error.code === 'ENOENT') {
        // File doesn't exist, that's fine for first run
        logger.debug('Memory file not found, starting fresh', {
          path: this.persistenceConfig.path,
        });
      } else {
        logger.error('Failed to load memory from file', {
          path: this.persistenceConfig.path,
          error: error.message,
        });
      }
    }
  }

  /**
   * Start auto-save timer
   */
  private startAutoSave(): void {
    if (this.saveTimer) {
      clearInterval(this.saveTimer);
    }

    this.saveTimer = setInterval(() => {
      this.saveToFile().catch(error => {
        logger.error('Auto-save failed', { error: error.message });
      });
    }, this.persistenceConfig.saveInterval);

    logger.debug('Auto-save started', {
      interval: this.persistenceConfig.saveInterval,
    });
  }

  /**
   * Stop auto-save timer
   */
  private stopAutoSave(): void {
    if (this.saveTimer) {
      clearInterval(this.saveTimer);
      this.saveTimer = null;
      logger.debug('Auto-save stopped');
    }
  }

  /**
   * Export memory for persistence
   */
  export(): SerializedMemory {
    return {
      version: MEMORY_FORMAT_VERSION,
      timestamp: new Date().toISOString(),
      shortTermMemory: this.shortTermMemory,
      longTermMemories: Array.from(this.longTermMemories.values()),
      patterns: this.learner.getPatterns(),
      stats: this.getStats(),
    };
  }

  /**
   * Import memory from persistence
   */
  import(data: SerializedMemory): void {
    this.shortTermMemory = data.shortTermMemory || [];

    this.longTermMemories.clear();
    for (const memory of data.longTermMemories || []) {
      this.longTermMemories.set(memory.id, memory);
    }

    // Note: Pattern import would require PatternLearner to support it
    logger.info('Imported memory', {
      messages: this.shortTermMemory.length,
      memories: this.longTermMemories.size,
    });
  }

  /**
   * Close memory manager
   */
  async close(): Promise<void> {
    this.stopAutoSave();

    // Save to file before closing
    if (this.persistenceConfig.enabled) {
      await this.saveToFile();
    }

    logger.info('Memory manager closed');
  }
}

// Singleton instance
let memoryManagerInstance: MemoryManager | null = null;

export function getMemoryManager(
  config?: Partial<ShortTermMemoryConfig>,
  persistenceConfig?: Partial<MemoryPersistenceConfig>
): MemoryManager {
  if (!memoryManagerInstance) {
    memoryManagerInstance = new MemoryManager(config, persistenceConfig);
  }
  return memoryManagerInstance;
}
