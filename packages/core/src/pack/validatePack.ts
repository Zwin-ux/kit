import { valid as isValidSemver } from "semver";
import type { ValidationIssue } from "../types.js";
import type { PackFrontMatterRaw } from "./parsePackMd.js";
import type { SkillPack } from "./types.js";

const NAME_RE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

/**
 * Validate pack front matter and build a SkillPack value.
 */
export function validatePackFrontMatter(
  frontMatter: PackFrontMatterRaw,
  body: string,
  rootDir: string,
): { ok: true; pack: SkillPack } | { ok: false; issues: ValidationIssue[] } {
  const issues: ValidationIssue[] = [];

  const name = requireString(frontMatter, "name", issues);
  if (name !== undefined && !NAME_RE.test(name)) {
    issues.push({
      field: "name",
      message:
        "name must be lowercase and use only letters, numbers, and hyphens.",
    });
  }

  const title = requireString(frontMatter, "title", issues);
  const description = requireString(frontMatter, "description", issues);
  if (description !== undefined) {
    const sentences = countSentences(description);
    if (sentences < 1 || sentences > 2) {
      issues.push({
        field: "description",
        message: "description must be one or two sentences.",
      });
    }
  }

  const version = requireString(frontMatter, "version", issues);
  if (version !== undefined && !isValidSemver(version)) {
    issues.push({
      field: "version",
      message: `version must be valid semver. Received: ${version}`,
    });
  }

  const tags = optionalStringList(frontMatter.tags, "tags", issues) ?? [];
  const projectTypes =
    optionalStringList(frontMatter.projectTypes, "projectTypes", issues) ?? [];
  const extendsList =
    optionalStringList(frontMatter.extends, "extends", issues) ?? [];
  const ownSkillNames = optionalSkillNames(frontMatter.skills, issues) ?? [];

  if (ownSkillNames.length === 0 && extendsList.length === 0) {
    issues.push({
      field: "skills",
      message:
        "skills must list at least one skill, or set extends to inherit from another pack.",
    });
  }

  for (const [i, base] of extendsList.entries()) {
    if (name !== undefined && base === name) {
      issues.push({
        field: `extends[${i}]`,
        message: "a pack cannot extend itself.",
      });
    }
  }

  if (issues.length > 0) {
    return { ok: false, issues };
  }

  return {
    ok: true,
    pack: {
      name: name as string,
      title: title as string,
      description: description as string,
      version: version as string,
      tags,
      projectTypes,
      extends: extendsList,
      ownSkillNames,
      // Filled by loadPack after resolving extends.
      skillNames: [...ownSkillNames],
      body,
      rootDir,
    },
  };
}

function requireString(
  frontMatter: PackFrontMatterRaw,
  field: "name" | "title" | "description" | "version",
  issues: ValidationIssue[],
): string | undefined {
  if (!(field in frontMatter) || frontMatter[field] === undefined) {
    issues.push({ field, message: `${field} is required.` });
    return undefined;
  }
  const value = frontMatter[field];
  if (typeof value !== "string") {
    issues.push({ field, message: `${field} must be a string.` });
    return undefined;
  }
  const trimmed = value.trim();
  if (!trimmed) {
    issues.push({ field, message: `${field} must not be empty.` });
    return undefined;
  }
  return trimmed;
}

function optionalStringList(
  value: unknown,
  field: string,
  issues: ValidationIssue[],
): string[] | undefined {
  if (value === undefined) return [];
  if (!Array.isArray(value)) {
    issues.push({ field, message: `${field} must be a list of strings.` });
    return undefined;
  }
  const items: string[] = [];
  value.forEach((entry, index) => {
    if (typeof entry !== "string" || !entry.trim()) {
      issues.push({
        field: `${field}[${index}]`,
        message: `${field} entries must be non-empty strings.`,
      });
      return;
    }
    items.push(entry.trim());
  });
  return items;
}

/** Own skills list — may be empty when `extends` supplies the base set. */
function optionalSkillNames(
  value: unknown,
  issues: ValidationIssue[],
): string[] | undefined {
  if (value === undefined) {
    return [];
  }
  if (!Array.isArray(value)) {
    issues.push({
      field: "skills",
      message: "skills must be a list of skill names.",
    });
    return undefined;
  }

  const names: string[] = [];
  const seen = new Set<string>();
  value.forEach((entry, index) => {
    const field = `skills[${index}]`;
    if (typeof entry !== "string" || !entry.trim()) {
      issues.push({
        field,
        message: "each skill entry must be a non-empty string.",
      });
      return;
    }
    const name = entry.trim();
    if (!NAME_RE.test(name)) {
      issues.push({
        field,
        message: `invalid skill name "${name}".`,
      });
      return;
    }
    if (seen.has(name)) {
      issues.push({ field, message: `duplicate skill "${name}".` });
      return;
    }
    seen.add(name);
    names.push(name);
  });
  return names;
}

function countSentences(text: string): number {
  return text
    .split(/(?<=[.!?])\s+/)
    .map((part) => part.trim())
    .filter((part) => part.length > 0).length;
}
