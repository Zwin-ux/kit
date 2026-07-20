import { access, readdir } from "node:fs/promises";
import path from "node:path";
import { installSkill, listSkills } from "../library/library.js";
import { loadSkill } from "../loadSkill.js";
import { getKitHome } from "../library/paths.js";
import {
  LINKABLE_HARNESSES,
  resolveHarnessSkillsRoot,
} from "./harness.js";
import type {
  HarnessId,
  ImportPlanItem,
  ImportResult,
  PathScope,
  PathsResult,
} from "./types.js";

const SKIP_DIR_NAMES = new Set([
  ".system",
  "node_modules",
  ".git",
  "dist",
]);

export interface ImportSkillsOptions {
  /** Source harnesses. Default: all linkable (not kit). */
  harnesses?: HarnessId[];
  /** personal = user home harness dirs; project = under projectDir. Default: personal */
  scope?: PathScope;
  projectDir?: string;
  kitHome?: string;
  homeDir?: string;
  /**
   * When false (default), only plan — no library writes.
   * When true, install into ~/.kit library.
   */
  write?: boolean;
  /** Replace existing library installs with the same name. */
  force?: boolean;
  /** Limit to these skill folder names. */
  skillNames?: string[];
}

/**
 * Scan agent harness skill folders and optionally install them into the Kit library.
 * Inverse of linkSkills. Never writes unless options.write === true.
 */
export async function importSkillsFromHarness(
  options: ImportSkillsOptions = {},
): Promise<PathsResult<ImportResult>> {
  const kitHome = options.kitHome ?? getKitHome();
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const scope: PathScope = options.scope ?? "personal";
  const write = options.write === true;
  const force = options.force === true;

  const harnesses = (options.harnesses ?? LINKABLE_HARNESSES).filter(
    (h) => h !== "kit",
  );
  if (harnesses.length === 0) {
    return { ok: false, error: "No harnesses selected for import." };
  }

  const want =
    options.skillNames && options.skillNames.length > 0
      ? new Set(options.skillNames)
      : null;

  const installed = await listSkills({ kitHome });
  if (!installed.ok) {
    return { ok: false, error: installed.error };
  }
  const installedNames = new Set(installed.value.map((s) => s.name));

  const items: ImportPlanItem[] = [];
  const failed: Array<{ skillName: string; error: string }> = [];
  let imported = 0;
  let skipped = 0;

  for (const harness of harnesses) {
    const skillsRoot = resolveHarnessSkillsRoot(harness, scope, {
      projectDir,
      kitHome,
      ...(options.homeDir !== undefined ? { homeDir: options.homeDir } : {}),
    });

    if (!(await pathExists(skillsRoot))) {
      items.push({
        skillName: "(none)",
        sourceDir: skillsRoot,
        harness,
        scope,
        action: "skip-missing-root",
        reason: `Harness skills root does not exist: ${skillsRoot}`,
      });
      skipped += 1;
      continue;
    }

    let entries: string[];
    try {
      const dirents = await readdir(skillsRoot, { withFileTypes: true });
      entries = dirents
        .filter((d) => d.isDirectory() || d.isSymbolicLink())
        .map((d) => d.name)
        .filter((name) => !SKIP_DIR_NAMES.has(name) && !name.startsWith("."))
        .sort();
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      return {
        ok: false,
        error: `Failed to read ${skillsRoot}: ${detail}`,
      };
    }

    for (const folderName of entries) {
      if (want && !want.has(folderName)) continue;

      const sourceDir = path.join(skillsRoot, folderName);
      const skillMd = path.join(sourceDir, "SKILL.md");
      if (!(await pathExists(skillMd))) {
        items.push({
          skillName: folderName,
          sourceDir,
          harness,
          scope,
          action: "skip-invalid",
          reason: "no SKILL.md",
        });
        skipped += 1;
        continue;
      }

      const loaded = await loadSkill(sourceDir);
      if (!loaded.ok) {
        items.push({
          skillName: folderName,
          sourceDir,
          harness,
          scope,
          action: "skip-invalid",
          reason: loaded.issues.map((i) => i.message).join("; "),
        });
        skipped += 1;
        continue;
      }

      const skillName = loaded.skill.name;
      const already = installedNames.has(skillName);

      if (already && !force) {
        items.push({
          skillName,
          sourceDir,
          harness,
          scope,
          action: "skip-exists",
          reason: "already in Kit library (use --force to replace)",
        });
        skipped += 1;
        continue;
      }

      const action = already && force ? "replace" : "import";
      const plan: ImportPlanItem = {
        skillName,
        sourceDir,
        harness,
        scope,
        action,
        ...(action === "replace"
          ? { reason: "will replace existing library install" }
          : {}),
      };
      items.push(plan);

      if (!write) {
        imported += 1;
        continue;
      }

      try {
        const result = await installSkill(sourceDir, { kitHome, force: true });
        if (!result.ok) {
          failed.push({ skillName, error: result.error });
          plan.action = "skip-invalid";
          plan.reason = result.error;
          continue;
        }
        installedNames.add(skillName);
        imported += 1;
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        failed.push({ skillName, error: detail });
      }
    }
  }

  return {
    ok: true,
    value: {
      dryRun: !write,
      items,
      imported,
      skipped,
      failed,
    },
  };
}

async function pathExists(target: string): Promise<boolean> {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}
