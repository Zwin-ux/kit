import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { parseSkillMd } from "../parse/skillMd.js";
import { KNOWN_AGENTS } from "../types.js";

export interface NormalizedSkillMd {
  /** Slug name safe for Kit schema. */
  name: string;
  description: string;
  version: string;
  compatibility: string[];
  body: string;
  /** Full SKILL.md text ready to write. */
  content: string;
  /** What we fixed for the user. */
  fixes: string[];
  /** True if strict Kit validation would pass after normalize. */
  kitReady: boolean;
}

const NAME_RE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

/**
 * Turn messy agent skill folders into Kit-compatible SKILL.md content.
 * Used by `kit unify` so Claude/Codex dump skills become portable.
 */
export function normalizeSkillMd(
  rawContent: string,
  options: { folderName: string },
): NormalizedSkillMd {
  const fixes: string[] = [];
  const text = rawContent.replace(/^\uFEFF/, "");
  const parsed = parseSkillMd(text);

  let fm: Record<string, unknown> = {};
  let body = text.trim();

  if (parsed.ok) {
    fm = { ...parsed.value.frontMatter };
    body = parsed.value.body.trim();
  } else {
    fixes.push("wrapped body with new Kit front matter (no valid YAML block)");
    // If file is pure markdown, keep whole thing as body
    body = text.replace(/^---[\s\S]*?---\s*/, "").trim() || text.trim();
  }

  // Name
  let name = typeof fm.name === "string" ? fm.name.trim() : "";
  if (!name || !NAME_RE.test(name)) {
    const fromFolder = slugify(options.folderName);
    if (name && name !== fromFolder) {
      fixes.push(`name "${name}" → "${fromFolder}"`);
    } else if (!name) {
      fixes.push(`name set from folder → "${fromFolder}"`);
    } else {
      fixes.push(`name normalized → "${fromFolder}"`);
    }
    name = fromFolder;
  }

  // Description
  let description =
    typeof fm.description === "string" ? fm.description.trim() : "";
  if (!description) {
    description = firstSentences(body, 2) || `Skill: ${name}`;
    fixes.push("description inferred from body");
  } else if (description.length > 220 || countSentences(description) > 2) {
    description = firstSentences(description, 2) || description.slice(0, 200);
    fixes.push("description shortened for Kit schema");
  }

  // Version
  let version = typeof fm.version === "string" ? fm.version.trim() : "";
  if (!version || !/^\d+\.\d+\.\d+/.test(version)) {
    version = "0.1.0";
    fixes.push("version defaulted to 0.1.0");
  } else {
    // strip prerelease noise if needed — keep simple semver core
    const m = version.match(/^(\d+\.\d+\.\d+)/);
    if (m && m[1] && m[1] !== version) {
      version = m[1];
      fixes.push(`version trimmed to ${version}`);
    }
  }

  // Compatibility
  let compatibility: string[] = [];
  if (Array.isArray(fm.compatibility)) {
    compatibility = fm.compatibility
      .map((x) => String(x).trim())
      .filter(Boolean);
  }
  if (compatibility.length === 0) {
    compatibility = [...KNOWN_AGENTS];
    fixes.push("compatibility set to claude-code, grok-build, codex");
  }

  if (!body) {
    body = `# ${name}\n\nUse this skill when relevant.\n`;
    fixes.push("empty body stubbed");
  }

  const content = renderSkillMd({
    name,
    description,
    version,
    compatibility,
    body,
  });

  // Kit-ready if name valid, description short-ish, version ok, compat non-empty
  const kitReady =
    NAME_RE.test(name) &&
    description.length >= 12 &&
    description.length <= 280 &&
    countSentences(description) <= 2 &&
    /^\d+\.\d+\.\d+$/.test(version) &&
    compatibility.length > 0 &&
    body.length >= 20;

  return {
    name,
    description,
    version,
    compatibility,
    body,
    content,
    fixes,
    kitReady,
  };
}

export async function writeNormalizedSkill(
  outDir: string,
  normalized: NormalizedSkillMd,
): Promise<string> {
  const skillDir = path.join(outDir, normalized.name);
  await mkdir(skillDir, { recursive: true });
  const file = path.join(skillDir, "SKILL.md");
  await writeFile(file, normalized.content, "utf8");
  return skillDir;
}

function renderSkillMd(input: {
  name: string;
  description: string;
  version: string;
  compatibility: string[];
  body: string;
}): string {
  const compat = input.compatibility.map((c) => `  - ${c}`).join("\n");
  return `---
name: ${input.name}
description: ${yamlEscape(input.description)}
version: ${input.version}
compatibility:
${compat}
---

${input.body.trim()}
`;
}

function yamlEscape(value: string): string {
  // Quote if special characters
  if (/[:#{}[\],&*?|>!%@`]/.test(value) || value.includes('"') || value.includes("'")) {
    return JSON.stringify(value);
  }
  return value;
}

export function slugify(input: string): string {
  const s = input
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .replace(/-{2,}/g, "-");
  return s || "skill";
}

function countSentences(text: string): number {
  return text.split(/[.!?]+/).map((s) => s.trim()).filter(Boolean).length;
}

function firstSentences(text: string, max: number): string {
  const cleaned = text
    .replace(/^#+\s+/gm, "")
    .replace(/\n+/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  if (!cleaned) return "";
  const parts = cleaned.split(/(?<=[.!?])\s+/).filter(Boolean);
  return parts.slice(0, max).join(" ").trim();
}
