import { valid as isValidSemver } from "semver";
import {
  KNOWN_AGENTS,
  type Skill,
  type SkillParseResult,
  type ValidationIssue,
} from "../types.js";
import type { SkillFrontMatterRaw } from "../parse/skillMd.js";

/** Lowercase letters, numbers, and hyphens. No leading or trailing hyphen. */
const NAME_RE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

const KNOWN_AGENT_SET = new Set<string>(KNOWN_AGENTS);

/**
 * Validate raw front matter and body against skill schema v0.
 */
export function validateSkill(
  frontMatter: SkillFrontMatterRaw,
  body: string,
  options?: { rootDir?: string },
): SkillParseResult {
  const issues: ValidationIssue[] = [];

  const name = requireString(frontMatter, "name", issues);
  if (name !== undefined) {
    if (!NAME_RE.test(name)) {
      issues.push({
        field: "name",
        message:
          "name must be lowercase and use only letters, numbers, and hyphens (example: add-readme).",
      });
    }
  }

  const description = requireString(frontMatter, "description", issues);
  if (description !== undefined) {
    validateDescription(description, issues);
  }

  const version = requireString(frontMatter, "version", issues);
  if (version !== undefined) {
    if (!isValidSemver(version)) {
      issues.push({
        field: "version",
        message: `version must be valid semver (example: 0.1.0). Received: ${version}`,
      });
    }
  }

  const compatibility = validateCompatibility(frontMatter.compatibility, issues);

  if (issues.length > 0) {
    return { ok: false, issues };
  }

  // After issue check, required strings are defined.
  const skill: Skill = {
    name: name as string,
    description: description as string,
    version: version as string,
    compatibility: compatibility as string[],
    body,
  };

  if (options?.rootDir !== undefined) {
    skill.rootDir = options.rootDir;
  }

  return { ok: true, skill };
}

function requireString(
  frontMatter: SkillFrontMatterRaw,
  field: "name" | "description" | "version",
  issues: ValidationIssue[],
): string | undefined {
  if (!(field in frontMatter) || frontMatter[field] === undefined) {
    issues.push({
      field,
      message: `${field} is required.`,
    });
    return undefined;
  }

  const value = frontMatter[field];
  if (typeof value !== "string") {
    issues.push({
      field,
      message: `${field} must be a string.`,
    });
    return undefined;
  }

  const trimmed = value.trim();
  if (!trimmed) {
    issues.push({
      field,
      message: `${field} must not be empty.`,
    });
    return undefined;
  }

  return trimmed;
}

/**
 * Description must be one or two non-empty sentences.
 */
function validateDescription(
  description: string,
  issues: ValidationIssue[],
): void {
  const sentenceCount = countSentences(description);
  if (sentenceCount < 1) {
    issues.push({
      field: "description",
      message: "description must contain at least one sentence.",
    });
    return;
  }
  if (sentenceCount > 2) {
    issues.push({
      field: "description",
      message:
        "description must be one or two sentences. Shorten the description.",
    });
  }
}

function countSentences(text: string): number {
  const parts = text
    .split(/(?<=[.!?])\s+/)
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  return parts.length;
}

function validateCompatibility(
  value: unknown,
  issues: ValidationIssue[],
): string[] | undefined {
  if (value === undefined) {
    issues.push({
      field: "compatibility",
      message: "compatibility is required.",
    });
    return undefined;
  }

  if (!Array.isArray(value)) {
    issues.push({
      field: "compatibility",
      message: "compatibility must be a list of agent ids.",
    });
    return undefined;
  }

  if (value.length === 0) {
    issues.push({
      field: "compatibility",
      message:
        "compatibility must list at least one agent (claude-code, grok-build, or codex).",
    });
    return undefined;
  }

  const agents: string[] = [];
  const seen = new Set<string>();

  value.forEach((entry, index) => {
    const field = `compatibility[${index}]`;
    if (typeof entry !== "string") {
      issues.push({
        field,
        message: "each compatibility entry must be a string.",
      });
      return;
    }

    const agent = entry.trim();
    if (!agent) {
      issues.push({
        field,
        message: "compatibility entry must not be empty.",
      });
      return;
    }

    if (!KNOWN_AGENT_SET.has(agent)) {
      issues.push({
        field,
        message: `unknown agent "${agent}". Allowed: ${KNOWN_AGENTS.join(", ")}.`,
      });
      return;
    }

    if (seen.has(agent)) {
      issues.push({
        field,
        message: `duplicate agent "${agent}".`,
      });
      return;
    }

    seen.add(agent);
    agents.push(agent);
  });

  return agents;
}
