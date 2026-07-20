/**
 * Pure black 8×8 status / UI glyphs for the TUI.
 * Same silhouette language as kit-idle and pack icons: no gray, no AA.
 */

export const STATUS_ICON_SIZE = 8;

/** Row-major: true = black pixel. */
export type StatusIconBitmap = boolean[];

export const STATUS_ICON_IDS = [
  "ok",
  "fail",
  "warn",
  "info",
  "star",
  "keeper",
  "noise",
  "skip",
  "link",
  "folder",
  "pack",
  "skill",
  "arrow",
  "agent-claude",
  "agent-codex",
  "agent-grok",
  "spinner-0",
  "spinner-1",
  "spinner-2",
  "spinner-3",
] as const;

export type StatusIconId = (typeof STATUS_ICON_IDS)[number];

function g(rows: string[]): StatusIconBitmap {
  const out: boolean[] = [];
  for (const row of rows) {
    if (row.length !== STATUS_ICON_SIZE) {
      throw new Error(
        `status icon row length ${row.length}, expected ${STATUS_ICON_SIZE}`,
      );
    }
    for (const c of row) {
      out.push(c === "#");
    }
  }
  if (out.length !== STATUS_ICON_SIZE * STATUS_ICON_SIZE) {
    throw new Error(
      `status icon expected ${STATUS_ICON_SIZE * STATUS_ICON_SIZE} pixels, got ${out.length}`,
    );
  }
  return out;
}

/**
 * Each grid is 8 chars × 8 rows.
 * # = black · . = empty
 */
const ICONS: Record<StatusIconId, StatusIconBitmap> = {
  // Check mark
  ok: g([
    "........",
    "......#.",
    ".....#..",
    "#...#...",
    ".#.#....",
    "..#.....",
    "........",
    "........",
  ]),

  // X fail
  fail: g([
    "........",
    ".#...#..",
    "..#.#...",
    "...#....",
    "..#.#...",
    ".#...#..",
    "........",
    "........",
  ]),

  // Exclamation warn
  warn: g([
    "........",
    "...##...",
    "...##...",
    "...##...",
    "...##...",
    "........",
    "...##...",
    "........",
  ]),

  // i info
  info: g([
    "........",
    "...##...",
    "........",
    "...##...",
    "...##...",
    "...##...",
    "...##...",
    "........",
  ]),

  // Star / top pick
  star: g([
    "........",
    "...#....",
    "..###...",
    ".#####..",
    "..###...",
    ".#.#.#..",
    "........",
    "........",
  ]),

  // Keeper diamond (unify keep)
  keeper: g([
    "........",
    "...#....",
    "..###...",
    ".#####..",
    "..###...",
    "...#....",
    "........",
    "........",
  ]),

  // Noise / filtered (hollow x-ish)
  noise: g([
    "........",
    ".#....#.",
    "..#..#..",
    "...##...",
    "...##...",
    "..#..#..",
    ".#....#.",
    "........",
  ]),

  // Skip dash
  skip: g([
    "........",
    "........",
    "........",
    ".######.",
    ".######.",
    "........",
    "........",
    "........",
  ]),

  // Link chain
  link: g([
    "........",
    ".###....",
    "#...#...",
    "#..###..",
    ".###..#.",
    "...#...#",
    "....###.",
    "........",
  ]),

  // Folder
  folder: g([
    "........",
    ".##.....",
    "######..",
    "#....#..",
    "#....#..",
    "#....#..",
    "######..",
    "........",
  ]),

  // Pack / box
  pack: g([
    "........",
    ".######.",
    "#......#",
    "#.####.#",
    "#......#",
    "#......#",
    ".######.",
    "........",
  ]),

  // Skill / scroll
  skill: g([
    "........",
    ".######.",
    ".#....#.",
    ".#.##.#.",
    ".#....#.",
    ".#.##.#.",
    ".######.",
    "........",
  ]),

  // Next-step arrow
  arrow: g([
    "........",
    "...#....",
    "....#...",
    ".######.",
    "....#...",
    "...#....",
    "........",
    "........",
  ]),

  // Claude-ish C mark
  "agent-claude": g([
    "........",
    "..####..",
    ".#......",
    ".#......",
    ".#......",
    ".#......",
    "..####..",
    "........",
  ]),

  // Codex box-X
  "agent-codex": g([
    "........",
    ".######.",
    ".#.##.#.",
    ".##..##.",
    ".##..##.",
    ".#.##.#.",
    ".######.",
    "........",
  ]),

  // Grok G
  "agent-grok": g([
    "........",
    "..####..",
    ".#......",
    ".#..##..",
    ".#...#..",
    ".#...#..",
    "..####..",
    "........",
  ]),

  // Spinner cycle (arc rotating)
  "spinner-0": g([
    "........",
    "...##...",
    "..#..#..",
    ".#....#.",
    "........",
    "........",
    "........",
    "........",
  ]),
  "spinner-1": g([
    "........",
    "........",
    ".....#..",
    ".....#..",
    ".....#..",
    "....#...",
    "...#....",
    "........",
  ]),
  "spinner-2": g([
    "........",
    "........",
    "........",
    "........",
    ".#....#.",
    "..#..#..",
    "...##...",
    "........",
  ]),
  "spinner-3": g([
    "........",
    "........",
    "..#.....",
    "..#.....",
    "..#.....",
    "...#....",
    "....#...",
    "........",
  ]),
};

