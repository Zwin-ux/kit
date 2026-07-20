import os from "node:os";
import path from "node:path";

/**
 * Root directory for Kit local state.
 * Override with KIT_HOME for tests or custom installs.
 */
export function getKitHome(): string {
  const fromEnv = process.env.KIT_HOME?.trim();
  if (fromEnv) {
    return path.resolve(fromEnv);
  }
  return path.join(os.homedir(), ".kit");
}

/** Directory that holds installed skill folders. */
export function getSkillsDir(kitHome: string = getKitHome()): string {
  return path.join(kitHome, "skills");
}

/** Index file for installed skill metadata. */
export function getLibraryIndexPath(kitHome: string = getKitHome()): string {
  return path.join(kitHome, "library.json");
}

/** User config file. */
export function getConfigPath(kitHome: string = getKitHome()): string {
  return path.join(kitHome, "config.json");
}

/** GitHub auth session file. */
export function getAuthPath(kitHome: string = getKitHome()): string {
  return path.join(kitHome, "auth.json");
}
