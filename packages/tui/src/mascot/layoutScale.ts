/**
 * Terminal layout scale — fixed mascot slot + content-first.
 *
 * Animation may change pixels inside the rail only.
 * Rail width/height never change on frame tick.
 */

export type LayoutMode = "narrow" | "normal" | "wide";

export interface LayoutScale {
  mode: LayoutMode;
  columns: number;
  rows: number;
  /**
   * Pixel canvas for letterboxing the fox (≤ rail interior).
   */
  mascotFit: { width: number; height: number };
  /** Rail always single-width █. */
  mascotCell: "single";
  /**
   * Reserved rail slot in terminal cells — CONSTANT while animating.
   * MascotPlayer must paint exactly railRows lines of length ≤ railCols.
   */
  railCols: number;
  railRows: number;
  packDetailSize: number;
  showPackDetail: boolean;
  splashFit: { width: number; height: number };
  splashCell: "single" | "double";
  contentMinCols: number;
  contentSoftMax: number;
  listMaxItems: number;
}

const DEFAULT_COLS = 80;
const DEFAULT_ROWS = 24;

export const LAYOUT_CAPS = {
  /** Pinned rail budgets — more room for fox, never thrash. */
  railColsNormal: 12,
  railRowsNormal: 10,
  railColsTall: 14,
  railRowsTall: 12,
  railColsNarrow: 10,
  railRowsNarrow: 9,
  packDetailMax: 4,
  packDetailMinRows: 28,
  splashFitMax: 12,
  /** Slightly slower rail fps reduces full-screen paint thrash. */
  railFrameDelayMs: 210,
} as const;

/**
 * Pick layout from terminal size.
 * Mode changes on resize only — never on animation tick.
 */
export function layoutScaleFromTerminal(
  columns?: number,
  rows?: number,
): LayoutScale {
  const cols = columns && columns > 0 ? columns : DEFAULT_COLS;
  const r = rows && rows > 0 ? rows : DEFAULT_ROWS;

  let mode: LayoutMode = "narrow";
  if (cols >= 110 && r >= 32) mode = "wide";
  else if (cols >= 80 && r >= 24) mode = "normal";
  else if (cols >= 100 || r >= 30) mode = "normal";

  // Fixed rail slot — enough room, constant per mode
  let railCols: number = LAYOUT_CAPS.railColsNormal;
  let railRows: number = LAYOUT_CAPS.railRowsNormal;
  if (mode === "narrow" || r < 22) {
    railCols = LAYOUT_CAPS.railColsNarrow;
    railRows = LAYOUT_CAPS.railRowsNarrow;
  } else if (r >= 32) {
    railCols = LAYOUT_CAPS.railColsTall;
    railRows = LAYOUT_CAPS.railRowsTall;
  }

  // Letterbox fox into interior (leave 1 col air if room)
  const fitW = Math.max(6, railCols - 1);
  const fitH = Math.max(6, railRows);
  const mascotFit = { width: fitW, height: fitH };

  const packDetailSize =
    r >= LAYOUT_CAPS.packDetailMinRows
      ? Math.min(LAYOUT_CAPS.packDetailMax, Math.floor(fitH / 2))
      : 0;

  const splashEdge = Math.min(
    LAYOUT_CAPS.splashFitMax,
    mode === "narrow" ? 10 : 12,
  );

  const listMaxItems = mode === "wide" ? 8 : mode === "normal" ? 4 : 3;

  return {
    mode,
    columns: cols,
    rows: r,
    mascotFit,
    mascotCell: "single",
    railCols,
    railRows,
    packDetailSize,
    showPackDetail: packDetailSize > 0,
    splashFit: { width: splashEdge, height: splashEdge },
    splashCell: "single",
    contentMinCols: mode === "narrow" ? 28 : 40,
    contentSoftMax: mode === "wide" ? 96 : mode === "normal" ? 72 : 48,
    listMaxItems,
  };
}

export function splashMascotFit(scale: LayoutScale): {
  width: number;
  height: number;
  cell: "single" | "double";
} {
  return {
    width: scale.splashFit.width,
    height: scale.splashFit.height,
    cell: scale.splashCell,
  };
}
