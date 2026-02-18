/**
 * Skill type definitions
 */

export interface Skill {
  id: string;
  name: string;
  description: string;
  prompt: string;
  tools: string[]; // JSON array of tool names
  created_at: string;
  updated_at: string;
}

export interface SkillCreateInput {
  id: string;
  name: string;
  description: string;
  prompt: string;
  tools: string[];
}

export interface SkillUpdateInput {
  id: string;
  name: string;
  description: string;
  prompt: string;
  tools: string[];
}

export const SKILL_LIMITS = {
  maxDescriptionLength: 500,
  maxPromptLength: 10240, // 10KB
  maxSkillsPerUser: 100,
} as const;
