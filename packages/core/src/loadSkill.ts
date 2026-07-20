import { readFile } from "node:fs/promises";
import path from "node:path";
import { parseSkillMd } from "./parse/skillMd.js";
import type { SkillParseResult, ValidationIssue } from "./types.js";
import { validateSkill } from "./validate/skill.js";

/**
 * Load a skill folder: read SKILL.md, parse front matter, validate schema v0.
 */
export async function loadSkill(
  skillDir: string,
): Promise<SkillParseResult> {
  const rootDir = path.resolve(skillDir);
  const skillFile = path.join(rootDir, "SKILL.md");

  let content: string;
  try {
    content = await readFile(skillFile, "utf8");
  } catch (error) {
    const code =
      error && typeof error === "object" && "code" in error
        ? String((error as { code: unknown }).code)
        : undefined;

    if (code === "ENOENT") {
      return {
        ok: false,
        issues: [
          {
            field: "file",
            message: `SKILL.md is missing in ${rootDir}`,
          },
        ],
      };
    }

    const detail = error instanceof Error ? error.message : String(error);
    return {
      ok: false,
      issues: [
        {
          field: "file",
          message: `Cannot read SKILL.md: ${detail}`,
        },
      ],
    };
  }

  return parseAndValidateSkillMd(content, { rootDir });
}

/**
 * Parse and validate SKILL.md text without reading the filesystem.
 */
export function parseAndValidateSkillMd(
  content: string,
  options?: { rootDir?: string },
): SkillParseResult {
  const parsed = parseSkillMd(content);
  if (!parsed.ok) {
    return { ok: false, issues: parsed.issues };
  }

  return validateSkill(parsed.value.frontMatter, parsed.value.body, options);
}

/**
 * Format validation issues as a single multi-line string.
 */
export function formatIssues(issues: ValidationIssue[]): string {
  return issues.map((issue) => `${issue.field}: ${issue.message}`).join("\n");
}
