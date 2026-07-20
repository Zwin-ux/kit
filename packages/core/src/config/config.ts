import { mkdir, readFile, writeFile } from "node:fs/promises";
import { listSkills } from "../library/library.js";
import { getConfigPath, getKitHome } from "../library/paths.js";
import {
  DEFAULT_KIT_CONFIG,
  FIRST_RUN_PACK_OPTIONS,
  type FirstRunPackName,
  type KitConfig,
} from "./types.js";

export { getConfigPath };

/**
 * Read Kit config. Missing file returns defaults (not an error).
 */
export async function readConfig(
  kitHome: string = getKitHome(),
): Promise<KitConfig> {
  const configPath = getConfigPath(kitHome);
  try {
    const raw = await readFile(configPath, "utf8");
    return normalizeConfig(JSON.parse(raw) as unknown);
  } catch (error) {
    const code =
      error && typeof error === "object" && "code" in error
        ? String((error as { code: unknown }).code)
        : undefined;
    if (code === "ENOENT") {
      return { ...DEFAULT_KIT_CONFIG };
    }
    throw error;
  }
}

/**
 * Write Kit config (creates ~/.kit if needed).
 */
export async function writeConfig(
  config: KitConfig,
  kitHome: string = getKitHome(),
): Promise<void> {
  await mkdir(kitHome, { recursive: true });
  const normalized = normalizeConfig(config);
  await writeFile(
    getConfigPath(kitHome),
    `${JSON.stringify(normalized, null, 2)}\n`,
    "utf8",
  );
}

/**
 * Merge partial updates into existing config and save.
 */
export async function updateConfig(
  patch: Partial<KitConfig>,
  kitHome: string = getKitHome(),
): Promise<KitConfig> {
  const current = await readConfig(kitHome);
  const next = normalizeConfig({ ...current, ...patch, version: 1 });
  await writeConfig(next, kitHome);
  return next;
}

export interface FirstRunStatus {
  /** True when the UI/CLI should offer the first-run flow. */
  shouldOffer: boolean;
  reason: "not-completed" | "completed" | "library-already-used";
  config: KitConfig;
  skillCount: number;
}

/**
 * First-run offers when the user has not completed it and the library is empty.
 * If the library already has skills, we do not nag (mark completed quietly optional).
 */
export async function getFirstRunStatus(
  kitHome: string = getKitHome(),
): Promise<FirstRunStatus> {
  const config = await readConfig(kitHome);
  const listed = await listSkills({ kitHome });
  const skillCount = listed.ok ? listed.value.length : 0;

  if (config.firstRunCompleted) {
    return {
      shouldOffer: false,
      reason: "completed",
      config,
      skillCount,
    };
  }

  if (skillCount > 0) {
    return {
      shouldOffer: false,
      reason: "library-already-used",
      config,
      skillCount,
    };
  }

  return {
    shouldOffer: true,
    reason: "not-completed",
    config,
    skillCount: 0,
  };
}

/**
 * Mark first-run done after install or skip.
 */
export async function completeFirstRun(
  outcome: "installed" | "skipped",
  options?: {
    kitHome?: string;
    preferredPack?: string;
  },
): Promise<KitConfig> {
  const kitHome = options?.kitHome ?? getKitHome();
  const preferredPack =
    options?.preferredPack ?? DEFAULT_KIT_CONFIG.preferredPack;

  return updateConfig(
    {
      firstRunCompleted: true,
      firstRunCompletedAt: new Date().toISOString(),
      firstRunOutcome: outcome,
      preferredPack,
    },
    kitHome,
  );
}

export function isFirstRunPackName(name: string): name is FirstRunPackName {
  return FIRST_RUN_PACK_OPTIONS.some((option) => option.name === name);
}

export function listFirstRunPackOptions() {
  return FIRST_RUN_PACK_OPTIONS.map((option) => ({ ...option }));
}

function normalizeConfig(raw: unknown): KitConfig {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) {
    return { ...DEFAULT_KIT_CONFIG };
  }

  const row = raw as Record<string, unknown>;
  const preferredPack =
    typeof row.preferredPack === "string" && row.preferredPack.trim()
      ? row.preferredPack.trim()
      : DEFAULT_KIT_CONFIG.preferredPack;

  const config: KitConfig = {
    version: 1,
    firstRunCompleted: row.firstRunCompleted === true,
    preferredPack,
  };

  if (typeof row.firstRunCompletedAt === "string") {
    config.firstRunCompletedAt = row.firstRunCompletedAt;
  }

  if (row.firstRunOutcome === "installed" || row.firstRunOutcome === "skipped") {
    config.firstRunOutcome = row.firstRunOutcome;
  }

  return config;
}
