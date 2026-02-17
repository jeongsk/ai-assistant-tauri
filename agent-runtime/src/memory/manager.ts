/**
 * Memory Manager - Context and conversation memory
 */

import { logger } from '../utils/logger.js';
import { Message } from '../providers/base.js';

export interface MemoryConfig {
  maxMessages: number;
  maxTokens: number;
}

export class MemoryManager {
  private shortTermMemory: Message[] = [];
  private config: MemoryConfig;
  private memoryPath: string | null = null;

  constructor(config?: Partial<MemoryConfig>) {
    this.config = {
      maxMessages: config?.maxMessages || 50,
      maxTokens: config?.maxTokens || 128000,
    };
    logger.info('MemoryManager initialized', this.config);
  }

  async initialize(memoryPath?: string): Promise<void> {
    this.memoryPath = memoryPath || null;
    logger.debug('Memory initialized', { path: memoryPath });
  }

  /**
   * Get context messages for LLM
   */
  async getContext(): Promise<Message[]> {
    return this.shortTermMemory.slice(-this.config.maxMessages);
  }

  /**
   * Add messages to memory
   */
  async addMessages(messages: Message[], response?: { content: string }): Promise<void> {
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
      this.shortTermMemory = this.shortTermMemory.slice(excess);
      logger.debug(`Trimmed ${excess} messages from memory`);
    }

    logger.debug(`Memory size: ${this.shortTermMemory.length} messages`);
  }

  /**
   * Clear short-term memory
   */
  async clear(): Promise<void> {
    this.shortTermMemory = [];
    logger.info('Memory cleared');
  }

  /**
   * Get memory stats
   */
  getStats(): { messageCount: number; maxMessages: number } {
    return {
      messageCount: this.shortTermMemory.length,
      maxMessages: this.config.maxMessages,
    };
  }

  /**
   * Export memory for persistence
   */
  export(): Message[] {
    return [...this.shortTermMemory];
  }

  /**
   * Import memory from persistence
   */
  import(messages: Message[]): void {
    this.shortTermMemory = messages;
    logger.info(`Imported ${messages.length} messages into memory`);
  }

  /**
   * Close memory manager
   */
  async close(): Promise<void> {
    // TODO: Persist to file if memoryPath is set
    logger.info('Memory manager closed');
  }
}
