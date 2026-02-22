/**
 * Memory Manager Tests
 */

import { describe, it, expect, beforeEach, afterEach } from "@jest/globals";
import { MemoryManager } from "./manager.js";
import { Message } from "../providers/base.js";
import { promises as fs } from "fs";

const TEST_MEMORY_PATH = "/tmp/test-memory.json";

describe("MemoryManager", () => {
  let manager: MemoryManager;

  beforeEach(() => {
    manager = new MemoryManager(
      { maxMessages: 10 },
      { enabled: true, path: TEST_MEMORY_PATH, autoSave: false },
    );
  });

  afterEach(async () => {
    await manager.close();
    try {
      await fs.unlink(TEST_MEMORY_PATH);
    } catch {
      // Ignore
    }
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const defaultManager = new MemoryManager();
      expect(defaultManager).toBeDefined();
    });
  });

  describe("initialization", () => {
    it("should initialize without path", async () => {
      await expect(manager.initialize()).resolves.toBeUndefined();
    });

    it("should load from file if exists", async () => {
      // Create a test memory file
      const testData = {
        version: "1.0",
        timestamp: new Date().toISOString(),
        shortTermMemory: [
          { role: "user", content: "Hello" },
          { role: "assistant", content: "Hi there!" },
        ],
        longTermMemories: [],
        patterns: [],
        stats: {
          messageCount: 2,
          maxMessages: 10,
          memoryCount: 0,
          patternCount: 0,
        },
      };

      await fs.writeFile(TEST_MEMORY_PATH, JSON.stringify(testData), "utf-8");

      const newManager = new MemoryManager(
        { maxMessages: 10 },
        { enabled: true, path: TEST_MEMORY_PATH, autoSave: false },
      );
      await newManager.initialize();

      const context = await newManager.getContext();
      expect(context).toHaveLength(2);
      expect(context[0].content).toBe("Hello");

      await newManager.close();
    });
  });

  describe("short-term memory", () => {
    it("should add messages", async () => {
      const messages: Message[] = [{ role: "user", content: "Test message" }];

      await manager.addMessages(messages);

      const context = await manager.getContext();
      expect(context).toHaveLength(1);
      expect(context[0].content).toBe("Test message");
    });

    it("should add messages with response", async () => {
      const messages: Message[] = [{ role: "user", content: "Test message" }];

      await manager.addMessages(messages, { content: "Response" });

      const context = await manager.getContext();
      expect(context).toHaveLength(2);
      expect(context[0].role).toBe("user");
      expect(context[1].role).toBe("assistant");
    });

    it("should trim messages when exceeding max", async () => {
      const maxMessages = 10;
      const manager2 = new MemoryManager(
        { maxMessages: maxMessages },
        { enabled: false },
      );

      // Add more than max messages
      for (let i = 0; i < maxMessages + 5; i++) {
        await manager2.addMessages([{ role: "user", content: `Message ${i}` }]);
      }

      const context = await manager2.getContext();
      expect(context.length).toBeLessThanOrEqual(maxMessages);
    });

    it("should clear short-term memory", async () => {
      await manager.addMessages([{ role: "user", content: "Test" }]);
      await manager.clear();

      const context = await manager.getContext();
      expect(context).toHaveLength(0);
    });
  });

  describe("long-term memory", () => {
    it("should create a long-term memory", async () => {
      const memory = await manager.createMemory({
        id: "test-1",
        type: "episodic",
        content: "Important fact to remember",
        importance: 0.8,
      });

      expect(memory.id).toBe("test-1");
      expect(memory.content).toBe("Important fact to remember");
      expect(memory.importance).toBe(0.8);
    });

    it("should get a memory by ID", async () => {
      await manager.createMemory({
        id: "test-2",
        type: "semantic",
        content: "Knowledge",
      });

      const memory = manager.getMemory("test-2");
      expect(memory).toBeDefined();
      expect(memory?.content).toBe("Knowledge");
    });

    it("should return undefined for non-existent memory", () => {
      const memory = manager.getMemory("non-existent");
      expect(memory).toBeUndefined();
    });

    it("should get recent memories", async () => {
      await manager.createMemory({
        id: "mem-1",
        type: "episodic",
        content: "First",
      });

      await manager.createMemory({
        id: "mem-2",
        type: "episodic",
        content: "Second",
      });

      await manager.createMemory({
        id: "mem-3",
        type: "episodic",
        content: "Third",
      });

      const recent = manager.getRecentMemories(2);
      expect(recent).toHaveLength(2);
    });

    it("should get memories by type", async () => {
      await manager.createMemory({
        id: "epi-1",
        type: "episodic",
        content: "Event",
      });

      await manager.createMemory({
        id: "sem-1",
        type: "semantic",
        content: "Fact",
      });

      const episodic = manager.getMemoriesByType("episodic");
      const semantic = manager.getMemoriesByType("semantic");

      expect(episodic).toHaveLength(1);
      expect(semantic).toHaveLength(1);
      expect(episodic[0].id).toBe("epi-1");
    });

    it("should search memories", async () => {
      await manager.createMemory({
        id: "search-1",
        type: "semantic",
        content: "Python programming language",
      });

      await manager.createMemory({
        id: "search-2",
        type: "semantic",
        content: "JavaScript frameworks",
      });

      const results = manager.searchMemories("python");
      expect(results).toHaveLength(1);
      expect(results[0].id).toBe("search-1");
    });

    it("should delete a memory", async () => {
      await manager.createMemory({
        id: "delete-1",
        type: "episodic",
        content: "To be deleted",
      });

      const deleted = await manager.deleteMemory("delete-1");
      expect(deleted).toBe(true);

      const memory = manager.getMemory("delete-1");
      expect(memory).toBeUndefined();
    });

    it("should return false when deleting non-existent memory", async () => {
      const deleted = await manager.deleteMemory("non-existent");
      expect(deleted).toBe(false);
    });
  });

  describe("persistence", () => {
    it("should save to file", async () => {
      await manager.addMessages([{ role: "user", content: "Save test" }]);
      await manager.saveToFile();

      const content = await fs.readFile(TEST_MEMORY_PATH, "utf-8");
      const data = JSON.parse(content);

      expect(data.version).toBe("1.0");
      expect(data.shortTermMemory).toHaveLength(1);
      expect(data.shortTermMemory[0].content).toBe("Save test");
    });

    it("should load from file", async () => {
      // Create and save
      await manager.addMessages([{ role: "user", content: "Load test" }]);
      await manager.saveToFile();

      // Create new manager and load
      const manager2 = new MemoryManager(
        { maxMessages: 10 },
        { enabled: true, path: TEST_MEMORY_PATH, autoSave: false },
      );
      await manager2.initialize();

      const context = await manager2.getContext();
      expect(context).toHaveLength(1);
      expect(context[0].content).toBe("Load test");

      await manager2.close();
    });

    it("should export and import memory", async () => {
      await manager.addMessages([{ role: "user", content: "Export test" }]);
      await manager.createMemory({
        id: "export-1",
        type: "semantic",
        content: "Exported memory",
      });

      const exported = manager.export();
      expect(exported.shortTermMemory).toHaveLength(1);
      expect(exported.longTermMemories).toHaveLength(1);

      const manager2 = new MemoryManager(
        { maxMessages: 10 },
        { enabled: false },
      );
      manager2.import(exported);

      const context = await manager2.getContext();
      expect(context).toHaveLength(1);
      expect(context[0].content).toBe("Export test");
    });
  });

  describe("enriched context", () => {
    it("should get enriched context with memories and patterns", async () => {
      await manager.addMessages([{ role: "user", content: "Hello" }]);

      const enriched = await manager.getEnrichedContext();

      expect(enriched.messages).toBeDefined();
      expect(enriched.memories).toBeDefined();
      expect(enriched.patterns).toBeDefined();
      expect(enriched.messages).toBeInstanceOf(Array);
      expect(enriched.memories).toBeInstanceOf(Array);
    });
  });

  describe("compression", () => {
    it("should compress conversation history", async () => {
      for (let i = 0; i < 5; i++) {
        await manager.addMessages([{ role: "user", content: `Message ${i}` }]);
      }

      const compressed = await manager.compress();
      expect(compressed).toBeDefined();
      expect(typeof compressed).toBe("string");

      // Compression should create a memory
      const stats = manager.getStats();
      expect(stats.memoryCount).toBeGreaterThan(0);
    });
  });

  describe("patterns and learning", () => {
    it("should get user preferences", () => {
      const preferences = manager.getUserPreferences();
      expect(preferences).toBeDefined();
      expect(typeof preferences).toBe("object");
    });

    it("should get learned patterns", () => {
      const patterns = manager.getPatterns();
      expect(Array.isArray(patterns)).toBe(true);
    });

    it("should record feedback", async () => {
      await expect(
        manager.recordFeedback({
          category: "code",
          preference: "language",
          value: "TypeScript",
          explicit: true,
        }),
      ).resolves.toBeUndefined();
    });
  });

  describe("stats", () => {
    it("should get memory stats", async () => {
      await manager.addMessages([{ role: "user", content: "Test" }]);
      await manager.createMemory({
        id: "stats-1",
        type: "episodic",
        content: "Stats test",
      });

      const stats = manager.getStats();
      expect(stats.messageCount).toBe(1);
      expect(stats.memoryCount).toBe(1);
      expect(stats.maxMessages).toBe(10);
      expect(stats.patternCount).toBeGreaterThanOrEqual(0);
    });
  });

  describe("clear all", () => {
    it("should clear all memory including persistence file", async () => {
      await manager.addMessages([{ role: "user", content: "Test" }]);
      await manager.createMemory({
        id: "clear-1",
        type: "episodic",
        content: "To be cleared",
      });
      await manager.saveToFile();

      await manager.clearAll();

      const stats = manager.getStats();
      expect(stats.messageCount).toBe(0);
      expect(stats.memoryCount).toBe(0);

      // File should be deleted
      await expect(fs.readFile(TEST_MEMORY_PATH)).rejects.toThrow();
    });
  });

  describe("close", () => {
    it("should stop auto-save timer on close", async () => {
      const managerWithAutoSave = new MemoryManager(
        { maxMessages: 10 },
        { enabled: false, autoSave: true, saveInterval: 1000 },
      );

      await expect(managerWithAutoSave.close()).resolves.toBeUndefined();
    });
  });
});
