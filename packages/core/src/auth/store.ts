import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { getAuthPath, getKitHome } from "../library/paths.js";
import type { AuthResult, KitAuthSession } from "./types.js";

export { getAuthPath };

export async function readAuthSession(
  kitHome: string = getKitHome(),
): Promise<KitAuthSession | null> {
  try {
    const raw = await readFile(getAuthPath(kitHome), "utf8");
    const parsed = JSON.parse(raw) as KitAuthSession;
    if (
      parsed.version !== 1 ||
      typeof parsed.accessToken !== "string" ||
      !parsed.user?.login
    ) {
      return null;
    }
    return parsed;
  } catch {
    return null;
  }
}

export async function writeAuthSession(
  session: KitAuthSession,
  kitHome: string = getKitHome(),
): Promise<void> {
  await mkdir(kitHome, { recursive: true });
  await writeFile(
    getAuthPath(kitHome),
    `${JSON.stringify(session, null, 2)}\n`,
    "utf8",
  );
}

export async function clearAuthSession(
  kitHome: string = getKitHome(),
): Promise<AuthResult<{ cleared: boolean }>> {
  try {
    await rm(getAuthPath(kitHome), { force: true });
    return { ok: true, value: { cleared: true } };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { ok: false, error: `Failed to clear auth: ${detail}` };
  }
}
