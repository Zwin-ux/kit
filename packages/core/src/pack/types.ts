import type { InstalledSkill } from "../library/types.js";
import type { Skill, ValidationIssue } from "../types.js";

/** Parsed starter pack (schema v0). */
export interface SkillPack {
  name: string;
  title: string;
  description: string;
  version: string;
  tags: string[];
  projectTypes: string[];
  /** Ordered skill names from PACK.md. */
  skillNames: string[];
  /** Markdown body after front matter. */
  body: string;
  /** Absolute path to the pack folder. */
  rootDir: string;
}

/** A skill resolved for a pack install. */
export interface ResolvedPackSkill {
  name: string;
  sourceDir: string;
  skill: Skill;
  origin: "pack-local" | "catalog";
}

export interface LoadedPack {
  pack: SkillPack;
  skills: ResolvedPackSkill[];
}

export type PackResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: string; issues?: ValidationIssue[] };

export interface PackListItem {
  name: string;
  title: string;
  description: string;
  version: string;
  skillCount: number;
  tags: string[];
  projectTypes: string[];
  rootDir: string;
}

export interface InstallPackResult {
  pack: SkillPack;
  installed: InstalledSkill[];
  skipped: string[];
}

export interface ApplyPackResult {
  pack: SkillPack;
  projectDir: string;
  installed: InstalledSkill[];
  projectSkillsDir: string;
  appliedPath: string;
}
