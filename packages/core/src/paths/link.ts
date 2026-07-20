import {
  access,
  cp,
  lstat,
  mkdir,
  readlink,
  rm,
  symlink,
} from "node:fs/promises";
import path from "node:path";
import { listSkills } from "../library/library.js";
import { getKitHome, getSkillsDir } from "../library/paths.js";
import {
  LINKABLE_HARNESSES,
  resolveHarnessSkillsRoot,
} from "./harness.js";
import type {
  HarnessId,
  LinkMode,
  LinkPlanItem,
  LinkResult,
  PathScope,
  PathsResult,
} from "./types.js";

export interface LinkSkillsOptions {
  /** Target harnesses. Default: all linkable (not kit). */
  harnesses?: HarnessId[];
  /** personal = user home harness dirs; project = under projectDir. Default: project */
  scope?: PathScope;
  /** Project directory for project scope and source .kit/skills preference. */
  projectDir?: string;
  kitHome?: string;
  homeDir?: string;
  /** Prefer project library, else global. Default true. */
  preferProjectSource?: boolean;
  /** symlink (default) or copy. */
  mode?: LinkMode;
  /**
   * When false (default), only plan actions — no writes.
   * When true, create links/copies.
   */
  write?: boolean;
  /** Replace existing targets that are not already the same link. */
  force?: boolean;
  /** Limit to these skill names. Default: all installed. */
  skillNames?: string[];
}

/**
 * Plan (and optionally write) links from Kit library skills into agent harness folders.
 * Never writes unless options.write === true.
 */
export async function linkSkills(
  options: LinkSkillsOptions = {},
): Promise<PathsResult<LinkResult>> {
  const kitHome = options.kitHome ?? getKitHome();
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const scope: PathScope = options.scope ?? "project";
  const mode: LinkMode = options.mode ?? "symlink";
  const write = options.write === true;
  const force = options.force === true;
  const preferProject = options.preferProjectSource !== false;

  const harnesses = (options.harnesses ?? LINKABLE_HARNESSES).filter(
    (h) => h !== "kit",
  );
  if (harnesses.length === 0) {
    return { ok: false, error: "No linkable harnesses selected." };
  }

  const listed = await listSkills({ kitHome });
  if (!listed.ok) {
    return { ok: false, error: listed.error };
  }

  let skills = listed.value;
  if (options.skillNames && options.skillNames.length > 0) {
    const want = new Set(options.skillNames);
    skills = skills.filter((s) => want.has(s.name));
  }

  if (skills.length === 0) {
    return {
      ok: false,
      error:
        "No skills to link. Install a pack first: kit pack install essentials",
    };
  }

  const projectLib = path.join(projectDir, ".kit", "skills");
  const globalLib = getSkillsDir(kitHome);

  const items: LinkPlanItem[] = [];
  const failed: Array<{ skillName: string; error: string }> = [];
  let linked = 0;
  let skipped = 0;

  for (const harness of harnesses) {
    const skillsRoot = resolveHarnessSkillsRoot(harness, scope, {
      projectDir,
      kitHome,
      ...(options.homeDir !== undefined ? { homeDir: options.homeDir } : {}),
    });

    for (const skill of skills) {
      const sourceDir = await resolveSourceDir(skill.name, {
        installPath: skill.installPath,
        projectLib,
        globalLib,
        preferProject,
      });

      if (!sourceDir) {
        failed.push({
          skillName: skill.name,
          error: `Source folder missing for ${skill.name}`,
        });
        continue;
      }

      const targetDir = path.join(skillsRoot, skill.name);
      const plan = await planOne({
        skillName: skill.name,
        sourceDir,
        targetDir,
        harness,
        scope,
        mode,
        force,
      });
      items.push(plan);

      if (
        plan.action === "skip-same" ||
        plan.action === "skip-exists"
      ) {
        skipped += 1;
        continue;
      }

      if (!write) {
        // dry-run still counts planned creates
        if (plan.action === "create" || plan.action === "replace") {
          linked += 1;
        }
        continue;
      }

      try {
        await mkdir(skillsRoot, { recursive: true });
        if (plan.action === "replace") {
          await rm(targetDir, { recursive: true, force: true });
        }
        if (mode === "symlink") {
          try {
            await symlink(sourceDir, targetDir, "junction");
          } catch {
            // Windows may need dir symlink; fall back to copy
            await cp(sourceDir, targetDir, {
              recursive: true,
              force: true,
              filter: (src) => {
                const base = path.basename(src);
                return base !== ".git" && base !== "node_modules";
              },
            });
            plan.mode = "copy";
            plan.reason = "symlink failed; copied instead";
          }
        } else {
          await cp(sourceDir, targetDir, {
            recursive: true,
            force: true,
            filter: (src) => {
              const base = path.basename(src);
              return base !== ".git" && base !== "node_modules";
            },
          });
        }
        linked += 1;
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        failed.push({ skillName: skill.name, error: detail });
      }
    }
  }

  return {
    ok: true,
    value: {
      dryRun: !write,
      items,
      linked,
      skipped,
      failed,
    },
  };
}

async function resolveSourceDir(
  skillName: string,
  options: {
    installPath: string;
    projectLib: string;
    globalLib: string;
    preferProject: boolean;
  },
): Promise<string | undefined> {
  const candidates = options.preferProject
    ? [
        path.join(options.projectLib, skillName),
        options.installPath,
        path.join(options.globalLib, skillName),
      ]
    : [
        options.installPath,
        path.join(options.globalLib, skillName),
        path.join(options.projectLib, skillName),
      ];

  for (const candidate of candidates) {
    if (await pathExists(candidate)) {
      return path.resolve(candidate);
    }
  }
  return undefined;
}

async function planOne(input: {
  skillName: string;
  sourceDir: string;
  targetDir: string;
  harness: HarnessId;
  scope: PathScope;
  mode: LinkMode;
  force: boolean;
}): Promise<LinkPlanItem> {
  const base: LinkPlanItem = {
    skillName: input.skillName,
    sourceDir: input.sourceDir,
    targetDir: input.targetDir,
    harness: input.harness,
    scope: input.scope,
    mode: input.mode,
    action: "create",
  };

  if (!(await pathExists(input.targetDir))) {
    return base;
  }

  try {
    const stat = await lstat(input.targetDir);
    if (stat.isSymbolicLink()) {
      const current = await readlink(input.targetDir);
      const resolved = path.resolve(path.dirname(input.targetDir), current);
      if (path.resolve(resolved) === path.resolve(input.sourceDir)) {
        return {
          ...base,
          action: "skip-same",
          reason: "already linked to Kit source",
        };
      }
    }
  } catch {
    // treat as exists
  }

  if (input.force) {
    return {
      ...base,
      action: "replace",
      reason: "existing target will be replaced",
    };
  }

  return {
    ...base,
    action: "skip-exists",
    reason: "target exists (use --force to replace)",
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
