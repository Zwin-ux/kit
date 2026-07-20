import os from "node:os";
import path from "node:path";
import type { HarnessId, PathScope } from "./types.js";

/**
 * Resolve the skills root directory for a harness + scope.
 * Paths follow public conventions (Claude Code, Codex) and Kit’s own layout.
 */
export function resolveHarnessSkillsRoot(
  harness: HarnessId,
  scope: PathScope,
  options: {
    projectDir: string;
    kitHome: string;
    homeDir?: string;
  },
): string {
  const home = options.homeDir ?? os.homedir();
  const projectDir = path.resolve(options.projectDir);

  switch (harness) {
    case "kit":
      return scope === "personal"
        ? path.join(options.kitHome, "skills")
        : path.join(projectDir, ".kit", "skills");

    case "claude-code":
      // https://code.claude.com/docs/en/skills
      // Personal: ~/.claude/skills  Project: .claude/skills
      return scope === "personal"
        ? path.join(home, ".claude", "skills")
        : path.join(projectDir, ".claude", "skills");

    case "codex":
      // Global: ~/.codex/skills  (project-local not standard; we still offer .codex/skills)
      return scope === "personal"
        ? path.join(home, ".codex", "skills")
        : path.join(projectDir, ".codex", "skills");

    case "grok-build":
      // Kit convention for Grok Build / Grok CLI skills until official docs lock a path.
      return scope === "personal"
        ? path.join(home, ".grok", "skills")
        : path.join(projectDir, ".grok", "skills");

    default: {
      const _exhaustive: never = harness;
      return _exhaustive;
    }
  }
}

export function harnessNotes(harness: HarnessId, scope: PathScope): string {
  switch (harness) {
    case "kit":
      return scope === "personal"
        ? "Kit global library (kit list / kit install)."
        : "Kit project skills after kit pack apply (./.kit/skills).";
    case "claude-code":
      return scope === "personal"
        ? "Claude Code personal skills (~/.claude/skills)."
        : "Claude Code project skills (./.claude/skills).";
    case "codex":
      return scope === "personal"
        ? "Codex global skills (~/.codex/skills)."
        : "Project-local Codex skills folder (optional layout).";
    case "grok-build":
      return scope === "personal"
        ? "Grok Build personal skills (~/.grok/skills) — Kit convention."
        : "Grok Build project skills (./.grok/skills) — Kit convention.";
    default: {
      const _exhaustive: never = harness;
      return _exhaustive;
    }
  }
}

export const ALL_HARNESSES: HarnessId[] = [
  "kit",
  "claude-code",
  "codex",
  "grok-build",
];

export const LINKABLE_HARNESSES: HarnessId[] = [
  "claude-code",
  "codex",
  "grok-build",
];
