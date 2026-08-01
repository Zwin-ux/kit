/**
 * Pure black 8×8 menu icons — one asset per button / option.
 * Video-game main-menu language: clear silhouette at one cell when mini.
 */

export const MENU_ICON_SIZE = 8;

export type MenuIconBitmap = boolean[];

export const MENU_ICON_IDS = [
  "skills",
  "agents",
  "services",
  "ops",
  "ready",
  "unify",
  "doctor",
  "paths",
  "refresh",
  "ollama",
  "run",
  "install",
  "apply",
  "stop",
  "model",
  "pack",
  "plugin",
  "home",
  "help",
  "quit",
  "point",
  "kit",
  "codex",
  "claude",
  "grok",
  "inspect",
  "build",
] as const;

export type MenuIconId = (typeof MENU_ICON_IDS)[number];

function g(rows: string[]): MenuIconBitmap {
  const out: boolean[] = [];
  for (const row of rows) {
    if (row.length !== MENU_ICON_SIZE) {
      throw new Error(`menu icon row ${row.length}, need ${MENU_ICON_SIZE}`);
    }
    for (const c of row) out.push(c === "#");
  }
  return out;
}

/** # = fill · . = empty */
const ICONS: Record<MenuIconId, MenuIconBitmap> = {
  // Toolbox / skills
  skills: g([
    "........",
    ".######.",
    ".#....#.",
    ".#.##.#.",
    ".#....#.",
    ".#.##.#.",
    ".######.",
    "........",
  ]),
  // Terminal / agents
  agents: g([
    "........",
    ".######.",
    ".#.>..#.",
    ".#....#.",
    ".#....#.",
    ".######.",
    "...##...",
    "..####..",
  ]),
  // Plug / services
  services: g([
    "........",
    "..#..#..",
    "..#..#..",
    ".######.",
    ".#....#.",
    ".#....#.",
    "..####..",
    "........",
  ]),
  // Gear / ops
  ops: g([
    "........",
    "...##...",
    ".#.##.#.",
    "##....##",
    ".#....#.",
    "##....##",
    ".#.##.#.",
    "...##...",
  ]),
  // Rocket / ready
  ready: g([
    "........",
    "...##...",
    "..####..",
    ".##..##.",
    ".##..##.",
    "..####..",
    "...##...",
    "...##...",
  ]),
  // Merge / unify
  unify: g([
    "........",
    ".#....#.",
    ".##..##.",
    "..####..",
    "...##...",
    "..####..",
    ".##..##.",
    "........",
  ]),
  // Stethoscope-ish / doctor
  doctor: g([
    "........",
    ".##..##.",
    ".#....#.",
    "..#..#..",
    "...##...",
    "...##...",
    "..####..",
    "........",
  ]),
  // Path / link
  paths: g([
    "........",
    ".##.....",
    ".#.#....",
    "....#.#.",
    ".....##.",
    "....#.#.",
    ".#.#....",
    ".##.....",
  ]),
  // Refresh arrows
  refresh: g([
    "........",
    "...####.",
    "..#.....",
    ".#..##..",
    "..##..#.",
    ".....#..",
    ".####...",
    "........",
  ]),
  // Local node / ollama
  ollama: g([
    "........",
    "..####..",
    ".#....#.",
    ".#.##.#.",
    ".#....#.",
    "..####..",
    "...##...",
    "........",
  ]),
  // Play triangle
  run: g([
    "........",
    "..#.....",
    "..##....",
    "..###...",
    "..####..",
    "..###...",
    "..##....",
    "..#.....",
  ]),
  // Down arrow install
  install: g([
    "........",
    "...##...",
    "...##...",
    "...##...",
    ".######.",
    "..####..",
    "...##...",
    "........",
  ]),
  // Stamp / apply
  apply: g([
    "........",
    ".######.",
    ".#....#.",
    ".#.##.#.",
    ".#....#.",
    ".######.",
    "...##...",
    "........",
  ]),
  // Stop square
  stop: g([
    "........",
    ".######.",
    ".#....#.",
    ".#....#.",
    ".#....#.",
    ".#....#.",
    ".######.",
    "........",
  ]),
  // Chip / model
  model: g([
    "........",
    ".######.",
    ".#.##.#.",
    ".######.",
    ".#.##.#.",
    ".######.",
    "........",
    "........",
  ]),
  pack: g([
    "........",
    ".######.",
    ".##..##.",
    ".#....#.",
    ".#....#.",
    ".##..##.",
    ".######.",
    "........",
  ]),
  plugin: g([
    "........",
    "..#..#..",
    ".######.",
    ".#....#.",
    ".#....#.",
    ".######.",
    "..#..#..",
    "........",
  ]),
  home: g([
    "........",
    "...##...",
    "..####..",
    ".##..##.",
    ".#....#.",
    ".#.##.#.",
    ".#.##.#.",
    "........",
  ]),
  help: g([
    "........",
    "..####..",
    ".#....#.",
    ".....#..",
    "...##...",
    "...##...",
    "........",
    "...##...",
  ]),
  quit: g([
    "........",
    ".#....#.",
    "..#..#..",
    "...##...",
    "...##...",
    "..#..#..",
    ".#....#.",
    "........",
  ]),
  point: g([
    "........",
    "...##...",
    "..####..",
    ".##..##.",
    "...##...",
    "...##...",
    "...##...",
    "........",
  ]),
  // Kit mark (ears)
  kit: g([
    "........",
    ".##..##.",
    "########",
    ".######.",
    "..####..",
    ".##..##.",
    ".#....#.",
    "........",
  ]),
  codex: g([
    "........",
    ".######.",
    ".#....#.",
    ".#.###..",
    ".#......",
    ".#....#.",
    ".######.",
    "........",
  ]),
  claude: g([
    "........",
    "..####..",
    ".#....#.",
    ".#......",
    ".#......",
    ".#....#.",
    "..####..",
    "........",
  ]),
  grok: g([
    "........",
    ".#....#.",
    ".#...#..",
    ".#.##...",
    ".##.#...",
    ".#...#..",
    ".#....#.",
    "........",
  ]),
  inspect: g([
    "........",
    "..####..",
    ".#....#.",
    ".#....#.",
    "..####..",
    "...#....",
    "..#.....",
    ".#......",
  ]),
  build: g([
    "........",
    ".....#..",
    "....##..",
    "...###..",
    "..####..",
    ".#####..",
    "######..",
    "........",
  ]),
};

