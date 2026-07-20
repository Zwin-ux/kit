import { parse as parseYaml } from "yaml";
import type { ValidationIssue } from "../types.js";

export interface PackFrontMatterRaw {
  name?: unknown;
  title?: unknown;
  description?: unknown;
  version?: unknown;
  tags?: unknown;
  projectTypes?: unknown;
  skills?: unknown;
  [key: string]: unknown;
}

export interface ParsedPackMd {
  frontMatter: PackFrontMatterRaw;
  body: string;
}

const FRONT_MATTER_RE = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?([\s\S]*)$/;

/**
 * Split PACK.md into YAML front matter and markdown body.
 */
export function parsePackMd(content: string): {
  ok: true;
  value: ParsedPackMd;
} | {
  ok: false;
  issues: ValidationIssue[];
} {
  const text = content.replace(/^\uFEFF/, "");

  if (!text.trim()) {
    return {
      ok: false,
      issues: [{ field: "file", message: "PACK.md is empty." }],
    };
  }

  const match = FRONT_MATTER_RE.exec(text);
  if (!match) {
    return {
      ok: false,
      issues: [
        {
          field: "frontMatter",
          message:
            "PACK.md must start with YAML front matter between --- markers.",
        },
      ],
    };
  }

  const yamlBlock = match[1] ?? "";
  const body = (match[2] ?? "").replace(/^\r?\n/, "");

  let parsed: unknown;
  try {
    parsed = parseYaml(yamlBlock);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return {
      ok: false,
      issues: [
        {
          field: "frontMatter",
          message: `YAML front matter is invalid: ${detail}`,
        },
      ],
    };
  }

  if (parsed === null || parsed === undefined) {
    return {
      ok: false,
      issues: [
        { field: "frontMatter", message: "YAML front matter is empty." },
      ],
    };
  }

  if (typeof parsed !== "object" || Array.isArray(parsed)) {
    return {
      ok: false,
      issues: [
        {
          field: "frontMatter",
          message: "YAML front matter must be a mapping of fields.",
        },
      ],
    };
  }

  return {
    ok: true,
    value: {
      frontMatter: parsed as PackFrontMatterRaw,
      body,
    },
  };
}
