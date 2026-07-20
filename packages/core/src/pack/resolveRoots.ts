import { access } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);

/**
 * Find the packs/ directory.
 * Order: KIT_PACKS env → @mzwin/kit-catalog package → monorepo walk.
 */
export async function resolvePacksRoot(
  options?: { startDir?: string },
): Promise<string | undefined> {
  const fromEnv = process.env.KIT_PACKS?.trim();
  if (fromEnv) {
    const candidate = path.resolve(fromEnv);
    if (await exists(candidate)) return candidate;
  }

  const fromCatalog = resolveCatalogSubdir("packs");
  if (fromCatalog && (await exists(fromCatalog))) return fromCatalog;

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
 * Find the skills/ catalog directory.
 * Order: KIT_SKILLS env → sibling of packs → @mzwin/kit-catalog → monorepo walk.
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

  const fromCatalog = resolveCatalogSubdir("skills");
  if (fromCatalog && (await exists(fromCatalog))) return fromCatalog;

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

/** Resolve packs/skills shipped inside @mzwin/kit-catalog (published npm layout). */
function resolveCatalogSubdir(sub: "packs" | "skills"): string | undefined {
  try {
    const pkgJson = require.resolve("@mzwin/kit-catalog/package.json");
    return path.join(path.dirname(pkgJson), sub);
  } catch {
    // Not installed / not linked — monorepo walk will handle dev.
    return undefined;
  }
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
