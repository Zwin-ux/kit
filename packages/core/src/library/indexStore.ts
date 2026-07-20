import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import {
  getKitHome,
  getLibraryIndexPath,
  getSkillsDir,
} from "./paths.js";
import type { LibraryIndex, LibraryIndexEntry } from "./types.js";

const EMPTY_INDEX: LibraryIndex = {
  version: 1,
  skills: {},
};

export async function ensureLibraryDirs(
  kitHome: string = getKitHome(),
): Promise<void> {
  await mkdir(getSkillsDir(kitHome), { recursive: true });
}

export async function readLibraryIndex(
  kitHome: string = getKitHome(),
): Promise<LibraryIndex> {
  const indexPath = getLibraryIndexPath(kitHome);
  try {
    const raw = await readFile(indexPath, "utf8");
    const parsed = JSON.parse(raw) as unknown;
    return normalizeIndex(parsed);
  } catch (error) {
    const code =
      error && typeof error === "object" && "code" in error
        ? String((error as { code: unknown }).code)
        : undefined;
    if (code === "ENOENT") {
      return { ...EMPTY_INDEX, skills: {} };
    }
    throw error;
  }
}

export async function writeLibraryIndex(
  index: LibraryIndex,
  kitHome: string = getKitHome(),
): Promise<void> {
  await ensureLibraryDirs(kitHome);
  const indexPath = getLibraryIndexPath(kitHome);
  const payload = `${JSON.stringify(index, null, 2)}\n`;
  await writeFile(indexPath, payload, "utf8");
}

export async function upsertIndexEntry(
  entry: LibraryIndexEntry,
  kitHome: string = getKitHome(),
): Promise<LibraryIndex> {
  const index = await readLibraryIndex(kitHome);
  index.skills[entry.name] = entry;
  await writeLibraryIndex(index, kitHome);
  return index;
}

export async function removeIndexEntry(
  name: string,
  kitHome: string = getKitHome(),
): Promise<boolean> {
  const index = await readLibraryIndex(kitHome);
  if (!(name in index.skills)) {
    return false;
  }
  delete index.skills[name];
  await writeLibraryIndex(index, kitHome);
  return true;
}

export function skillInstallPath(
  name: string,
  kitHome: string = getKitHome(),
): string {
  return path.join(getSkillsDir(kitHome), name);
}

function normalizeIndex(parsed: unknown): LibraryIndex {
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    return { ...EMPTY_INDEX, skills: {} };
  }

  const record = parsed as { version?: unknown; skills?: unknown };
  const skills: Record<string, LibraryIndexEntry> = {};

  if (record.skills && typeof record.skills === "object" && !Array.isArray(record.skills)) {
    for (const [key, value] of Object.entries(
      record.skills as Record<string, unknown>,
    )) {
      const entry = normalizeEntry(key, value);
      if (entry) {
        skills[entry.name] = entry;
      }
    }
  }

  return { version: 1, skills };
}

function normalizeEntry(
  key: string,
  value: unknown,
): LibraryIndexEntry | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return undefined;
  }

  const row = value as Record<string, unknown>;
  const name = typeof row.name === "string" ? row.name : key;
  const version = typeof row.version === "string" ? row.version : undefined;
  const description =
    typeof row.description === "string" ? row.description : undefined;
  const installedAt =
    typeof row.installedAt === "string" ? row.installedAt : undefined;
  const compatibility = Array.isArray(row.compatibility)
    ? row.compatibility.filter((item): item is string => typeof item === "string")
    : [];

  if (!version || !description || !installedAt) {
    return undefined;
  }

  const entry: LibraryIndexEntry = {
    name,
    version,
    description,
    compatibility,
    installedAt,
  };

  if (typeof row.sourcePath === "string") {
    entry.sourcePath = row.sourcePath;
  }

  return entry;
}
