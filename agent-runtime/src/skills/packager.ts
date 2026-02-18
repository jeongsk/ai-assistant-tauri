/**
 * Skill Packager - Export and import skills as packages
 */

import { logger } from "../utils/logger.js";
import crypto from "crypto";

export interface SkillPackage {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  prompt: string;
  tools: string[];
  variables?: Record<string, any>;
  tags: string[];
  checksum: string;
  exportedAt: string;
  compatibilityVersion: string;
}

export interface SkillMetadata {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  tags: string[];
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  securityIssues: string[];
}

const COMPATIBILITY_VERSION = "1.0.0";

export class SkillPackager {
  /**
   * Export a skill to package format
   */
  export(
    skill: {
      id: string;
      name: string;
      description: string;
      prompt: string;
      tools: string[];
      variables?: Record<string, any>;
    },
    author: string = "local",
  ): SkillPackage {
    const pkg: SkillPackage = {
      id: skill.id,
      name: skill.name,
      version: "1.0.0",
      description: skill.description,
      author,
      prompt: skill.prompt,
      tools: skill.tools,
      variables: skill.variables,
      tags: [],
      checksum: "",
      exportedAt: new Date().toISOString(),
      compatibilityVersion: COMPATIBILITY_VERSION,
    };

    // Generate checksum
    pkg.checksum = this.generateChecksum(pkg);

    logger.info("Exported skill package", { id: skill.id, name: skill.name });
    return pkg;
  }

  /**
   * Import a skill from package format
   */
  async import(pkg: SkillPackage): Promise<{
    id: string;
    name: string;
    description: string;
    prompt: string;
    tools: string[];
    variables?: Record<string, any>;
  }> {
    // Validate package
    const validation = this.validate(pkg);
    if (!validation.valid) {
      throw new Error(`Invalid skill package: ${validation.errors.join(", ")}`);
    }

    if (validation.securityIssues.length > 0) {
      logger.warn("Security issues in skill package", {
        issues: validation.securityIssues,
      });
    }

    // Generate new ID to avoid conflicts
    const importedId = `imported-${pkg.id}-${Date.now()}`;

    logger.info("Imported skill package", {
      originalId: pkg.id,
      newId: importedId,
    });

    return {
      id: importedId,
      name: pkg.name,
      description: pkg.description,
      prompt: pkg.prompt,
      tools: pkg.tools,
      variables: pkg.variables,
    };
  }

  /**
   * Validate a skill package
   */
  validate(pkg: SkillPackage): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    const securityIssues: string[] = [];

    // Required fields
    if (!pkg.id) errors.push("Missing id");
    if (!pkg.name) errors.push("Missing name");
    if (!pkg.prompt) errors.push("Missing prompt");

    // Field length limits
    if (pkg.name && pkg.name.length > 100) {
      errors.push("Name too long (max 100 chars)");
    }
    if (pkg.description && pkg.description.length > 500) {
      warnings.push("Description is long (max 500 chars recommended)");
    }
    if (pkg.prompt && pkg.prompt.length > 10240) {
      errors.push("Prompt too long (max 10KB)");
    }

    // Validate tools
    if (pkg.tools && !Array.isArray(pkg.tools)) {
      errors.push("Tools must be an array");
    }

    // Security checks
    if (pkg.prompt) {
      // Check for potential injection patterns
      const injectionPatterns = [
        /ignore\s+previous\s+instructions/i,
        /system\s*:/i,
        /<\|.*?\|>/,
        /execute\s+code/i,
      ];

      for (const pattern of injectionPatterns) {
        if (pattern.test(pkg.prompt)) {
          securityIssues.push(
            `Potential prompt injection pattern detected: ${pattern.source}`,
          );
        }
      }
    }

    // Verify checksum if present
    if (pkg.checksum) {
      const { checksum, ...rest } = pkg;
      const expectedChecksum = this.generateChecksum(rest as any);
      if (pkg.checksum !== expectedChecksum) {
        errors.push("Checksum verification failed");
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      securityIssues,
    };
  }

  /**
   * Generate checksum for package
   */
  private generateChecksum(pkg: Omit<SkillPackage, "checksum">): string {
    const content = JSON.stringify({
      id: pkg.id,
      name: pkg.name,
      version: pkg.version,
      prompt: pkg.prompt,
      tools: pkg.tools,
    });
    return crypto
      .createHash("sha256")
      .update(content)
      .digest("hex")
      .substring(0, 16);
  }
}

// Singleton instance
let instance: SkillPackager | null = null;

export function getSkillPackager(): SkillPackager {
  if (!instance) {
    instance = new SkillPackager();
  }
  return instance;
}
