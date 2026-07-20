/**
 * Local skills engine for Kit.
 * Parse, validate, and manage skills. No TUI code here.
 */

export { KIT_PACKAGE_VERSION } from "@kit-skills/shared";

/** Package identity for consumers. */
export const CORE_PACKAGE_NAME = "@kit-skills/core" as const;

export {
  KNOWN_AGENTS,
  type KnownAgent,
  type Skill,
  type SkillParseResult,
  type ValidationIssue,
} from "./types.js";

export { parseSkillMd, type ParsedSkillMd, type SkillFrontMatterRaw } from "./parse/skillMd.js";
export { validateSkill } from "./validate/skill.js";
export {
  loadSkill,
  parseAndValidateSkillMd,
  formatIssues,
} from "./loadSkill.js";
