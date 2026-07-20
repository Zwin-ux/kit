import path from "node:path";
import { completeFirstRun } from "../config/config.js";
import { getKitHome } from "../library/paths.js";
import { applyPack, installPack } from "../pack/installPack.js";
import { linkSkills } from "../paths/link.js";
import type { HarnessId } from "../paths/types.js";
import type { PackLoadOptions } from "../pack/loadPack.js";
import { recommendToolkits } from "../recommend/recommend.js";
import { runDoctor } from "../doctor/doctor.js";
import { runUnify } from "../unify/unify.js";
import { detectSituation } from "./situation.js";
import type { UserStory } from "./stories.js";

export interface ReadyOptions extends PackLoadOptions {
  projectDir?: string;
  kitHome?: string;
  homeDir?: string;
  /** Override pack instead of recommend. */
  pack?: string;
  /** When false, plan only. Default false. */
  write?: boolean;
  /**
   * Also run unify keepers before link.
   * Opt-in only — never auto-run from chaos story alone.
   */
  unify?: boolean;
  /** Harnesses to link. Default all linkable. */
  harnesses?: HarnessId[];
  /**
   * Force overwrite of existing harness skill dirs on link.
   * Default false (skip existing). Pack catalog install still replaces.
   */
  force?: boolean;
  onProgress?: (msg: string) => void;
}

export interface ReadyReport {
  dryRun: boolean;
  projectDir: string;
  story: UserStory;
  packName: string;
  recommendSummary: string;
  steps: Array<{
    id: string;
    status: "planned" | "done" | "skipped" | "failed";
    detail: string;
  }>;
  doctorOk: boolean | null;
  notes: string[];
  /**
   * Write path: true only when no failed steps and doctor has zero failures.
   * Dry-run: always false.
   */
  complete: boolean;
}

export type ReadyResult =
  | { ok: true; value: ReadyReport }
  | { ok: false; error: string; value?: ReadyReport };

/**
 * One-shot: make this project agent-ready.
 * recommend → install pack → apply → optional unify → link → doctor
 *
 * Write path: ok only when complete (agents actually wired + doctor green).
 */
