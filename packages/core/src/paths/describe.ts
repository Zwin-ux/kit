import { access } from "node:fs/promises";
import path from "node:path";
import { listSkills } from "../library/library.js";
import { getKitHome, getSkillsDir } from "../library/paths.js";
import {
  ALL_HARNESSES,
  harnessNotes,
  resolveHarnessSkillsRoot,
} from "./harness.js";
import type {
  HarnessId,
  HarnessSkillPath,
  PathReport,
  PathScope,
  PathsResult,
} from "./types.js";

export interface DescribePathsOptions {
  kitHome?: string;
  projectDir?: string;
  skillName?: string;
  homeDir?: string;
  harnesses?: HarnessId[];
}

/**
 * Describe where Kit and major agent harnesses look for skills.
 * Read-only. Does not create directories.
 */
export async function describePaths(
  options: DescribePathsOptions = {},
): Promise<PathsResult<PathReport>> {
  const kitHome = options.kitHome ?? getKitHome();
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const skillName = options.skillName?.trim() || undefined;
  const harnesses = options.harnesses ?? ALL_HARNESSES;

  const listed = await listSkills({ kitHome });
  if (!listed.ok) {
    return { ok: false, error: listed.error };
  }

  const scopes: PathScope[] = ["personal", "project"];
  const entries: HarnessSkillPath[] = [];

  for (const harness of harnesses) {
    for (const scope of scopes) {
      const skillsRoot = resolveHarnessSkillsRoot(harness, scope, {
        projectDir,
        kitHome,
        ...(options.homeDir !== undefined ? { homeDir: options.homeDir } : {}),
      });
      const entry: HarnessSkillPath = {
        harness,
        scope,
        skillsRoot,
        notes: harnessNotes(harness, scope),
        exists: await pathExists(skillsRoot),
      };
      if (skillName) {
        entry.skillDir = path.join(skillsRoot, skillName);
      }
      entries.push(entry);
    }
  }

  return {
    ok: true,
    value: {
      kitHome,
      projectDir,
      globalLibrary: getSkillsDir(kitHome),
      projectLibrary: path.join(projectDir, ".kit", "skills"),
      ...(skillName !== undefined ? { skillName } : {}),
      entries,
      installedSkillNames: listed.value.map((s) => s.name).sort(),
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
