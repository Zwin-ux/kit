import { access, mkdtemp, readdir, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { installSkill, listSkills } from "../library/library.js";
import { getKitHome } from "../library/paths.js";
import {
  LINKABLE_HARNESSES,
  resolveHarnessSkillsRoot,
} from "../paths/harness.js";
import { linkSkills } from "../paths/link.js";
import type { HarnessId, PathScope } from "../paths/types.js";
import {
  normalizeSkillMd,
  writeNormalizedSkill,
  type NormalizedSkillMd,
} from "./normalize.js";
import {
  looksKitShaped,
  scoreUnifyCandidate,
  type SkillGrade,
} from "./score.js";

const SKIP_DIRS = new Set([
  ".system",
  "node_modules",
  ".git",
  "dist",
  ".gitkeep",
]);

export interface UnifySourceHit {
  sourceDir: string;
  folderName: string;
  harness: HarnessId | "kit" | "project";
  scope: PathScope | "library";
}

export interface UnifyCandidate {
  name: string;
  score: number;
  grade: SkillGrade;
  description: string;
  sources: UnifySourceHit[];
  fixes: string[];
  signals: string[];
  kitReady: boolean;
  isNoise: boolean;
  noiseReasons: string[];
  isKeeper: boolean;
  inLibrary: boolean;
  normalized: NormalizedSkillMd;
  bestSourceDir: string;
}

export interface UnifyReport {
  projectDir: string;
  kitHome: string;
  dryRun: boolean;
  includeNoise: boolean;
  scanned: number;
  unique: number;
  noiseCount: number;
  keeperCount: number;
  alreadyInLibrary: number;
  adoptReady: number;
  adopted: number;
  linked: number;
  candidates: UnifyCandidate[];
  /** Keepers only, sorted by score. */
  keepers: UnifyCandidate[];
  /** Noise sample for the product story. */
  noiseSample: UnifyCandidate[];
  adoptedNames: string[];
  notes: string[];
}

export type UnifyResult =
  | { ok: true; value: UnifyReport }
  | { ok: false; error: string };

export interface UnifyOptions {
  projectDir?: string;
  kitHome?: string;
  homeDir?: string;
  harnesses?: HarnessId[];
  includeProject?: boolean;
  includeLibrary?: boolean;
  write?: boolean;
  link?: boolean;
  force?: boolean;
  /** Disable noise filter (power user). Default false. */
  includeNoise?: boolean;
  /** Max skills to adopt. Default 25. */
  top?: number;
  /**
   * Minimum score to adopt when writing.
   * Default 70 (S/A band). Listing still returns all non-filtered.
   */
  minScore?: number;
  skillNames?: string[];
  onProgress?: (msg: string) => void;
}

/**
 * Unify the skill mess: scan → normalize → dedupe → rank →
 * optionally adopt keepers into ~/.kit and broadcast via link.
 */
export async function runUnify(
  options: UnifyOptions = {},
): Promise<UnifyResult> {
  const kitHome = options.kitHome ?? getKitHome();
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const write = options.write === true;
  const doLink = options.link === true;
  const force = options.force === true;
  const includeNoise = options.includeNoise === true;
  const minScore = options.minScore ?? 70;
  const top = options.top ?? 25;
  const includeProject = options.includeProject !== false;
  const includeLibrary = options.includeLibrary !== false;
  const harnesses = (options.harnesses ?? LINKABLE_HARNESSES).filter(
    (h) => h !== "kit",
  );
  const want =
    options.skillNames && options.skillNames.length > 0
      ? new Set(options.skillNames)
      : null;
  const progress = options.onProgress;

  const notes: string[] = [];
  const hits: UnifySourceHit[] = [];

  for (const harness of harnesses) {
    for (const scope of ["personal", "project"] as const) {
      if (scope === "project" && !includeProject) continue;
      const root = resolveHarnessSkillsRoot(harness, scope, {
        projectDir,
        kitHome,
        ...(options.homeDir !== undefined
          ? { homeDir: options.homeDir }
          : {}),
      });
      progress?.(`Scanning ${harness} (${scope})…`);
      await collectSkillHits(root, harness, scope, hits);
    }
  }

  if (includeLibrary) {
    progress?.("Scanning Kit library…");
    await collectSkillHits(path.join(kitHome, "skills"), "kit", "library", hits);
  }

  progress?.(`Found ${hits.length} skill folders. Ranking…`);

  const library = await listSkills({ kitHome });
  if (!library.ok) {
    return { ok: false, error: library.error };
  }
  const inLib = new Set(library.value.map((s) => s.name));

  type GroupItem = {
    hit: UnifySourceHit;
    raw: string;
    normalized: NormalizedSkillMd;
    wasKitShaped: boolean;
  };

  const groups = new Map<string, GroupItem[]>();

  for (const hit of hits) {
    let raw: string;
    try {
      raw = await readFile(path.join(hit.sourceDir, "SKILL.md"), "utf8");
    } catch {
      continue;
    }
    const normalized = normalizeSkillMd(raw, { folderName: hit.folderName });
    if (want && !want.has(normalized.name) && !want.has(hit.folderName)) {
      continue;
    }
    const list = groups.get(normalized.name) ?? [];
    list.push({
      hit,
      raw,
      normalized,
      wasKitShaped: looksKitShaped(raw),
    });
    groups.set(normalized.name, list);
  }

  const candidates: UnifyCandidate[] = [];
  for (const [name, list] of groups) {
    // Prefer kit-shaped + longer structured bodies as best source
    list.sort((a, b) => {
      const as =
        (a.wasKitShaped ? 50 : 0) +
        Math.min(40, a.normalized.body.length / 50) +
        (a.normalized.fixes.length === 0 ? 10 : 0);
      const bs =
        (b.wasKitShaped ? 50 : 0) +
        Math.min(40, b.normalized.body.length / 50) +
        (b.normalized.fixes.length === 0 ? 10 : 0);
      return bs - as;
    });
    const best = list[0];
    if (!best) continue;

    const scored = scoreUnifyCandidate({
      normalized: best.normalized,
      sources: list.map((x) => x.hit),
      rawBody: best.raw,
      wasKitShaped: best.wasKitShaped,
    });

    candidates.push({
      name,
      score: scored.score,
      grade: scored.grade,
      description: best.normalized.description,
      sources: list.map((x) => x.hit),
      fixes: best.normalized.fixes,
      signals: scored.signals,
      kitReady: best.normalized.kitReady,
      isNoise: scored.isNoise,
      noiseReasons: scored.noiseReasons,
      isKeeper: scored.isKeeper && best.normalized.kitReady,
      inLibrary: inLib.has(name),
      normalized: best.normalized,
      bestSourceDir: best.hit.sourceDir,
    });
  }

  candidates.sort((a, b) => b.score - a.score);

  const visible = includeNoise
    ? candidates
    : candidates.filter((c) => !c.isNoise);
  const noiseAll = candidates.filter((c) => c.isNoise);
  const keepers = visible.filter((c) => c.isKeeper);
  const noiseSample = noiseAll.slice(0, 5);

  const adoptPool = keepers.filter(
    (c) => c.score >= minScore && c.kitReady && (!c.inLibrary || force),
  );
  const adoptReady = adoptPool.length;

  const adoptedNames: string[] = [];
  let adopted = 0;
  let linked = 0;

  if (write) {
    if (includeNoise) {
      notes.push(
        "WARNING: --all disabled the noise filter. Adopting keepers only still applies unless you lower --min-score.",
      );
    }

    const toAdopt = adoptPool.slice(0, top);
    const tmpRoot = await mkdtemp(path.join(os.tmpdir(), "kit-unify-"));
    try {
      for (const c of toAdopt) {
        const skillDir = await writeNormalizedSkill(tmpRoot, c.normalized);
        const result = await installSkill(skillDir, { kitHome, force: true });
        if (!result.ok) {
          notes.push(`skip ${c.name}: ${result.error}`);
          continue;
        }
        adopted += 1;
        adoptedNames.push(c.name);
        c.inLibrary = true;
      }
    } finally {
      await rm(tmpRoot, { recursive: true, force: true });
    }

    // Link keepers (new adopts ∪ already in library), not only this-run adopts
    if (doLink) {
      const linkNames = [
        ...new Set([
          ...adoptedNames,
          ...keepers.filter((c) => c.inLibrary).map((c) => c.name),
        ]),
      ];
      if (linkNames.length === 0) {
        notes.push(
          "Link requested but no keepers in library. Adopt first or lower --min-score.",
        );
      } else {
        const linkResult = await linkSkills({
          kitHome,
          projectDir,
          harnesses,
          scope: "project",
          write: true,
          force,
          mode: "symlink",
          skillNames: linkNames,
        });
        if (linkResult.ok) {
          linked = linkResult.value.linked;
          if (linkResult.value.failed.length > 0) {
            notes.push(
              `link partial: ${linked} ok, ${linkResult.value.failed.length} failed`,
            );
          } else {
            notes.push(
              `linked ${linked} skill install(s) into project harness folders`,
            );
          }
        } else {
          notes.push(`link failed: ${linkResult.error}`);
        }
      }
    }

    if (adopted === 0) {
      notes.push(
        "Nothing adopted. Try lowering --min-score, raising --top, or fixing empty libraries.",
      );
    }
  } else {
    notes.push(
      `Safe default would adopt ${Math.min(top, adoptReady)} keepers (not ${noiseAll.length} noise skills).`,
    );
    notes.push("Pass --write to adopt keepers into ~/.kit.");
    notes.push("Pass --write --link to also wire keepers into this project.");
    if (doLink && !write) {
      notes.push("--link requires --write (no silent no-op).");
    }
    if (!includeNoise) {
      notes.push("Noise filter on. Use --all to include automation bulk (not recommended).");
    }
  }

  const linkHardFail =
    write && doLink && notes.some((n) => n.startsWith("link failed:"));
  const linkPartial =
    write && doLink && notes.some((n) => n.startsWith("link partial:"));

  const report: UnifyReport = {
    projectDir,
    kitHome,
    dryRun: !write,
    includeNoise,
    scanned: hits.length,
    unique: candidates.length,
    noiseCount: noiseAll.length,
    keeperCount: keepers.length,
    alreadyInLibrary: candidates.filter((c) => c.inLibrary).length,
    adoptReady,
    adopted,
    linked,
    candidates: visible,
    keepers,
    noiseSample,
    adoptedNames,
    notes,
  };

  if (linkHardFail) {
    return { ok: false, error: "Unify link step failed" };
  }
  // Partial link still returns report; CLI should exit 1 if notes mention partial
  if (linkPartial) {
    notes.push("Link had partial failures — check harness folders.");
  }

  return { ok: true, value: report };
}

async function collectSkillHits(
  root: string,
  harness: UnifySourceHit["harness"],
  scope: UnifySourceHit["scope"],
  out: UnifySourceHit[],
): Promise<void> {
  if (!(await exists(root))) return;
  let entries: string[];
  try {
    const dirents = await readdir(root, { withFileTypes: true });
    entries = dirents
      .filter((d) => d.isDirectory() || d.isSymbolicLink())
      .map((d) => d.name)
      .filter((n) => !SKIP_DIRS.has(n) && !n.startsWith("."));
  } catch {
    return;
  }

  for (const folderName of entries) {
    const sourceDir = path.join(root, folderName);
    if (!(await exists(path.join(sourceDir, "SKILL.md")))) continue;
    out.push({ sourceDir, folderName, harness, scope });
  }
}

async function exists(p: string): Promise<boolean> {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
}
