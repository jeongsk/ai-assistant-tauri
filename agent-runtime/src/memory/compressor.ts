/**
 * Context Compressor - Compresses and summarizes context
 */

import { Memory, ContextCompressionResult } from "./types.js";

export class ContextCompressor {
  private readonly maxTokens: number;
  private readonly summaryThreshold: number;

  constructor(maxTokens: number = 4000, summaryThreshold: number = 0.7) {
    this.maxTokens = maxTokens;
    this.summaryThreshold = summaryThreshold;
  }

  /**
   * Compress memories into a summary
   */
  compress(memories: Memory[]): ContextCompressionResult {
    if (memories.length === 0) {
      return {
        summary: "",
        keyPoints: [],
        tokensSaved: 0,
      };
    }

    // Sort by importance
    const sorted = [...memories].sort((a, b) => b.importance - a.importance);

    // Extract key points
    const keyPoints = this.extractKeyPoints(sorted);

    // Generate summary
    const summary = this.generateSummary(sorted, keyPoints);

    // Calculate tokens saved (rough estimate)
    const originalTokens = memories.reduce(
      (sum, m) => sum + this.estimateTokens(m.content),
      0,
    );
    const compressedTokens = this.estimateTokens(summary);
    const tokensSaved = Math.max(0, originalTokens - compressedTokens);

    return {
      summary,
      keyPoints,
      tokensSaved,
    };
  }

  /**
   * Extract key points from memories
   */
  private extractKeyPoints(memories: Memory[]): string[] {
    const points: string[] = [];
    const seenPhrases = new Set<string>();

    for (const memory of memories) {
      // Extract sentences
      const sentences = memory.content.split(/[.!?]+/).filter((s) => s.trim());

      for (const sentence of sentences) {
        const trimmed = sentence.trim();

        // Skip short or duplicate sentences
        if (trimmed.length < 10) continue;
        if (seenPhrases.has(trimmed.toLowerCase())) continue;

        // Check if it's a key point based on keywords
        if (this.isKeyPoint(trimmed)) {
          points.push(trimmed);
          seenPhrases.add(trimmed.toLowerCase());
        }

        // Limit key points
        if (points.length >= 10) break;
      }

      if (points.length >= 10) break;
    }

    return points;
  }

  /**
   * Check if a sentence is a key point
   */
  private isKeyPoint(sentence: string): boolean {
    const keyIndicators = [
      "important",
      "remember",
      "key",
      "critical",
      "essential",
      "must",
      "should",
      "need",
      "require",
      "goal",
      "objective",
      "prefer",
      "always",
      "never",
      "favorite",
      "default",
    ];

    const lower = sentence.toLowerCase();
    return keyIndicators.some((indicator) => lower.includes(indicator));
  }

  /**
   * Generate summary from memories and key points
   */
  private generateSummary(memories: Memory[], keyPoints: string[]): string {
    const parts: string[] = [];

    // Group by type
    const byType = this.groupByType(memories);

    // Add episodic summary
    if (byType.episodic.length > 0) {
      parts.push(
        `Recent activities: ${byType.episodic.length} events recorded.`,
      );
    }

    // Add semantic summary
    if (byType.semantic.length > 0) {
      parts.push(`Knowledge: ${byType.semantic.length} facts stored.`);
    }

    // Add procedural summary
    if (byType.procedural.length > 0) {
      parts.push(`Patterns: ${byType.procedural.length} workflows learned.`);
    }

    // Add key points
    if (keyPoints.length > 0) {
      parts.push("\nKey points:");
      keyPoints.slice(0, 5).forEach((point, i) => {
        parts.push(`${i + 1}. ${point}`);
      });
    }

    return parts.join("\n");
  }

  /**
   * Group memories by type
   */
  private groupByType(memories: Memory[]): Record<string, Memory[]> {
    return {
      episodic: memories.filter((m) => m.type === "episodic"),
      semantic: memories.filter((m) => m.type === "semantic"),
      procedural: memories.filter((m) => m.type === "procedural"),
    };
  }

  /**
   * Estimate token count (rough approximation)
   */
  private estimateTokens(text: string): number {
    // Rough estimate: ~4 characters per token
    return Math.ceil(text.length / 4);
  }

  /**
   * Compress conversation history
   */
  compressConversation(
    messages: Array<{ role: string; content: string }>,
  ): string {
    if (messages.length === 0) return "";

    // Keep recent messages intact
    const recentCount = Math.min(3, messages.length);
    const recentMessages = messages.slice(-recentCount);

    // Summarize older messages
    const olderMessages = messages.slice(0, -recentCount);

    if (olderMessages.length === 0) {
      return recentMessages.map((m) => `${m.role}: ${m.content}`).join("\n");
    }

    const summary = this.summarizeMessages(olderMessages);
    const recent = recentMessages
      .map((m) => `${m.role}: ${m.content}`)
      .join("\n");

    return `[Earlier context: ${summary}]\n\n${recent}`;
  }

  /**
   * Summarize a list of messages
   */
  private summarizeMessages(
    messages: Array<{ role: string; content: string }>,
  ): string {
    const userMessages = messages.filter((m) => m.role === "user");
    const assistantMessages = messages.filter((m) => m.role === "assistant");

    const topics = this.extractTopics(userMessages.map((m) => m.content));

    return (
      `Discussed ${topics.length} topics: ${topics.slice(0, 3).join(", ")}. ` +
      `${userMessages.length} questions, ${assistantMessages.length} responses.`
    );
  }

  /**
   * Extract topics from content
   */
  private extractTopics(contents: string[]): string[] {
    const words = contents.join(" ").toLowerCase().split(/\s+/);
    const stopWords = new Set([
      "the",
      "a",
      "an",
      "is",
      "are",
      "was",
      "were",
      "be",
      "been",
      "being",
      "have",
      "has",
      "had",
      "do",
      "does",
      "did",
      "will",
      "would",
      "could",
      "should",
      "may",
      "might",
      "must",
      "shall",
      "can",
      "need",
      "to",
      "of",
      "in",
      "for",
      "on",
      "with",
      "at",
      "by",
      "from",
      "as",
      "into",
      "through",
      "during",
      "before",
      "after",
      "above",
      "below",
      "between",
      "under",
      "again",
      "further",
      "then",
      "once",
      "here",
      "there",
      "when",
      "where",
      "why",
      "how",
      "all",
      "each",
      "few",
      "more",
      "most",
      "other",
      "some",
      "such",
      "no",
      "nor",
      "not",
      "only",
      "own",
      "same",
      "so",
      "than",
      "too",
      "very",
      "just",
      "and",
      "but",
      "if",
      "or",
      "because",
      "until",
      "while",
      "what",
      "which",
      "who",
      "whom",
      "this",
      "that",
      "these",
      "those",
      "i",
      "me",
      "my",
      "we",
      "our",
    ]);

    const wordFreq: Record<string, number> = {};
    for (const word of words) {
      const cleaned = word.replace(/[^a-z]/g, "");
      if (cleaned.length > 3 && !stopWords.has(cleaned)) {
        wordFreq[cleaned] = (wordFreq[cleaned] || 0) + 1;
      }
    }

    return Object.entries(wordFreq)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([word]) => word);
  }
}

// Singleton instance
let compressorInstance: ContextCompressor | null = null;

export function getContextCompressor(): ContextCompressor {
  if (!compressorInstance) {
    compressorInstance = new ContextCompressor();
  }
  return compressorInstance;
}
