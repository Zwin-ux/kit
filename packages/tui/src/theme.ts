/**
 * Kit visual system — from PIXEL_ART.md + marketing generators.
 *
 * Paper ink console, not Charm/Clack cyan:
 * - Accent orange #C45C2A (fox brand, arrows, rules, CTA heat)
 * - Green for OK / success rows
 * - High contrast default text + dim muted meta
 * - Command-style STE labels (KIT SETUP, OK PACK INSTALL)
 *
 * Hierarchy without font size: position, bold, inverse, orange, dim.
 * Color is never the only signal — pair with letters (OK / ! / →).
 */

export type InkColor =
  | "black"
  | "red"
  | "green"
  | "yellow"
  | "blue"
  | "magenta"
  | "cyan"
  | "white"
  | "gray"
  | "blackBright"
  | "redBright"
  | "greenBright"
  | "yellowBright"
  | "blueBright"
  | "magentaBright"
  | "cyanBright"
  | "whiteBright"
  | `#${string}`;

/** Truecolor orange from generate-readme-assets.py ACCENT */
export const ORANGE = "#C45C2A" as const;

export const theme = {
  /** Fox brand / CTA / arrows / top rule */
  accent: ORANGE as InkColor,
  /** Success, OK rows, DONE */
  success: "green" as InkColor,
  /** Confirm write / pending caution */
  warning: "yellow" as InkColor,
  /** Failed / danger */
  error: "red" as InkColor,
} as const;

/** Monochrome-safe status marks (marketing uses → ✓ OK) */
export const mark = {
  arrow: "→",
  ok: "OK",
  fail: "!",
  pending: "·",
  run: ">",
  skip: "-",
  cta: "▶",
  sep: "·",
} as const;

export function stepMark(
  status: "pending" | "running" | "done" | "failed" | "skipped",
): string {
  switch (status) {
    case "done":
      return mark.ok;
    case "failed":
      return mark.fail;
    case "running":
      return mark.run;
    case "skipped":
      return mark.skip;
    default:
      return mark.pending;
  }
}

export function stepTone(
  status: "pending" | "running" | "done" | "failed" | "skipped",
): InkColor | undefined {
  switch (status) {
    case "done":
      return theme.success;
    case "failed":
      return theme.error;
    case "running":
      return theme.accent;
    default:
      return undefined;
  }
}
