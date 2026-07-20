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
  SkillPack,
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
 * Resolves `extends` so dependency packs contribute their skills first.
 */
export async function loadPack(
  packDirOrName: string,
  options: PackLoadOptions = {},
): Promise<PackResult<LoadedPack>> {
  const expanded = await loadPackExpanded(packDirOrName, options, []);
  if (!expanded.ok) return expanded;

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
  const packDir = expanded.value.rootDir;

  for (const skillName of expanded.value.skillNames) {
    const candidates = [
      {
        dir: path.join(packDir, "skills", skillName),
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
      // Also try pack-local folders of extended packs (dependency skills live there).
      const fromExtends = await tryResolveFromExtendedPacks(
        skillName,
        expanded.value.extends,
        options,
      );
      if (fromExtends) {
        resolved.push(fromExtends);
        found = true;
      }
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
      error: `Pack "${expanded.value.name}" has skill problems:\n${skillErrors.join("\n")}`,
    };
  }

  return {
    ok: true,
    value: {
      pack: expanded.value,
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
      continue;
    }
    const { pack } = loaded.value;
    items.push({
      name: pack.name,
      title: pack.title,
      description: pack.description,
      version: pack.version,
      skillCount: pack.skillNames.length,
      extends: pack.extends,
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

/** Parse + validate + merge extends into skillNames (no skill file resolve). */
async function loadPackExpanded(
  packDirOrName: string,
  options: PackLoadOptions,
  stack: string[],
): Promise<PackResult<SkillPack>> {
  const meta = await loadPackMeta(packDirOrName, options);
  if (!meta.ok) return meta;

  const pack = meta.value;
  if (stack.includes(pack.name)) {
    return {
      ok: false,
      error: `Pack extend cycle: ${[...stack, pack.name].join(" → ")}`,
    };
  }

  const nextStack = [...stack, pack.name];
  const merged: string[] = [];
  const seen = new Set<string>();

  const pushAll = (names: string[]) => {
    for (const n of names) {
      if (seen.has(n)) continue;
      seen.add(n);
      merged.push(n);
    }
  };

  for (const base of pack.extends) {
    const parent = await loadPackExpanded(base, options, nextStack);
    if (!parent.ok) {
      return {
        ok: false,
        error: `Pack "${pack.name}" extends "${base}" failed: ${parent.error}`,
        ...(parent.issues ? { issues: parent.issues } : {}),
      };
    }
    pushAll(parent.value.skillNames);
  }

  pushAll(pack.ownSkillNames);

  if (merged.length === 0) {
    return {
      ok: false,
      error: `Pack "${pack.name}" has no skills after resolving extends.`,
    };
  }

  return {
    ok: true,
    value: {
      ...pack,
      skillNames: merged,
    },
  };
}

async function loadPackMeta(
  packDirOrName: string,
  options: PackLoadOptions,
): Promise<PackResult<SkillPack>> {
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

  return { ok: true, value: validated.pack };
}

async function tryResolveFromExtendedPacks(
  skillName: string,
  extendsList: string[],
  options: PackLoadOptions,
): Promise<ResolvedPackSkill | null> {
  for (const base of extendsList) {
    const dir = await resolvePackDir(base, options);
    if (!dir.ok) continue;
    const packLocal = path.join(dir.value, "skills", skillName);
    const loaded = await loadSkill(packLocal);
    if (loaded.ok && loaded.skill.name === skillName) {
      return {
        name: skillName,
        sourceDir: packLocal,
        skill: loaded.skill,
        origin: "pack-local",
      };
    }
  }
  return null;
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
