/**
 * Task Classifier - Classifies tasks for intelligent routing
 */

export type TaskType = 'coding' | 'creative' | 'analysis' | 'chat' | 'research' | 'planning';

export interface TaskClassification {
  type: TaskType;
  confidence: number;
  keywords: string[];
  complexity: 'simple' | 'medium' | 'complex';
  estimatedTokens: number;
}

// Keywords for each task type
const TASK_KEYWORDS: Record<TaskType, string[]> = {
  coding: [
    'code', 'function', 'class', 'method', 'variable', 'bug', 'fix',
    'implement', 'refactor', 'debug', 'compile', 'syntax', 'api',
    'typescript', 'javascript', 'python', 'rust', 'go', 'react',
    'component', 'module', 'import', 'export', 'async', 'await'
  ],
  creative: [
    'write', 'story', 'poem', 'creative', 'imagine', 'narrative',
    'fiction', 'blog', 'article', 'content', 'marketing', 'brand',
    'slogan', 'headline', 'script', 'dialogue'
  ],
  analysis: [
    'analyze', 'analysis', 'review', 'compare', 'evaluate', 'assess',
    'examine', 'investigate', 'study', 'metrics', 'data', 'statistics',
    'report', 'summary', 'insights', 'trends', 'patterns'
  ],
  chat: [
    'hello', 'hi', 'hey', 'thanks', 'thank', 'please', 'help',
    'what', 'how', 'why', 'when', 'where', 'can you', 'could you'
  ],
  research: [
    'research', 'find', 'search', 'look up', 'information', 'learn',
    'explore', 'discover', 'sources', 'references', 'documentation',
    'explain', 'describe', 'overview'
  ],
  planning: [
    'plan', 'strategy', 'roadmap', 'schedule', 'organize', 'structure',
    'steps', 'workflow', 'process', 'breakdown', 'tasks', 'goals',
    'objectives', 'milestones'
  ]
};

// Complexity indicators
const COMPLEXITY_INDICATORS = {
  simple: ['quick', 'simple', 'basic', 'short', 'brief'],
  complex: ['complex', 'detailed', 'comprehensive', 'thorough', 'extensive', 'complete']
};

export class TaskClassifier {
  /**
   * Classify a prompt/task
   */
  classify(prompt: string): TaskClassification {
    const lowerPrompt = prompt.toLowerCase();
    const words = lowerPrompt.split(/\s+/);

    // Count keyword matches for each type
    const scores: Record<TaskType, number> = {
      coding: 0,
      creative: 0,
      analysis: 0,
      chat: 0,
      research: 0,
      planning: 0,
    };

    const matchedKeywords: string[] = [];

    for (const [type, keywords] of Object.entries(TASK_KEYWORDS)) {
      for (const keyword of keywords) {
        if (lowerPrompt.includes(keyword)) {
          scores[type as TaskType]++;
          matchedKeywords.push(keyword);
        }
      }
    }

    // Find best match
    let bestType: TaskType = 'chat';
    let bestScore = 0;

    for (const [type, score] of Object.entries(scores)) {
      if (score > bestScore) {
        bestScore = score;
        bestType = type as TaskType;
      }
    }

    // Calculate confidence
    const totalMatches = Object.values(scores).reduce((a, b) => a + b, 0);
    const confidence = totalMatches > 0 ? bestScore / totalMatches : 0.3;

    // Determine complexity
    let complexity: 'simple' | 'medium' | 'complex' = 'medium';

    for (const indicator of COMPLEXITY_INDICATORS.simple) {
      if (lowerPrompt.includes(indicator)) {
        complexity = 'simple';
        break;
      }
    }

    for (const indicator of COMPLEXITY_INDICATORS.complex) {
      if (lowerPrompt.includes(indicator)) {
        complexity = 'complex';
        break;
      }
    }

    // Override complexity based on prompt length
    if (words.length < 10) {
      complexity = 'simple';
    } else if (words.length > 100) {
      complexity = 'complex';
    }

    // Estimate tokens (rough approximation)
    const estimatedTokens = Math.ceil(words.length * 1.3);

    return {
      type: bestType,
      confidence,
      keywords: [...new Set(matchedKeywords)].slice(0, 10),
      complexity,
      estimatedTokens,
    };
  }

  /**
   * Check if task requires reasoning capabilities
   */
  requiresReasoning(classification: TaskClassification): boolean {
    return ['coding', 'analysis', 'planning'].includes(classification.type);
  }

  /**
   * Check if task is time-sensitive
   */
  isTimeSensitive(classification: TaskClassification): boolean {
    return classification.type === 'chat' && classification.complexity === 'simple';
  }
}

// Singleton instance
let instance: TaskClassifier | null = null;

export function getTaskClassifier(): TaskClassifier {
  if (!instance) {
    instance = new TaskClassifier();
  }
  return instance;
}
