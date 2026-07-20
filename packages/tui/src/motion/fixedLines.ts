/**
 * Pure helpers for selection-stable terminal copy.
 * Selection must never change row count or glyph cell width.
 */

export type SelectDirection = "up" | "down" | "none";

/** Split text into exactly `lineCount` lines, each ≤ maxCols (pad with spaces). */
export function fixedLines(
  text: string,
  lineCount: number,
  maxCols: number,
): string[] {
  const cols = Math.max(1, maxCols);
  const n = Math.max(1, lineCount);
  const clean = (text ?? "").replace(/\s+/g, " ").trim();
  const out: string[] = [];

  if (!clean) {
    for (let i = 0; i < n; i++) out.push(" ".repeat(cols));
    return out;
  }

  let remaining = clean;
  for (let i = 0; i < n; i++) {
    if (!remaining) {
      out.push(" ".repeat(cols));
      continue;
    }
    if (remaining.length <= cols) {
      out.push(remaining.padEnd(cols, " "));
      remaining = "";
      continue;
    }
    // Prefer break at space
    let cut = cols;
    const slice = remaining.slice(0, cols + 1);
    const sp = slice.lastIndexOf(" ");
    if (sp > cols * 0.4) cut = sp;
    const piece = remaining.slice(0, cut).trimEnd();
    out.push(piece.padEnd(cols, " ").slice(0, cols));
    remaining = remaining.slice(cut).trimStart();
  }
  return out;
}

/** One truncated line, fixed width. */
export function fixedLine(text: string, maxCols: number): string {
  return fixedLines(text, 1, maxCols)[0] ?? " ".repeat(Math.max(1, maxCols));
}

/** ASCII single-cell pack marks (display width 1). */
export const ASCII_PACK_GLYPHS: Record<string, string> = {
  essentials: "*",
  "web-app": "#",
  library: "=",
  "cli-tool": ">",
  "api-service": "+",
  "full-stack": "%",
  "data-ml": "~",
};

export function asciiPackGlyph(packName: string): string {
  return ASCII_PACK_GLYPHS[packName] ?? "o";
}

/** Select cursor — always 2 ASCII cells. */
export function selectCursorGlyph(
  selected: boolean,
  direction: "up" | "down" | "none" = "none",
): string {
  if (!selected) return "  ";
  if (direction === "up") return "^ ";
  if (direction === "down") return "v ";
  return "> ";
}
