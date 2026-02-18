/**
 * Skill Manager - Manage and execute skills
 */

import { logger } from '../utils/logger.js';

export interface Skill {
  id: string;
  name: string;
  description: string;
  prompt: string;
  tools: string[];
}

export class SkillManager {
  private skills: Map<string, Skill> = new Map();

  async loadSkills(fetchSkills: () => Promise<Skill[]>): Promise<void> {
    try {
      const skills = await fetchSkills();
      this.skills.clear();
      for (const skill of skills) {
        this.skills.set(skill.id, skill);
      }
      logger.info(`Loaded ${skills.length} skills`);
    } catch (error) {
      logger.error('Failed to load skills', error);
    }
  }

  getSkill(id: string): Skill | undefined {
    return this.skills.get(id);
  }

  getSkillByName(name: string): Skill | undefined {
    for (const skill of this.skills.values()) {
      if (skill.name.toLowerCase() === name.toLowerCase()) {
        return skill;
      }
    }
    return undefined;
  }

  listSkills(): Skill[] {
    return Array.from(this.skills.values());
  }

  /**
   * Build a system prompt with skill context
   */
  buildSkillPrompt(skillId: string, userMessage: string): string {
    const skill = this.skills.get(skillId);
    if (!skill) {
      throw new Error(`Skill not found: ${skillId}`);
    }

    return `You are operating with the "${skill.name}" skill.

Description: ${skill.description}

Instructions:
${skill.prompt}

Available tools: ${skill.tools.length > 0 ? skill.tools.join(', ') : 'none'}

User request: ${userMessage}`;
  }

  /**
   * Check if a tool is allowed for a skill
   */
  isToolAllowed(skillId: string, toolName: string): boolean {
    const skill = this.skills.get(skillId);
    if (!skill) {
      return false;
    }
    // If tools list is empty, allow all tools
    if (skill.tools.length === 0) {
      return true;
    }
    return skill.tools.includes(toolName);
  }

  /**
   * Filter tool calls to only include allowed tools
   */
  filterToolCalls(skillId: string, tools: string[]): string[] {
    const skill = this.skills.get(skillId);
    if (!skill || skill.tools.length === 0) {
      return tools;
    }
    return tools.filter((tool) => skill.tools.includes(tool));
  }
}

// Singleton instance
export const skillManager = new SkillManager();
