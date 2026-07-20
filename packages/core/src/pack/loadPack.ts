import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { formatIssues, loadSkill } from "../loadSkill.js";
import { parsePackMd } from "./parsePackMd.js";
import {
  resolvePacksRoot,
  resolveSkillsCatalogRoot,
} from "./resolveRoots.js";
import type {
  LoadedPack,
  PackListItem,
  PackResult,
  ResolvedPackSkill,
} from "./types.js";
import { validatePackFrontMatter } from "./validatePack.js";

export interface PackLoadOptions {
  /** Directory that contains pack folders. */
  packsRoot?: string;
  /** Shared skill catalog root (skills/). */
  skillsRoot?: string;
  /** Start directory for auto-discovery. */
  startDir?: string;
}

/**
 * Load a pack by folder path or by pack name under packs/.
 */
export async function loadPack(
  packDirOrName: string,
  options: PackLoadOptions = {},
): Promise<PackResult<LoadedPack>> {
  const packDir = await resolvePackDir(packDirOrName, options);
  if (!packDir.ok) return packDir;

  const packFile = path.join(packDir.value, "PACK.md");
  let content: string;
  try {
    content = await readFile(packFile, "utf8");
  } catch {
    return {
      ok: false,
      error: `PACK.md is missing in ${packDir.value}`,
    };
  }

  const parsed = parsePackMd(content);
  if (!parsed.ok) {
    return {
      ok: false,
      error: formatIssues(parsed.issues),
      issues: parsed.issues,
    };
  }

  const validated = validatePackFrontMatter(
    parsed.value.frontMatter,
    parsed.value.body,
    packDir.value,
  );
  if (!validated.ok) {
    return {
      ok: false,
      error: formatIssues(validated.issues),
      issues: validated.issues,
    };
  }

  const packsRootForSkills =
    options.packsRoot ?? (await resolvePacksRoot(options));
  const skillsRoot =
    options.skillsRoot ??
    (await resolveSkillsCatalogRoot({
      ...(options.startDir !== undefined
        ? { startDir: options.startDir }
        : {}),
      ...(packsRootForSkills !== undefined
        ? { packsRoot: packsRootForSkills }
        : {}),
    }));

  const resolved: ResolvedPackSkill[] = [];
  const skillErrors: string[] = [];

  for (const skillName of validated.pack.skillNames) {
    const candidates = [
      {
        dir: path.join(packDir.value, "skills", skillName),
        origin: "pack-local" as const,
      },
      ...(skillsRoot
        ? [
            {
              dir: path.join(skillsRoot, skillName),
              origin: "catalog" as const,
            },
          ]
        : []),
    ];

    let found = false;
    for (const candidate of candidates) {
      const loaded = await loadSkill(candidate.dir);
      if (!loaded.ok) {
        // Missing folder is normal when trying the next origin.
        if (loaded.issues[0]?.message.includes("SKILL.md is missing")) {
          continue;
        }
        skillErrors.push(
          `${skillName} (${candidate.origin}): ${formatIssues(loaded.issues)}`,
        );
        found = true;
        break;
      }
      if (loaded.skill.name !== skillName) {
        skillErrors.push(
          `${skillName}: folder skill name is "${loaded.skill.name}" but pack lists "${skillName}".`,
        );
        found = true;
        break;
      }
      resolved.push({
        name: skillName,
        sourceDir: candidate.dir,
        skill: loaded.skill,
        origin: candidate.origin,
      });
      found = true;
      break;
    }

    if (!found) {
      skillErrors.push(
        `${skillName}: not found in pack-local skills/ or catalog skills/.`,
      );
    }
  }

  if (skillErrors.length > 0) {
    return {
      ok: false,
      error: `Pack "${validated.pack.name}" has skill problems:\n${skillErrors.join("\n")}`,
    };
  }

  return {
    ok: true,
    value: {
      pack: validated.pack,
      skills: resolved,
    },
  };
}

/**
 * List packs under the packs root.
 */
export async function listPacks(
  options: PackLoadOptions = {},
): Promise<PackResult<PackListItem[]>> {
  const packsRoot =
    options.packsRoot ?? (await resolvePacksRoot(options));
  if (!packsRoot) {
    return {
      ok: false,
      error:
        "Could not find packs/. Set KIT_PACKS or run from the Kit repository.",
    };
  }

  let entries: string[];
  try {
    const dirents = await readdir(packsRoot, { withFileTypes: true });
    entries = dirents.filter((d) => d.isDirectory()).map((d) => d.name);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Cannot read packs directory: ${detail}` };
  }

  const items: PackListItem[] = [];
  for (const name of entries.sort()) {
    const loaded = await loadPack(path.join(packsRoot, name), {
      ...options,
      packsRoot,
    });
    if (!loaded.ok) {
      // Skip invalid folders (keep list useful).
      continue;
    }
    const { pack } = loaded.value;
    items.push({
      name: pack.name,
      title: pack.title,
      description: pack.description,
      version: pack.version,
      skillCount: pack.skillNames.length,
      tags: pack.tags,
      projectTypes: pack.projectTypes,
      rootDir: pack.rootDir,
    });
  }

  return { ok: true, value: items };
}

/**
 * Validate a pack and all of its skills without installing.
 */
export async function validatePack(
  packDirOrName: string,
  options: PackLoadOptions = {},
): Promise<PackResult<LoadedPack>> {
  return loadPack(packDirOrName, options);
}

async function resolvePackDir(
  packDirOrName: string,
  options: PackLoadOptions,
): Promise<PackResult<string>> {
  const asPath = path.resolve(packDirOrName);
  try {
    await readFile(path.join(asPath, "PACK.md"));
    return { ok: true, value: asPath };
  } catch {
    // treat as name
  }

  const packsRoot =
    options.packsRoot ?? (await resolvePacksRoot(options));
  if (!packsRoot) {
    return {
      ok: false,
      error: `Pack not found: ${packDirOrName}. Set KIT_PACKS or pass a pack directory.`,
    };
  }

  const byName = path.join(packsRoot, packDirOrName);
  try {
    await readFile(path.join(byName, "PACK.md"));
    return { ok: true, value: byName };
  } catch {
    return {
      ok: false,
      error: `Pack "${packDirOrName}" not found under ${packsRoot}`,
    };
  }
}
