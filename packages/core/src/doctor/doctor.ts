import { access, readdir } from "node:fs/promises";
import path from "node:path";
import { KIT_PACKAGE_VERSION } from "@kit-skills/shared";
import { readAuthSession } from "../auth/store.js";
import { getRegistryUrl } from "../auth/login.js";
import { getFirstRunStatus, readConfig } from "../config/config.js";
import { listSkills } from "../library/library.js";
import { getConfigPath, getKitHome, getSkillsDir } from "../library/paths.js";
import { listPacks } from "../pack/loadPack.js";
import {
  resolvePacksRoot,
  resolveSkillsCatalogRoot,
} from "../pack/resolveRoots.js";
import { describePaths } from "../paths/describe.js";
import type { CheckResult } from "../test/types.js";

export interface DoctorReport {
  ok: boolean;
  version: string;
  kitHome: string;
  projectDir: string;
  checks: CheckResult[];
  summary: {
    passed: number;
    failed: number;
    warnings: number;
  };
}

export interface DoctorOptions {
  kitHome?: string;
  projectDir?: string;
  /** Also check mascot frames under assets/pixel. Default true. */
  checkAssets?: boolean;
}

const FRAME_FILES = [
  "kit-frame-1.png",
  "kit-frame-2.png",
  "kit-frame-3.png",
  "kit-frame-4.png",
  "kit-frame-5.png",
  "kit-frame-6.png",
] as const;

/**
 * One-shot health check for a Kit install and local workspace.
 */
