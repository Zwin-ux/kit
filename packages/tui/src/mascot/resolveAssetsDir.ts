import { access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Find assets/pixel for mascot frames.
 * Order: KIT_ASSETS env, walk from cwd, walk from this package toward monorepo root.
 */
export async function resolvePixelAssetsDir(): Promise<string | undefined> {
  const fromEnv = process.env.KIT_ASSETS?.trim();
  if (fromEnv) {
    const candidate = path.resolve(fromEnv);
    if (await exists(candidate)) return candidate;
  }

  const starts = [
    process.cwd(),
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../.."),
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../.."),
  ];

  for (const start of starts) {
    const found = await walkForPixelDir(start, 6);
    if (found) return found;
  }

  return undefined;
}

async function walkForPixelDir(
  start: string,
  maxUp: number,
): Promise<string | undefined> {
  let current = path.resolve(start);
  for (let i = 0; i <= maxUp; i++) {
    const candidate = path.join(current, "assets", "pixel");
    if (await exists(candidate)) {
      return candidate;
    }
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
