import { access, mkdir, readFile, writeFile, cp } from "node:fs/promises";
import path from "node:path";
import { installSkill } from "../library/library.js";
import { getKitHome } from "../library/paths.js";
import type { InstalledSkill } from "../library/types.js";
import { loadPack, type PackLoadOptions } from "./loadPack.js";
import type {
  ApplyPackResult,
  InstallPackResult,
  PackResult,
} from "./types.js";

export interface InstallPackOptions extends PackLoadOptions {
  kitHome?: string;
  /**
   * Replace skills that already exist in the library.
   * Default true for packs so a pack install is complete.
   */
  force?: boolean;
}

export interface ApplyPackOptions extends InstallPackOptions {
  /** Project directory to receive the pack. Default: cwd. */
  projectDir?: string;
}

/** Soft warning when applying outside a git work tree. */
export async function detectMissingGitRoot(
  projectDir: string,
): Promise<string | undefined> {
  let current = path.resolve(projectDir);
  for (let i = 0; i < 12; i++) {
    try {
      await access(path.join(current, ".git"));
      return undefined;
    } catch {
      // keep walking
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return `No .git found near ${projectDir}. Apply still works; commit .kit/ if you want the pack in version control.`;
}

/**
 * Install every skill in a pack into the local offline library.
 */
export async function installPack(
  packDirOrName: string,
  options: InstallPackOptions = {},
): Promise<PackResult<InstallPackResult>> {
  const loaded = await loadPack(packDirOrName, options);
  if (!loaded.ok) return loaded;

  const kitHome = options.kitHome ?? getKitHome();
  const force = options.force ?? true;
  const installed: InstalledSkill[] = [];
  const skipped: string[] = [];
  const errors: string[] = [];

  for (const entry of loaded.value.skills) {
    const result = await installSkill(entry.sourceDir, { kitHome, force });
    if (!result.ok) {
      if (result.error.includes("already installed") && !force) {
        skipped.push(entry.name);
        continue;
      }
      errors.push(`${entry.name}: ${result.error}`);
      continue;
    }
    installed.push(result.value);
  }

  if (errors.length > 0) {
    return {
      ok: false,
      error: `Pack install incomplete:\n${errors.join("\n")}`,
    };
  }

  await recordInstalledPack(loaded.value.pack.name, loaded.value.pack.version, {
    kitHome,
    skillNames: loaded.value.skills.map((s) => s.name),
  });

  return {
    ok: true,
    value: {
      pack: loaded.value.pack,
      installed,
      skipped,
    },
  };
}

/**
 * Install a pack into the library and copy skills into the project.
 * Writes `.kit/applied-packs.json` and `.kit/skills/<name>/`.
 */
export async function applyPack(
  packDirOrName: string,
  options: ApplyPackOptions = {},
): Promise<PackResult<ApplyPackResult>> {
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const loaded = await loadPack(packDirOrName, options);
  if (!loaded.ok) return loaded;

  const gitWarning = await detectMissingGitRoot(projectDir);

  const installed = await installPack(packDirOrName, {
    ...options,
    force: options.force ?? true,
  });
  if (!installed.ok) return installed;

  const kitDir = path.join(projectDir, ".kit");
  const projectSkillsDir = path.join(kitDir, "skills");
  await mkdir(projectSkillsDir, { recursive: true });

  for (const entry of loaded.value.skills) {
    const target = path.join(projectSkillsDir, entry.name);
    await cp(entry.sourceDir, target, {
      recursive: true,
      force: true,
      filter: (src) => {
        const base = path.basename(src);
        return base !== ".git" && base !== "node_modules";
      },
    });
  }

  const appliedPath = path.join(kitDir, "applied-packs.json");
  const applied = await readAppliedPacksFile(appliedPath);
  const previous = applied.packs[loaded.value.pack.name];
  const nextVersion = loaded.value.pack.version;

  applied.packs[loaded.value.pack.name] = {
    name: loaded.value.pack.name,
    version: loaded.value.pack.version,
    title: loaded.value.pack.title,
    skills: loaded.value.skills.map((s) => s.name),
    appliedAt: new Date().toISOString(),
  };
  await writeFile(
    appliedPath,
    `${JSON.stringify({ version: 1, packs: applied.packs }, null, 2)}\n`,
    "utf8",
  );

  const result: ApplyPackResult = {
    pack: loaded.value.pack,
    projectDir,
    installed: installed.value.installed,
    projectSkillsDir,
    appliedPath,
    reapplied: previous !== undefined,
    versionChanged: previous !== undefined && previous.version !== nextVersion,
  };

  if (gitWarning !== undefined) {
    result.gitWarning = gitWarning;
  }

  return {
    ok: true,
    value: result,
  };
}

export interface AppliedPackRecord {
  name: string;
  version: string;
  title: string;
  skills: string[];
  appliedAt: string;
}

export interface AppliedPacksFile {
  version: 1;
  packs: Record<string, AppliedPackRecord>;
}

/**
 * Read project applied packs from `.kit/applied-packs.json`.
 */
export async function readProjectAppliedPacks(
  projectDir: string = process.cwd(),
): Promise<AppliedPacksFile> {
  const filePath = path.join(
    path.resolve(projectDir),
    ".kit",
    "applied-packs.json",
  );
  return readAppliedPacksFile(filePath);
}

async function readAppliedPacksFile(filePath: string): Promise<AppliedPacksFile> {
  try {
    const raw = await readFile(filePath, "utf8");
    const parsed = JSON.parse(raw) as AppliedPacksFile;
    if (!parsed.packs || typeof parsed.packs !== "object") {
      return { version: 1, packs: {} };
    }
    return { version: 1, packs: parsed.packs };
  } catch {
    return { version: 1, packs: {} };
  }
}

async function recordInstalledPack(
  name: string,
  version: string,
  options: { kitHome: string; skillNames: string[] },
): Promise<void> {
  const filePath = path.join(options.kitHome, "installed-packs.json");
  await mkdir(options.kitHome, { recursive: true });

  let data: {
    version: 1;
    packs: Record<
      string,
      { name: string; version: string; skills: string[]; installedAt: string }
    >;
  } = { version: 1, packs: {} };

  try {
    const raw = await readFile(filePath, "utf8");
    const parsed = JSON.parse(raw) as typeof data;
    if (parsed.packs && typeof parsed.packs === "object") {
      data = { version: 1, packs: parsed.packs };
    }
  } catch {
    // fresh file
  }

  data.packs[name] = {
    name,
    version,
    skills: options.skillNames,
    installedAt: new Date().toISOString(),
  };

  await writeFile(filePath, `${JSON.stringify(data, null, 2)}\n`, "utf8");
}
