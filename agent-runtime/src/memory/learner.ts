/**
 * Pattern Learner - Learns and tracks user patterns
 */

import { UserPattern, PatternType, PatternInput } from "./types.js";

export interface LearningEvent {
  type: PatternType;
  data: Record<string, any>;
  timestamp: string;
}

export class PatternLearner {
  private patterns: Map<string, UserPattern> = new Map();
  private recentEvents: LearningEvent[] = [];
  private readonly maxEvents: number = 100;
  private readonly confidenceThreshold: number = 0.7;

  constructor() {
    this.initializeDefaultPatterns();
  }

  /**
   * Initialize default pattern categories
   */
  private initializeDefaultPatterns(): void {
    const defaultPatterns: Array<{ id: string; type: PatternType }> = [
      { id: "pref-code-style", type: "preference" },
      { id: "pref-response-length", type: "preference" },
      { id: "pref-language", type: "preference" },
      { id: "workflow-typical", type: "workflow" },
      { id: "comm-style", type: "communication_style" },
      { id: "task-freq", type: "task_frequency" },
      { id: "ctx-pref", type: "context_preference" },
    ];

    for (const { id, type } of defaultPatterns) {
      this.patterns.set(id, {
        id,
        patternType: type,
        patternData: {},
        confidence: 0,
        sampleCount: 0,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      });
    }
  }

  /**
   * Record a learning event
   */
  recordEvent(event: Omit<LearningEvent, "timestamp">): void {
    const fullEvent: LearningEvent = {
      ...event,
      timestamp: new Date().toISOString(),
    };

    this.recentEvents.push(fullEvent);

    // Trim old events
    if (this.recentEvents.length > this.maxEvents) {
      this.recentEvents = this.recentEvents.slice(-this.maxEvents);
    }

    // Update patterns based on event
    this.updatePatternsFromEvent(fullEvent);
  }

  /**
   * Update patterns from an event
   */
  private updatePatternsFromEvent(event: LearningEvent): void {
    const relevantPatterns = Array.from(this.patterns.values()).filter(
      (p) => p.patternType === event.type,
    );

    for (const pattern of relevantPatterns) {
      this.updatePatternData(pattern, event.data);
    }
  }

  /**
   * Update pattern data with new information
   */
  private updatePatternData(
    pattern: UserPattern,
    newData: Record<string, any>,
  ): void {
    const current = pattern.patternData;
    const merged = this.mergeData(current, newData);

    pattern.patternData = merged;
    pattern.sampleCount += 1;
    pattern.confidence = this.calculateConfidence(
      pattern.sampleCount,
      pattern.patternData,
    );
    pattern.updatedAt = new Date().toISOString();
  }

  /**
   * Merge data with frequency tracking
   */
  private mergeData(
    current: Record<string, any>,
    newData: Record<string, any>,
  ): Record<string, any> {
    const result = { ...current };

    for (const [key, value] of Object.entries(newData)) {
      const keyWithFreq = `${key}_freq`;

      if (typeof value === "string" || typeof value === "number") {
        // Track frequency of values
        const valueKey = `${key}:${value}`;

        if (!result._valueFrequencies) {
          result._valueFrequencies = {};
        }

        result._valueFrequencies[valueKey] =
          (result._valueFrequencies[valueKey] || 0) + 1;

        // Update current value if frequency is higher
        const currentFreq =
          result._valueFrequencies[`${key}:${result[key]}`] || 0;
        const newFreq = result._valueFrequencies[valueKey];

        if (newFreq > currentFreq) {
          result[key] = value;
        }
      } else if (typeof value === "object" && value !== null) {
        result[key] = this.mergeData(result[key] || {}, value);
      }
    }

    return result;
  }

  /**
   * Calculate confidence based on sample count and data consistency
   */
  private calculateConfidence(
    sampleCount: number,
    data: Record<string, any>,
  ): number {
    // Base confidence from sample count
    const countConfidence = Math.min(1, sampleCount / 10);

    // Consistency confidence
    let consistencyScore = 0;
    if (data._valueFrequencies) {
      const frequencies = Object.values(data._valueFrequencies) as number[];
      if (frequencies.length > 0) {
        const max = Math.max(...frequencies);
        const sum = frequencies.reduce((a, b) => a + b, 0);
        consistencyScore = max / sum;
      }
    }

    // Weighted combination
    return countConfidence * 0.4 + consistencyScore * 0.6;
  }

  /**
   * Get learned patterns
   */
  getPatterns(): UserPattern[] {
    return Array.from(this.patterns.values())
      .filter((p) => p.confidence >= this.confidenceThreshold)
      .sort((a, b) => b.confidence - a.confidence);
  }

  /**
   * Get patterns by type
   */
  getPatternsByType(type: PatternType): UserPattern[] {
    return this.getPatterns().filter((p) => p.patternType === type);
  }

  /**
   * Get a specific pattern
   */
  getPattern(id: string): UserPattern | undefined {
    return this.patterns.get(id);
  }

  /**
   * Get user preferences (high confidence preferences)
   */
  getUserPreferences(): Record<string, any> {
    const preferences = this.getPatternsByType("preference");
    const result: Record<string, any> = {};

    for (const pref of preferences) {
      Object.assign(result, pref.patternData);
    }

    // Clean up internal data
    delete result._valueFrequencies;

    return result;
  }

  /**
   * Learn from explicit user feedback
   */
  learnFromFeedback(feedback: {
    category: string;
    preference: string;
    value: any;
    explicit: boolean;
  }): void {
    this.recordEvent({
      type: "preference",
      data: {
        category: feedback.category,
        [feedback.preference]: feedback.value,
        explicit: feedback.explicit,
      },
    });
  }

  /**
   * Get recent events for analysis
   */
  getRecentEvents(limit: number = 20): LearningEvent[] {
    return this.recentEvents.slice(-limit);
  }

  /**
   * Clear all learned patterns
   */
  clearPatterns(): void {
    this.patterns.clear();
    this.recentEvents = [];
    this.initializeDefaultPatterns();
  }
}

// Singleton instance
let learnerInstance: PatternLearner | null = null;

export function getPatternLearner(): PatternLearner {
  if (!learnerInstance) {
    learnerInstance = new PatternLearner();
  }
  return learnerInstance;
}