/** Single-cell glyphs when bitmap cannot draw (fallback). */
const GLYPH: Record<MenuIconId, string> = {
  skills: "▣",
  agents: "›",
  services: "⬡",
  ops: "⚙",
  ready: "▲",
  unify: "⇄",
  doctor: "+",
  paths: "⇄",
  refresh: "↻",
  ollama: "◉",
  run: "▶",
  install: "↓",
  apply: "▣",
  stop: "■",
  model: "◆",
  pack: "▦",
  plugin: "⬡",
  home: "⌂",
  help: "?",
  quit: "×",
  point: "⌖",
  kit: "♦",
  codex: "C",
  claude: "A",
  grok: "G",
  inspect: "⌕",
  build: "⚒",
};

export function isMenuIconId(id: string): id is MenuIconId {
  return (MENU_ICON_IDS as readonly string[]).includes(id);
}

export function getMenuIconBitmap(id: string): MenuIconBitmap {
  if (isMenuIconId(id)) return ICONS[id];
  return ICONS.kit;
}

/** One ASCII cell for dense rows (game menu list). */
export function menuIconGlyph(id: string): string {
  if (isMenuIconId(id)) return GLYPH[id];
  return "·";
}

export function renderMenuIconLines(id: string): string[] {
  const bmp = getMenuIconBitmap(id);
  const lines: string[] = [];
  for (let y = 0; y < MENU_ICON_SIZE; y++) {
    let line = "";
    for (let x = 0; x < MENU_ICON_SIZE; x++) {
      line += bmp[y * MENU_ICON_SIZE + x] ? "█" : " ";
    }
    lines.push(line);
  }
  return lines;
}

/** Map terminal lane → icon asset. */
export function laneMenuIcon(
  lane: "skills" | "agents" | "services" | "ops",
): MenuIconId {
  return lane;
}

/** Map runner id → icon. */
export function runnerMenuIcon(runnerId: string): MenuIconId {
  if (runnerId === "ollama") return "ollama";
  if (runnerId === "codex") return "codex";
  if (runnerId === "claude-code") return "claude";
  if (runnerId === "grok-build") return "grok";
  return "agents";
}

/** Map ops action id → icon. */
export function opsMenuIcon(opId: string): MenuIconId {
  if (isMenuIconId(opId)) return opId;
  return "ops";
}
