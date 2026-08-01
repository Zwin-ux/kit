import type { ReadyReport, UnifyReport } from "@mzwin/kit-core";

/** One sticky status line after a ready run. */
export function formatReadyStatus(report: ReadyReport): string {
  const mode = report.dryRun ? "plan" : report.complete ? "ready" : "partial";
  const fails = report.steps.filter((s) => s.status === "failed").length;
  if (fails > 0) {
    return `Ready ${mode}: ${fails} failed · pack ${report.packName}`;
  }
  if (report.dryRun) {
    return `Ready plan · pack ${report.packName} · y write · n cancel`;
  }
  if (report.complete) {
    return `Ready complete · ${report.packName} · agents linked`;
  }
  return `Ready partial · ${report.packName} · check doctor (d)`;
}

/** Compact step list for flash / multi-line feedback. */
export function formatReadySteps(report: ReadyReport, max = 4): string[] {
  const lines = report.steps.slice(0, max).map((step) => {
    const mark =
      step.status === "done"
        ? "+"
        : step.status === "failed"
          ? "!"
          : step.status === "skipped"
            ? "-"
            : "~";
    return `${mark} ${step.id}: ${step.detail}`;
  });
  if (report.notes[0]) {
    lines.push(report.notes[0]!);
  }
  return lines;
}

export function formatUnifyStatus(report: UnifyReport): string {
  if (report.dryRun) {
    return `Unify plan · ${report.scanned} scanned · ${report.keeperCount} keepers · ${report.adoptReady} adopt · y write`;
  }
  if (report.adopted > 0) {
    const link =
      report.linked > 0 ? ` · linked ${report.linked}` : "";
    return `Unify wrote · adopted ${report.adopted}${link}`;
  }
  return `Unify · ${report.keeperCount} keepers · ${report.alreadyInLibrary} already in library`;
}

export function formatUnifyPreview(report: UnifyReport, max = 3): string[] {
  const lines = [
    `${report.scanned} folders · ${report.unique} unique · ${report.noiseCount} noise · ${report.keeperCount} keepers`,
  ];
  for (const k of report.keepers.slice(0, max)) {
    lines.push(`  ${k.grade} ${k.score} ${k.name}`);
  }
  if (report.keepers.length > max) {
    lines.push(`  +${report.keepers.length - max} more keepers`);
  }
  if (report.notes[0]) {
    lines.push(report.notes[0]!);
  }
  return lines;
}
