/**
 * Built-in Recipes
 */

import type { Recipe } from './types.js';

export const BUILTIN_RECIPES: Recipe[] = [
  {
    id: 'builtin:code-review',
    name: 'Code Review',
    description: 'Review code files for quality, security, and best practices',
    version: '1.0.0',
    steps: [
      {
        id: 'step1',
        type: 'prompt',
        name: 'Analyze Code',
        prompt: `Review the following code file for:
1. Code quality
2. Security issues
3. Best practices
4. Performance concerns

File: \${file_path}

Provide actionable feedback.`,
        outputVariable: 'analysis',
      },
      {
        id: 'step2',
        type: 'prompt',
        name: 'Summarize Findings',
        prompt: `Based on the analysis:
\${analysis}

Create a concise summary with severity levels.`,
        outputVariable: 'summary',
      },
    ],
    variables: {
      file_path: '',
    },
  },
  {
    id: 'builtin:generate-docs',
    name: 'Generate Documentation',
    description: 'Generate documentation for code files',
    version: '1.0.0',
    steps: [
      {
        id: 'step1',
        type: 'prompt',
        name: 'Analyze Code Structure',
        prompt: `Analyze the code in \${file_path} and identify:
- Functions/methods
- Classes
- Interfaces
- Key functionality`,
        outputVariable: 'structure',
      },
      {
        id: 'step2',
        type: 'prompt',
        name: 'Generate Documentation',
        prompt: `Generate comprehensive documentation for:
\${structure}

Format in Markdown with proper headings and code examples.`,
        outputVariable: 'documentation',
      },
    ],
    variables: {
      file_path: '',
    },
  },
  {
    id: 'builtin:summarize-file',
    name: 'Summarize File',
    description: "Create a summary of a file's contents",
    version: '1.0.0',
    steps: [
      {
        id: 'step1',
        type: 'tool_call',
        name: 'Read File',
        tool: 'read_file',
        args: { path: '\${file_path}' },
        outputVariable: 'file_content',
      },
      {
        id: 'step2',
        type: 'prompt',
        name: 'Summarize',
        prompt: `Summarize the following content in a clear, concise manner:

\${file_content}`,
        outputVariable: 'summary',
      },
    ],
    variables: {
      file_path: '',
    },
  },
];

/**
 * Get a built-in recipe by ID
 */
export function getBuiltinRecipe(id: string): Recipe | undefined {
  return BUILTIN_RECIPES.find((r) => r.id === id);
}

/**
 * List all built-in recipes
 */
export function listBuiltinRecipes(): Recipe[] {
  return [...BUILTIN_RECIPES];
}
