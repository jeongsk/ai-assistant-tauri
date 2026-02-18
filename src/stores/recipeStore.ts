/**
 * Recipe Store - Zustand state management for recipes
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Recipe, RecipeExecution, RecipeCreateInput, RecipeUpdateInput, RecipeStep } from '../types/recipe';

interface RecipeState {
  recipes: Recipe[];
  executions: RecipeExecution[];
  loading: boolean;
  executing: boolean;
  error: string | null;

  // Actions
  loadRecipes: () => Promise<void>;
  createRecipe: (recipe: RecipeCreateInput) => Promise<void>;
  updateRecipe: (recipe: RecipeUpdateInput) => Promise<void>;
  deleteRecipe: (id: string) => Promise<void>;
  getRecipe: (id: string) => Recipe | undefined;

  // Execution
  loadExecutions: (recipeId?: string) => Promise<void>;
  executeRecipe: (recipeId: string, variables?: Record<string, unknown>) => Promise<string>;
}

export const useRecipeStore = create<RecipeState>((set, get) => ({
  recipes: [],
  executions: [],
  loading: false,
  executing: false,
  error: null,

  loadRecipes: async () => {
    set({ loading: true, error: null });
    try {
      const rawRecipes = await invoke<Array<{
        id: string;
        name: string;
        description: string | null;
        version: string;
        steps: string;
        variables: string | null;
        is_builtin: number;
        created_at: string;
        updated_at: string;
      }>>('list_recipes');

      const recipes: Recipe[] = rawRecipes.map((r) => ({
        id: r.id,
        name: r.name,
        description: r.description || undefined,
        version: r.version,
        steps: JSON.parse(r.steps || '[]'),
        variables: r.variables ? JSON.parse(r.variables) : undefined,
        is_builtin: r.is_builtin !== 0,
        created_at: r.created_at,
        updated_at: r.updated_at,
      }));

      set({ recipes, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  createRecipe: async (input: RecipeCreateInput) => {
    try {
      await invoke('create_recipe', {
        id: input.id,
        name: input.name,
        description: input.description || null,
        version: input.version,
        steps: JSON.stringify(input.steps),
        variables: input.variables ? JSON.stringify(input.variables) : null,
      });

      await get().loadRecipes();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  updateRecipe: async (input: RecipeUpdateInput) => {
    try {
      await invoke('update_recipe', {
        id: input.id,
        name: input.name,
        description: input.description || null,
        version: input.version,
        steps: JSON.stringify(input.steps),
        variables: input.variables ? JSON.stringify(input.variables) : null,
      });

      await get().loadRecipes();
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  deleteRecipe: async (id: string) => {
    try {
      await invoke('delete_recipe', { id });

      set((state) => ({
        recipes: state.recipes.filter((r) => r.id !== id),
      }));
    } catch (error) {
      set({ error: String(error) });
      throw error;
    }
  },

  getRecipe: (id: string) => {
    return get().recipes.find((r) => r.id === id);
  },

  loadExecutions: async (recipeId?: string) => {
    try {
      const rawExecutions = await invoke<Array<{
        id: string;
        recipe_id: string;
        status: string;
        variables: string | null;
        result: string | null;
        error: string | null;
        started_at: string;
        completed_at: string | null;
      }>>('list_recipe_executions', { recipeId: recipeId || null });

      const executions: RecipeExecution[] = rawExecutions.map((e) => ({
        id: e.id,
        recipe_id: e.recipe_id,
        status: e.status as RecipeExecution['status'],
        variables: e.variables ? JSON.parse(e.variables) : undefined,
        result: e.result || undefined,
        error: e.error || undefined,
        started_at: e.started_at,
        completed_at: e.completed_at || undefined,
      }));

      set({ executions });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  executeRecipe: async (recipeId: string, variables?: Record<string, unknown>) => {
    const executionId = crypto.randomUUID();

    set({ executing: true, error: null });

    try {
      // Create execution record
      await invoke('create_recipe_execution', {
        id: executionId,
        recipeId,
        variables: variables ? JSON.stringify(variables) : null,
      });

      // TODO: Actually execute the recipe via agent runtime
      // For now, just mark as completed
      await invoke('update_recipe_execution', {
        id: executionId,
        status: 'completed',
        result: 'Recipe execution not yet implemented',
        error: null,
      });

      await get().loadExecutions(recipeId);

      return executionId;
    } catch (error) {
      // Update execution as failed
      await invoke('update_recipe_execution', {
        id: executionId,
        status: 'failed',
        result: null,
        error: String(error),
      });

      set({ error: String(error), executing: false });
      throw error;
    } finally {
      set({ executing: false });
    }
  },
}));
