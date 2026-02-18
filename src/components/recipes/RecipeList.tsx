/**
 * RecipeList Component - Display and manage recipes
 */

import { useState, useEffect } from 'react';
import { Plus, Search, BookOpen, Trash2, Edit, Play, Clock, CheckCircle, XCircle } from 'lucide-react';
import { useRecipeStore } from '../../stores/recipeStore';
import type { Recipe } from '../../types/recipe';

interface RecipeListProps {
  onEditRecipe: (recipe: Recipe) => void;
  onExecuteRecipe: (recipe: Recipe) => void;
}

export function RecipeList({ onEditRecipe, onExecuteRecipe }: RecipeListProps) {
  const { recipes, executions, loading, loadRecipes, loadExecutions, deleteRecipe } = useRecipeStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedRecipe, setSelectedRecipe] = useState<Recipe | null>(null);

  useEffect(() => {
    loadRecipes();
    loadExecutions();
  }, [loadRecipes, loadExecutions]);

  const filteredRecipes = recipes.filter(
    (recipe) =>
      recipe.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (recipe.description?.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  const builtinRecipes = filteredRecipes.filter((r) => r.is_builtin);
  const customRecipes = filteredRecipes.filter((r) => !r.is_builtin);

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this recipe?')) {
      await deleteRecipe(id);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-green-600" />;
      case 'failed':
        return <XCircle className="w-4 h-4 text-red-600" />;
      case 'running':
        return <div className="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin" />;
      default:
        return <Clock className="w-4 h-4 text-gray-400" />;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">Recipes</h2>
        </div>

        {/* Search */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search recipes..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Recipe List */}
      <div className="flex-1 overflow-auto p-4">
        {/* Built-in Recipes */}
        {builtinRecipes.length > 0 && (
          <div className="mb-6">
            <h3 className="text-sm font-medium text-gray-500 mb-3">Built-in Recipes</h3>
            <div className="space-y-2">
              {builtinRecipes.map((recipe) => (
                <div
                  key={recipe.id}
                  className="border rounded-lg p-3 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer"
                  onClick={() => setSelectedRecipe(recipe)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <BookOpen className="w-5 h-5 text-purple-600" />
                      <div>
                        <h4 className="font-medium">{recipe.name}</h4>
                        <p className="text-sm text-gray-500">{recipe.description}</p>
                      </div>
                    </div>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onExecuteRecipe(recipe);
                      }}
                      className="px-3 py-1 bg-purple-600 text-white rounded hover:bg-purple-700"
                    >
                      Run
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Custom Recipes */}
        {customRecipes.length > 0 && (
          <div>
            <h3 className="text-sm font-medium text-gray-500 mb-3">Custom Recipes</h3>
            <div className="space-y-2">
              {customRecipes.map((recipe) => (
                <div
                  key={recipe.id}
                  className="border rounded-lg p-3 hover:bg-gray-50 dark:hover:bg-gray-800"
                >
                  <div className="flex items-center justify-between">
                    <div
                      className="flex items-center gap-3 cursor-pointer flex-1"
                      onClick={() => setSelectedRecipe(recipe)}
                    >
                      <BookOpen className="w-5 h-5 text-blue-600" />
                      <div>
                        <h4 className="font-medium">{recipe.name}</h4>
                        <p className="text-sm text-gray-500">{recipe.description}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => onExecuteRecipe(recipe)}
                        className="px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700"
                      >
                        Run
                      </button>
                      <button
                        onClick={() => onEditRecipe(recipe)}
                        className="p-1 hover:bg-gray-100 rounded"
                      >
                        <Edit className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => handleDelete(recipe.id)}
                        className="p-1 hover:bg-red-100 rounded text-red-600"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {filteredRecipes.length === 0 && (
          <div className="text-center py-12 text-gray-500">
            <BookOpen className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>No recipes found</p>
          </div>
        )}

        {/* Recent Executions */}
        {executions.length > 0 && (
          <div className="mt-6">
            <h3 className="text-sm font-medium text-gray-500 mb-3">Recent Executions</h3>
            <div className="space-y-2">
              {executions.slice(0, 5).map((execution) => {
                const recipe = recipes.find((r) => r.id === execution.recipe_id);
                return (
                  <div
                    key={execution.id}
                    className="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800 rounded"
                  >
                    <div className="flex items-center gap-2">
                      {getStatusIcon(execution.status)}
                      <span className="text-sm">{recipe?.name || 'Unknown'}</span>
                    </div>
                    <span className="text-xs text-gray-500">
                      {new Date(execution.started_at).toLocaleString()}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