export async function runReady(
  options: ReadyOptions = {},
): Promise<ReadyResult> {
  const kitHome = options.kitHome ?? getKitHome();
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const write = options.write === true;
  const doUnify = options.unify === true;
  const force = options.force === true;
  const harnesses = options.harnesses ?? [
    "claude-code" as const,
    "codex" as const,
    "grok-build" as const,
  ];
  const progress = options.onProgress;
  const notes: string[] = [];
  const steps: ReadyReport["steps"] = [];

  const dirGuard = guardProjectDir(projectDir, write, force);
  if (!dirGuard.ok) {
    return { ok: false, error: dirGuard.error };
  }

  progress?.("Reading your setup…");
  const situation = await detectSituation({
    kitHome,
    projectDir,
    ...(options.homeDir !== undefined ? { homeDir: options.homeDir } : {}),
  });

  const packOpts: PackLoadOptions = {
    ...(options.packsRoot !== undefined ? { packsRoot: options.packsRoot } : {}),
    ...(options.skillsRoot !== undefined
      ? { skillsRoot: options.skillsRoot }
      : {}),
  };

  progress?.("Recommending a pack…");
  const rec = await recommendToolkits({ projectDir, ...packOpts });
  if (!rec.ok) {
    return { ok: false, error: rec.error };
  }

  const packName =
    options.pack?.trim() || rec.value.topPick || "essentials";
  const recommendSummary = rec.value.summary;

  if (!write) {
    steps.push({
      id: "pack-install",
      status: "planned",
      detail: `Would install pack "${packName}" into ~/.kit`,
    });
    steps.push({
      id: "pack-apply",
      status: "planned",
      detail: `Would apply "${packName}" to ${projectDir}`,
    });
    if (doUnify) {
      steps.push({
        id: "unify",
        status: "planned",
        detail: "Would adopt skill keepers from Claude/Codex/Grok into ~/.kit",
      });
    } else {
      steps.push({
        id: "unify",
        status: "skipped",
        detail: "Skipped (pass --unify to clean personal skill dumps)",
      });
    }
    steps.push({
      id: "link",
      status: "planned",
      detail: `Would link library skills → ${harnesses.join(", ")} (project)`,
    });
    steps.push({
      id: "doctor",
      status: "planned",
      detail: "Would run kit doctor",
    });
    notes.push("Dry-run. Pass --write to apply.");
    if (situation.story.id === "chaos-cleanup" && !doUnify) {
      notes.push(
        "Many agent skill folders detected. Pass --unify with --write to also clean personal dumps.",
      );
    }
    return {
      ok: true,
      value: {
        dryRun: true,
        projectDir,
        story: situation.story,
        packName,
        recommendSummary,
        steps,
        doctorOk: null,
        notes,
        complete: false,
      },
    };
  }

  // --- write path ---
  progress?.(`Installing pack ${packName}…`);
  const installed = await installPack(packName, {
    kitHome,
    force: true,
    ...packOpts,
  });
  if (!installed.ok) {
    steps.push({
      id: "pack-install",
      status: "failed",
      detail: installed.error,
    });
    return { ok: false, error: installed.error };
  }
  steps.push({
    id: "pack-install",
    status: "done",
    detail: `Installed ${installed.value.installed.length} skills from ${packName}`,
  });

  progress?.(`Applying pack to project…`);
  const applied = await applyPack(packName, {
    kitHome,
    projectDir,
    force: true,
    ...packOpts,
  });
  if (!applied.ok) {
    steps.push({ id: "pack-apply", status: "failed", detail: applied.error });
    return { ok: false, error: applied.error };
  }
  steps.push({
    id: "pack-apply",
    status: "done",
    detail: `Applied to ${projectDir}`,
  });

  await completeFirstRun("installed", {
    kitHome,
    preferredPack: packName,
  });

  if (doUnify) {
    progress?.("Unifying personal skill keepers…");
    const unified = await runUnify({
      kitHome,
      projectDir,
      write: true,
      link: false,
      force,
      top: 25,
      minScore: 70,
      includeLibrary: true,
      ...(options.homeDir !== undefined ? { homeDir: options.homeDir } : {}),
      ...(progress ? { onProgress: progress } : {}),
    });
    if (unified.ok) {
      steps.push({
        id: "unify",
        status: "done",
        detail: `Adopted ${unified.value.adopted} keepers (scanned ${unified.value.scanned})`,
      });
    } else {
      steps.push({
        id: "unify",
        status: "failed",
        detail: unified.error,
      });
      notes.push(`Unify failed: ${unified.error}`);
    }
  } else {
    steps.push({
      id: "unify",
      status: "skipped",
      detail: "Skipped (pass --unify to clean personal skill dumps)",
    });
    if (situation.story.id === "chaos-cleanup") {
      notes.push(
        "Chaos pile detected but unify skipped. Re-run with --unify if you want keepers adopted.",
      );
    }
  }

  progress?.("Linking into agent harnesses…");
  const linked = await linkSkills({
    kitHome,
    projectDir,
    harnesses,
    scope: "project",
    write: true,
    force,
    mode: "symlink",
  });
  if (linked.ok) {
    if (linked.value.failed.length > 0) {
      steps.push({
        id: "link",
        status: "failed",
        detail: `Linked ${linked.value.linked}, failed ${linked.value.failed.length}`,
      });
      notes.push(
        `Link partial failures: ${linked.value.failed
          .slice(0, 5)
          .map((f) => f.skillName)
          .join(", ")}`,
      );
    } else {
      steps.push({
        id: "link",
        status: "done",
        detail: `Linked ${linked.value.linked} (skipped ${linked.value.skipped})`,
      });
    }
  } else {
    steps.push({ id: "link", status: "failed", detail: linked.error });
    notes.push(`Link failed: ${linked.error}`);
  }

  progress?.("Running doctor…");
  const doctor = await runDoctor({ kitHome, projectDir, checkAssets: false });
  steps.push({
    id: "doctor",
    status: doctor.ok ? "done" : "failed",
    detail: doctor.ok
      ? `Doctor green (${doctor.summary.passed} checks)`
      : `Doctor found ${doctor.summary.failed} failure(s)`,
  });

  const failedSteps = steps.filter((s) => s.status === "failed");
  const complete = failedSteps.length === 0 && doctor.ok;

  if (complete) {
    notes.push("Agents on this project can load Kit skills.");
    notes.push(
      "Open work in Claude Code / Codex from this folder, or run kit tui.",
    );
  } else {
    notes.push(
      `Incomplete: ${failedSteps.map((s) => s.id).join(", ") || "doctor"}. Fix and re-run kit ready --write.`,
    );
  }

  const report: ReadyReport = {
    dryRun: false,
    projectDir,
    story: situation.story,
    packName,
    recommendSummary,
    steps,
    doctorOk: doctor.ok,
    notes,
    complete,
  };

  if (!complete) {
    return {
      ok: false,
      error: `Ready incomplete (${failedSteps.map((s) => s.id).join(", ") || "doctor"}). Project: ${projectDir}`,
      value: report,
    };
  }

  return { ok: true, value: report };
}

function guardProjectDir(
  projectDir: string,
  write: boolean,
  force: boolean,
): { ok: true } | { ok: false; error: string } {
  if (!write) return { ok: true };

  const resolved = path.resolve(projectDir);
  const homeRaw = process.env.USERPROFILE ?? process.env.HOME ?? "";
  const home = homeRaw ? path.resolve(homeRaw) : "";
  const root = path.parse(resolved).root;

  const banned =
    home &&
    (resolved === home ||
      resolved === root ||
      resolved === path.resolve(home, "Downloads") ||
      resolved === path.resolve(home, "Desktop"));

  if (!force && banned) {
    return {
      ok: false,
      error: `Refusing to write kit state into ${resolved}. Pass --dir <project> or --force if intentional.`,
    };
  }

  return { ok: true };
}
