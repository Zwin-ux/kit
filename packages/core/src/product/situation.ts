import { access, readdir } from "node:fs/promises";
import path from "node:path";
import { listSkills } from "../library/library.js";
import { getKitHome } from "../library/paths.js";
import {
  LINKABLE_HARNESSES,
  resolveHarnessSkillsRoot,
} from "../paths/harness.js";
import { recommendToolkits } from "../recommend/recommend.js";
import {
  pickStory,
  type SituationSnapshot,
  type UserStory,
} from "./stories.js";

export interface KitSituation {
  kitHome: string;
  projectDir: string;
  snapshot: SituationSnapshot;
  story: UserStory;
  /** One-line headline for CLI */
  headline: string;
}

export interface DetectSituationOptions {
  kitHome?: string;
  projectDir?: string;
  homeDir?: string;
  /** Skip full recommend scan (faster). Default false. */
  skipRecommend?: boolean;
}

/**
 * Lightweight read of the user's world — library size, harness skill counts,
 * project shape — then map to a product story.
 */
export async function detectSituation(
  options: DetectSituationOptions = {},
): Promise<KitSituation> {
  const kitHome = options.kitHome ?? getKitHome();
  const projectDir = path.resolve(options.projectDir ?? process.cwd());

  const listed = await listSkills({ kitHome });
  const libraryCount = listed.ok ? listed.value.length : 0;

  let harnessSkillEstimate = 0;
  for (const harness of LINKABLE_HARNESSES) {
    for (const scope of ["personal", "project"] as const) {
      if (scope === "project") {
        // only count project if it looks like a real project dir
      }
      const root = resolveHarnessSkillsRoot(harness, scope, {
        projectDir,
        kitHome,
        ...(options.homeDir !== undefined
          ? { homeDir: options.homeDir }
          : {}),
      });
      harnessSkillEstimate += await countSkillFolders(root);
    }
  }

  // Cheap keeper/noise estimate without full unify
  // (full numbers come from kit unify; this is for routing only)
  const noiseEstimate = Math.round(harnessSkillEstimate * 0.75);
  const keeperEstimate = Math.max(
    0,
    Math.min(40, Math.round(harnessSkillEstimate * 0.08)),
  );

  let hasProjectSignals = false;
  let recommendedPack: string | null = null;
  let recommendSummary: string | null = null;
  let projectLinkedLikely = false;

  if (!options.skipRecommend) {
    const rec = await recommendToolkits({ projectDir });
    if (rec.ok) {
      hasProjectSignals = rec.value.signals.length > 0;
      recommendedPack = rec.value.topPick;
      recommendSummary = rec.value.summary;
    }
  } else {
    hasProjectSignals = await exists(path.join(projectDir, "package.json"));
  }

  // Heuristic: project .claude/skills or .kit/skills has content
  for (const p of [
    path.join(projectDir, ".kit", "skills"),
    path.join(projectDir, ".claude", "skills"),
    path.join(projectDir, ".codex", "skills"),
  ]) {
    if ((await countSkillFolders(p)) > 0) {
      projectLinkedLikely = true;
      break;
    }
  }

  const snapshot: SituationSnapshot = {
    libraryCount,
    harnessSkillEstimate,
    keeperEstimate,
    noiseEstimate,
    hasProjectSignals,
    recommendedPack,
    recommendSummary,
    projectLinkedLikely,
  };

  const story = pickStory(snapshot);
  const headline = buildHeadline(snapshot, story);

  return { kitHome, projectDir, snapshot, story, headline };
}

function buildHeadline(s: SituationSnapshot, story: UserStory): string {
  if (story.id === "chaos-cleanup") {
    return `~${s.harnessSkillEstimate} skills across agents · library has ${s.libraryCount} · time to unify`;
  }
  if (story.id === "new-repo-agents") {
    return s.recommendSummary
      ? `${s.recommendSummary} · make agents useful on this repo`
      : "This repo needs agent skills · run kit ready";
  }
  if (story.id === "empty-start") {
    return "Fresh Kit install · start with a pack or unify existing skills";
  }
  if (story.id === "multi-agent-sync") {
    return `${s.libraryCount} skills in library · wire them into every agent`;
  }
  return `${s.libraryCount} skills ready · keep shipping`;
}

async function countSkillFolders(root: string): Promise<number> {
  try {
    await access(root);
    const ents = await readdir(root, { withFileTypes: true });
    let n = 0;
    for (const e of ents) {
      if (!e.isDirectory() && !e.isSymbolicLink()) continue;
      if (e.name.startsWith(".") || e.name === "node_modules") continue;
      n += 1;
    }
    return n;
  } catch {
    return 0;
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
