/**
 * Skill schema v0 types.
 * See docs/dev/SKILL_SCHEMA.md.
 */

/** Agents that Kit recognizes in compatibility lists. */
export const KNOWN_AGENTS = [
  "claude-code",
  "grok-build",
  "codex",
] as const;

export type KnownAgent = (typeof KNOWN_AGENTS)[number];

/** Parsed and validated skill (schema v0). */
export interface Skill {
  name: string;
  description: string;
  version: string;
  compatibility: string[];
  /** Markdown body after YAML front matter. */
  body: string;
  /** Absolute path to the skill folder, when loaded from disk. */
  rootDir?: string;
}

/** One validation problem with a stable field path. */
export interface ValidationIssue {
  /** Dot path such as "name" or "compatibility". */
  field: string;
  /** Clear message for humans and agents. */
  message: string;
}

/** Outcome of parse + validate. */
export type SkillParseResult =
  | { ok: true; skill: Skill }
  | { ok: false; issues: ValidationIssue[] };
