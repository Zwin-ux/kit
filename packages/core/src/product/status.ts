import { access, readdir } from "node:fs/promises";
import path from "node:path";
import { listSkills } from "../library/library.js";
import { getKitHome } from "../library/paths.js";
import {
  LINKABLE_HARNESSES,
  resolveHarnessSkillsRoot,
} from "../paths/harness.js";
import type { HarnessId, PathScope } from "../paths/types.js";

export type HarnessLinkState = "ok" | "partial" | "missing" | "no-root";

export interface HarnessStatusRow {
  harness: HarnessId;
  scope: PathScope;
  root: string;
  exists: boolean;
  skillDirCount: number;
  /** Library skill names present under this root. */
  linkedNames: string[];
  /** Library skill names not found under this root. */
  missingNames: string[];
  state: HarnessLinkState;
}

export interface StatusReport {
  projectDir: string;
  kitHome: string;
  libraryCount: number;
  libraryNames: string[];
  rows: HarnessStatusRow[];
  /** True when every linkable project harness has at least one library skill present. */
  allOk: boolean;
  /** Suggested next command when not allOk. */
  nextCommand: string | null;
  notes: string[];
}

export interface StatusOptions {
  projectDir?: string;
  kitHome?: string;
  homeDir?: string;
  /** Default project only. */
  scope?: PathScope | "both";
  harnesses?: HarnessId[];
}

export type StatusResult =
  | { ok: true; value: StatusReport }
  | { ok: false; error: string };

/**
 * Prove whether Kit library skills are visible to agent harnesses.
 * This is the "are agents actually wired?" check (unlike doctor install health).
 */
export async function runStatus(
  options: StatusOptions = {},
): Promise<StatusResult> {
  const kitHome = options.kitHome ?? getKitHome();
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const scopeOpt = options.scope ?? "project";
  const harnesses = options.harnesses ?? [...LINKABLE_HARNESSES];
  const scopes: PathScope[] =
    scopeOpt === "both" ? ["project", "personal"] : [scopeOpt];

  const listed = await listSkills({ kitHome });
  if (!listed.ok) {
    return { ok: false, error: listed.error };
  }

  const libraryNames = listed.value.map((s) => s.name).sort();
  const librarySet = new Set(libraryNames);
  const rows: HarnessStatusRow[] = [];
  const notes: string[] = [];

  for (const harness of harnesses) {
    for (const scope of scopes) {
      const root = resolveHarnessSkillsRoot(harness, scope, {
        projectDir,
        kitHome,
        ...(options.homeDir !== undefined ? { homeDir: options.homeDir } : {}),
      });
      const exists = await pathExists(root);
      if (!exists) {
        rows.push({
          harness,
          scope,
          root,
          exists: false,
          skillDirCount: 0,
          linkedNames: [],
          missingNames: libraryNames.slice(0, 12),
          state: libraryNames.length === 0 ? "ok" : "no-root",
        });
        continue;
      }

      const dirs = await listSkillDirs(root);
      const linkedNames = dirs.filter((d) => librarySet.has(d)).sort();
      const missingNames = libraryNames
        .filter((n) => !dirs.includes(n))
        .slice(0, 24);

      let state: HarnessLinkState = "missing";
      if (libraryNames.length === 0) {
        state = "ok";
      } else if (linkedNames.length === 0) {
        state = dirs.length === 0 ? "missing" : "missing";
      } else if (missingNames.length === 0) {
        state = "ok";
      } else {
        state = "partial";
      }

      rows.push({
        harness,
        scope,
        root,
        exists: true,
        skillDirCount: dirs.length,
        linkedNames,
        missingNames,
        state,
      });
    }
  }

  const projectRows = rows.filter((r) => r.scope === "project");
  const allOk =
    libraryNames.length === 0 ||
    projectRows.every((r) => r.state === "ok" || r.state === "partial");
  // stricter allOk: every project harness ok
  const strictOk =
    libraryNames.length === 0 ||
    projectRows.every((r) => r.state === "ok");

  const broken = projectRows.filter((r) => r.state !== "ok");
  let nextCommand: string | null = null;
  if (libraryNames.length === 0) {
    nextCommand = "kit ready --write";
    notes.push("Library empty — install a pack first.");
  } else if (broken.length > 0) {
    const first = broken[0]!;
    nextCommand = `kit link --to ${first.harness} --write`;
    notes.push(
      `${broken.length} project harness(es) missing Kit skills. Link to wire them.`,
    );
  }

  return {
    ok: true,
    value: {
      projectDir,
      kitHome,
      libraryCount: libraryNames.length,
      libraryNames,
      rows,
      allOk: strictOk,
      nextCommand,
      notes,
    },
  };
}

async function pathExists(p: string): Promise<boolean> {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
}

async function listSkillDirs(root: string): Promise<string[]> {
  try {
    const entries = await readdir(root, { withFileTypes: true });
    return entries
      .filter((e) => e.isDirectory() && !e.name.startsWith("."))
      .map((e) => e.name)
      .sort();
  } catch {
    return [];
  }
}
