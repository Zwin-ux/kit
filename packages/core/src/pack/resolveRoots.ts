import { access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Find the monorepo packs/ directory (or KIT_PACKS override).
 */
export async function resolvePacksRoot(
  options?: { startDir?: string },
): Promise<string | undefined> {
  const fromEnv = process.env.KIT_PACKS?.trim();
  if (fromEnv) {
    const candidate = path.resolve(fromEnv);
    if (await exists(candidate)) return candidate;
  }

  const starts = [
    options?.startDir,
    process.cwd(),
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../.."),
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../../.."),
  ].filter((value): value is string => Boolean(value));

  for (const start of starts) {
    const found = await walkForDir(start, "packs", 8);
    if (found) return found;
  }
  return undefined;
}

/**
 * Find the monorepo skills/ catalog directory (or KIT_SKILLS override).
 */
export async function resolveSkillsCatalogRoot(
  options?: { startDir?: string; packsRoot?: string },
): Promise<string | undefined> {
  const fromEnv = process.env.KIT_SKILLS?.trim();
  if (fromEnv) {
    const candidate = path.resolve(fromEnv);
    if (await exists(candidate)) return candidate;
  }

  if (options?.packsRoot) {
    const sibling = path.join(path.dirname(options.packsRoot), "skills");
    if (await exists(sibling)) return sibling;
  }

  const starts = [
    options?.startDir,
    process.cwd(),
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../.."),
  ].filter((value): value is string => Boolean(value));

  for (const start of starts) {
    const found = await walkForDir(start, "skills", 8);
    if (found) return found;
  }
  return undefined;
}

async function walkForDir(
  start: string,
  dirName: string,
  maxUp: number,
): Promise<string | undefined> {
  let current = path.resolve(start);
  for (let i = 0; i <= maxUp; i++) {
    const candidate = path.join(current, dirName);
    if (await exists(candidate)) return candidate;
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return undefined;
}

async function exists(target: string): Promise<boolean> {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}
