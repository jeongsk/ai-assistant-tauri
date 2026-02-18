/**
 * Recipe Types
 */

export interface Recipe {
  id: string;
  name: string;
  description?: string;
  version: string;
  steps: RecipeStep[];
  variables?: Record<string, unknown>;
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

export interface RecipeExecutionContext {
  recipeId: string;
  executionId: string;
  variables: Record<string, unknown>;
  stepResults: Map<string, unknown>;
  depth: number;
}

export interface RecipeResult {
  success: boolean;
  results: Record<string, unknown>;
  error?: string;
}

export const RECIPE_LIMITS = {
  maxSteps: 50,
  maxNestingDepth: 5,
  maxStepTimeout: 60000, // 60 seconds
} as const;