/** Map doctor / check levels to icon ids. */
export function levelToStatusIcon(
  level: string,
): Extract<StatusIconId, "ok" | "fail" | "warn" | "info"> {
  if (level === "pass" || level === "ok" || level === "success") return "ok";
  if (level === "fail" || level === "error") return "fail";
  if (level === "warn" || level === "warning") return "warn";
  return "info";
}

/** Map harness id to agent glyph. */
export function harnessToStatusIcon(
  harness: string,
): Extract<StatusIconId, "agent-claude" | "agent-codex" | "agent-grok" | "link"> {
  if (harness.includes("claude")) return "agent-claude";
  if (harness.includes("codex")) return "agent-codex";
  if (harness.includes("grok")) return "agent-grok";
  return "link";
}

export function getStatusIconBitmap(id: StatusIconId | string): StatusIconBitmap {
  if (id in ICONS) {
    return ICONS[id as StatusIconId];
  }
  return ICONS.info;
}

/** Render as terminal lines (single-width blocks). */
export function renderStatusIconLines(
  id: StatusIconId | string,
  options?: { cell?: string; empty?: string },
): string[] {
  const cell = options?.cell ?? "█";
  const empty = options?.empty ?? " ";
  const pixels = getStatusIconBitmap(id);
  const lines: string[] = [];
  for (let y = 0; y < STATUS_ICON_SIZE; y++) {
    let line = "";
    for (let x = 0; x < STATUS_ICON_SIZE; x++) {
      line += pixels[y * STATUS_ICON_SIZE + x] ? cell : empty;
    }
    lines.push(line);
  }
  return lines;
}

/**
 * Dense one-cell glyph for list rows (prefer unicode that matches meaning).
 * Full 8×8 still available via StatusIcon size="full".
 */
export function statusIconGlyph(id: StatusIconId | string): string {
  const glyphs: Record<string, string> = {
    ok: "✓",
    fail: "✗",
    warn: "!",
    info: "·",
    star: "★",
    keeper: "◆",
    noise: "×",
    skip: "–",
    link: "⛓",
    folder: "▣",
    pack: "▦",
    skill: "▤",
    arrow: "→",
    "agent-claude": "C",
    "agent-codex": "X",
    "agent-grok": "G",
    "spinner-0": "⠋",
    "spinner-1": "⠙",
    "spinner-2": "⠹",
    "spinner-3": "⠸",
  };
  return glyphs[id] ?? "·";
}

export const SPINNER_ICON_FRAMES: StatusIconId[] = [
  "spinner-0",
  "spinner-1",
  "spinner-2",
  "spinner-3",
];
