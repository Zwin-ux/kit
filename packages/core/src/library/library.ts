import { cp, mkdir, rename, rm, access } from "node:fs/promises";
import path from "node:path";
import { loadSkill, formatIssues } from "../loadSkill.js";
import {
  ensureLibraryDirs,
  readLibraryIndex,
  removeIndexEntry,
  skillInstallPath,
  upsertIndexEntry,
} from "./indexStore.js";
import { getKitHome, getSkillsDir } from "./paths.js";
import type {
  InstalledSkill,
  LibraryIndexEntry,
  LibraryResult,
} from "./types.js";

export interface InstallOptions {
  /** Kit home directory. Defaults to KIT_HOME or ~/.kit. */
  kitHome?: string;
  /**
   * Replace an existing install with the same name.
   * Default: false (fail if already installed).
   */
  force?: boolean;
}

export interface ListOptions {
  kitHome?: string;
}

export interface RemoveOptions {
  kitHome?: string;
}

/**
 * Install a skill from a local folder into the offline library.
 * Validates with schema v0 before copying.
 */
export async function installSkill(
  sourceDir: string,
  options: InstallOptions = {},
): Promise<LibraryResult<InstalledSkill>> {
  const kitHome = options.kitHome ?? getKitHome();
  const resolvedSource = path.resolve(sourceDir);

  const loaded = await loadSkill(resolvedSource);
  if (!loaded.ok) {
    return {
      ok: false,
      error: `Skill failed validation:\n${formatIssues(loaded.issues)}`,
    };
  }

  const skill = loaded.skill;
  const target = skillInstallPath(skill.name, kitHome);

  await ensureLibraryDirs(kitHome);

  if (await pathExists(target) && !options.force) {
    return {
      ok: false,
      error: `Skill "${skill.name}" is already installed at ${target}. Use force to replace it.`,
    };
  }

  // Stage → validate → rename. Never leave a half-deleted live skill.
  const skillsRoot = getSkillsDir(kitHome);
  const staging = path.join(
    skillsRoot,
    `.tmp-${skill.name}-${Date.now().toString(36)}`,
  );

  try {
    await mkdir(staging, { recursive: true });
    await cp(resolvedSource, staging, {
      recursive: true,
      force: true,
      errorOnExist: false,
      filter: (src) => {
        const base = path.basename(src);
        return base !== ".git" && base !== "node_modules";
      },
    });

    const staged = await loadSkill(staging);
    if (!staged.ok) {
      await rm(staging, { recursive: true, force: true });
      return {
        ok: false,
        error: `Staged skill failed validation:\n${formatIssues(staged.issues)}`,
      };
    }

    if (await pathExists(target)) {
      await rm(target, { recursive: true, force: true });
    }
    await rename(staging, target);
  } catch (error) {
    await rm(staging, { recursive: true, force: true }).catch(() => {});
    const detail = error instanceof Error ? error.message : String(error);
    return {
      ok: false,
      error: `Failed to install skill into library: ${detail}`,
    };
  }

  const installedAt = new Date().toISOString();
  const entry: LibraryIndexEntry = {
    name: skill.name,
    version: skill.version,
    description: skill.description,
    compatibility: skill.compatibility,
    installedAt,
    sourcePath: resolvedSource,
  };

  await upsertIndexEntry(entry, kitHome);

  return {
    ok: true,
    value: toInstalledSkill(entry, target),
  };
}

/**
 * List skills installed in the local library.
 */
export async function listSkills(
  options: ListOptions = {},
): Promise<LibraryResult<InstalledSkill[]>> {
  const kitHome = options.kitHome ?? getKitHome();
  await ensureLibraryDirs(kitHome);

  try {
    const index = await readLibraryIndex(kitHome);
    const skills: InstalledSkill[] = [];

    for (const entry of Object.values(index.skills)) {
      const installPath = skillInstallPath(entry.name, kitHome);
      if (!(await pathExists(installPath))) {
        // Index is stale; skip missing folders.
        continue;
      }
      skills.push(toInstalledSkill(entry, installPath));
    }

    skills.sort((a, b) => a.name.localeCompare(b.name));
    return { ok: true, value: skills };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Failed to list skills: ${detail}` };
  }
}

/**
 * Remove an installed skill by name.
 */
export async function removeSkill(
  name: string,
  options: RemoveOptions = {},
): Promise<LibraryResult<{ name: string }>> {
  const kitHome = options.kitHome ?? getKitHome();
  const trimmed = name.trim();

  if (!trimmed) {
    return { ok: false, error: "Skill name is required." };
  }

  await ensureLibraryDirs(kitHome);

  const installPath = skillInstallPath(trimmed, kitHome);
  const inIndex = await removeIndexEntry(trimmed, kitHome);
  const onDisk = await pathExists(installPath);

  if (!inIndex && !onDisk) {
    return {
      ok: false,
      error: `Skill "${trimmed}" is not installed.`,
    };
  }

  if (onDisk) {
    try {
      await rm(installPath, { recursive: true, force: true });
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      return {
        ok: false,
        error: `Failed to remove skill folder: ${detail}`,
      };
    }
  }

  return { ok: true, value: { name: trimmed } };
}

function toInstalledSkill(
  entry: LibraryIndexEntry,
  installPath: string,
): InstalledSkill {
  const skill: InstalledSkill = {
    name: entry.name,
    version: entry.version,
    description: entry.description,
    compatibility: entry.compatibility,
    installPath,
    installedAt: entry.installedAt,
  };

  if (entry.sourcePath !== undefined) {
    skill.sourcePath = entry.sourcePath;
  }

  return skill;
}

async function pathExists(target: string): Promise<boolean> {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}
