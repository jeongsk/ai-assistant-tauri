/**
 * Recipe type definitions
 */

export interface Recipe {
  id: string;
  name: string;
  description?: string;
  version: string;
  steps: RecipeStep[];
  variables?: Record<string, unknown>;
  is_builtin: boolean;
  created_at: string;
  updated_at: string;
}

export interface RecipeStep {
  id: string;
  type: 'prompt' | 'tool_call' | 'condition' | 'loop' | 'parallel';
  name: string;
  description?: string;

  // For 'prompt' type
  prompt?: string;

  // For 'tool_call' type
  tool?: string;
  args?: Record<string, unknown>;

  // For 'condition' type
  condition?: string;
  thenSteps?: RecipeStep[];
  elseSteps?: RecipeStep[];

  // For 'loop' type
  iterateOver?: string;
  loopSteps?: RecipeStep[];

  // For 'parallel' type
  parallelSteps?: RecipeStep[];

  // Common
  outputVariable?: string;
  onError?: 'continue' | 'abort' | 'retry';
  retryCount?: number;
}

export interface RecipeExecution {
  id: string;
  recipe_id: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  variables?: Record<string, unknown>;
  result?: string;
  error?: string;
  started_at: string;
  completed_at?: string;
}

export interface RecipeCreateInput {
  id: string;
  name: string;
  description?: string;
  version: string;
  steps: RecipeStep[];
  variables?: Record<string, unknown>;
}

export interface RecipeUpdateInput {
  id: string;
  name: string;
  description?: string;
  version: string;
  steps: RecipeStep[];
  variables?: Record<string, unknown>;
}

export const RECIPE_LIMITS = {
  maxSteps: 50,
  maxNestingDepth: 5,
  maxStepTimeout: 60000, // 60 seconds
} as const;

// Built-in recipe IDs
export const BUILTIN_RECIPE_IDS = {
  CODE_REVIEW: 'builtin:code-review',
  GENERATE_DOCS: 'builtin:generate-docs',
  SUMMARIZE_FILE: 'builtin:summarize-file',
} as const;