export async function runDoctor(
  options: DoctorOptions = {},
): Promise<DoctorReport> {
  const kitHome = options.kitHome ?? getKitHome();
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const checkAssets = options.checkAssets !== false;
  const checks: CheckResult[] = [];

  checks.push({
    id: "version",
    level: "info",
    message: `Kit package version ${KIT_PACKAGE_VERSION}`,
  });

  checks.push({
    id: "node",
    level: "info",
    message: `Node ${process.version}`,
  });

  // Kit home / config
  checks.push({
    id: "kit-home",
    level: "info",
    message: `KIT_HOME → ${kitHome}`,
  });

  try {
    const config = await readConfig(kitHome);
    checks.push({
      id: "config",
      level: "pass",
      message: `config.json readable (firstRunCompleted=${config.firstRunCompleted}, preferredPack=${config.preferredPack})`,
      detail: getConfigPath(kitHome),
    });
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    checks.push({
      id: "config",
      level: "fail",
      message: `Cannot read config: ${detail}`,
    });
  }

  const firstRun = await getFirstRunStatus(kitHome);
  checks.push({
    id: "first-run",
    level: firstRun.shouldOffer ? "warn" : "pass",
    message: firstRun.shouldOffer
      ? "First-run not completed (empty library)."
      : `First-run settled (${firstRun.reason}, ${firstRun.skillCount} skill(s)).`,
  });

  const session = await readAuthSession(kitHome);
  if (session) {
    checks.push({
      id: "auth",
      level: "pass",
      message: `Logged in as @${session.user.login}`,
      detail: session.registryUrl,
    });
  } else {
    checks.push({
      id: "auth",
      level: "warn",
      message: `Not logged in (registry ${getRegistryUrl()}). Run: kit login`,
    });
  }

  try {
    const regRes = await fetch(`${getRegistryUrl()}/health`, {
      headers: { Accept: "application/json" },
    });
    if (regRes.ok) {
      const body = (await regRes.json()) as {
        authConfigured?: boolean;
        appCredentialsConfigured?: boolean;
      };
      checks.push({
        id: "registry",
        level: "pass",
        message: `Registry reachable (${getRegistryUrl()}) authConfigured=${Boolean(body.authConfigured)} appKey=${Boolean(body.appCredentialsConfigured)}`,
      });
    } else {
      checks.push({
        id: "registry",
        level: "warn",
        message: `Registry HTTP ${regRes.status} at ${getRegistryUrl()}`,
      });
    }
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    checks.push({
      id: "registry",
      level: "warn",
      message: `Registry unreachable: ${detail}`,
    });
  }

  const listed = await listSkills({ kitHome });
  if (!listed.ok) {
    checks.push({
      id: "library",
      level: "fail",
      message: listed.error,
    });
  } else {
    checks.push({
      id: "library",
      level: listed.value.length > 0 ? "pass" : "warn",
      message:
        listed.value.length > 0
          ? `Library has ${listed.value.length} skill(s) at ${getSkillsDir(kitHome)}`
          : `Library empty at ${getSkillsDir(kitHome)}. Try: kit init`,
    });
  }

  const packsRoot = await resolvePacksRoot({ startDir: projectDir });
  if (!packsRoot) {
    checks.push({
      id: "packs-root",
      level: "warn",
      message:
        "packs/ not found. Set KIT_PACKS or run from the Kit repository.",
    });
  } else {
    const packs = await listPacks({ packsRoot, startDir: projectDir });
    if (!packs.ok) {
      checks.push({
        id: "packs-root",
        level: "fail",
        message: packs.error,
        detail: packsRoot,
      });
    } else {
      checks.push({
        id: "packs-root",
        level: packs.value.length > 0 ? "pass" : "warn",
        message: `packs/ → ${packsRoot} (${packs.value.length} pack(s))`,
      });
    }
  }

  const skillsRoot = await resolveSkillsCatalogRoot({
    startDir: projectDir,
    ...(packsRoot ? { packsRoot } : {}),
  });
  if (!skillsRoot) {
    checks.push({
      id: "skills-catalog",
      level: "warn",
      message: "skills/ catalog not found near the workspace.",
    });
  } else {
    checks.push({
      id: "skills-catalog",
      level: "pass",
      message: `skills/ catalog → ${skillsRoot}`,
    });
  }

  const paths = await describePaths({ kitHome, projectDir });
  if (!paths.ok) {
    checks.push({
      id: "paths",
      level: "fail",
      message: paths.error,
    });
  } else {
    const existing = paths.value.entries.filter((e) => e.exists).length;
    checks.push({
      id: "paths",
      level: "pass",
      message: `Harness map OK (${existing}/${paths.value.entries.length} roots exist on disk).`,
    });
  }

  if (checkAssets) {
    const assetsDir = await findPixelAssets(projectDir);
    if (!assetsDir) {
      checks.push({
        id: "assets",
        level: "warn",
        message:
          "assets/pixel not found. TUI will use built-in placeholder frames.",
      });
    } else {
      let present = 0;
      for (const name of FRAME_FILES) {
        if (await pathExists(path.join(assetsDir, name))) present += 1;
      }
      const gif = await pathExists(path.join(assetsDir, "kit-idle.gif"));
      if (present === FRAME_FILES.length) {
        checks.push({
          id: "assets",
          level: "pass",
          message: `Mascot frames complete (${present}/6)${gif ? " + kit-idle.gif" : ""}.`,
          detail: assetsDir,
        });
      } else {
        checks.push({
          id: "assets",
          level: "warn",
          message: `Mascot frames partial (${present}/6) in ${assetsDir}`,
        });
      }
    }
  }

  const failed = checks.filter((c) => c.level === "fail").length;
  const warnings = checks.filter((c) => c.level === "warn").length;
  const passed = checks.filter((c) => c.level === "pass").length;

  return {
    ok: failed === 0,
    version: KIT_PACKAGE_VERSION,
    kitHome,
    projectDir,
    checks,
    summary: { passed, failed, warnings },
  };
}

async function findPixelAssets(startDir: string): Promise<string | undefined> {
  let current = path.resolve(startDir);
  for (let i = 0; i < 8; i++) {
    const candidate = path.join(current, "assets", "pixel");
    if (await pathExists(candidate)) {
      try {
        await readdir(candidate);
        return candidate;
      } catch {
        // continue
      }
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  const fromEnv = process.env.KIT_ASSETS?.trim();
  if (fromEnv && (await pathExists(fromEnv))) return path.resolve(fromEnv);
  return undefined;
}

async function pathExists(target: string): Promise<boolean> {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}
