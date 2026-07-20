/**
 * Terminal-aware scale so full-screen and tiny panes both keep art intact.
 * Icons stay smaller than the mascot rail; canvas sizes are fixed (no clip jump).
 */

export type LayoutMode = "narrow" | "normal" | "wide";

export interface LayoutScale {
  mode: LayoutMode;
  columns: number;
  rows: number;
  /**
   * Pixel canvas for mascot (source grid before cell expansion).
   * All frames letterbox into this — same terminal footprint every frame.
   */
  mascotFit: { width: number; height: number };
  /** Single-width █ for rail, double ██ for hero splash when wide enough. */
  mascotCell: "single" | "double";
  /** Terminal columns reserved for mascot rail (includes padding). */
  railCols: number;
  /** Pack detail silhouette edge (always ≤ mascot height). */
  packDetailSize: number;
  /** Status full icon size. */
  statusDetailSize: number;
}

const DEFAULT_COLS = 80;
const DEFAULT_ROWS = 24;

/**
 * Pick layout from terminal size.
 * Full-screen (large) gets a bigger fox; pack logos stay smaller than fox.
 */
export function layoutScaleFromTerminal(
  columns?: number,
  rows?: number,
): LayoutScale {
  const cols = columns && columns > 0 ? columns : DEFAULT_COLS;
  const r = rows && rows > 0 ? rows : DEFAULT_ROWS;

  // Wide / full-screen: room for double-wide cells + taller canvas
  if (cols >= 110 && r >= 32) {
    const fit = { width: 14, height: 14 };
    return {
      mode: "wide",
      columns: cols,
      rows: r,
      mascotFit: fit,
      mascotCell: "double",
      railCols: fit.width * 2 + 4,
      packDetailSize: 8, // always smaller than 14
      statusDetailSize: 8,
    };
  }

  // Normal desktop terminal
  if (cols >= 80 && r >= 24) {
    const fit = { width: 12, height: 12 };
    return {
      mode: "normal",
      columns: cols,
      rows: r,
      mascotFit: fit,
      mascotCell: "single",
      railCols: fit.width + 4,
      packDetailSize: 8,
      statusDetailSize: 8,
    };
  }

  // Narrow / short (small panes, split terminals)
  const fit = { width: 10, height: 10 };
  return {
    mode: "narrow",
    columns: cols,
    rows: r,
    mascotFit: fit,
    mascotCell: "single",
    railCols: fit.width + 3,
    packDetailSize: 6,
    statusDetailSize: 6,
  };
}

/** Splash hero: larger on big screens, still letterboxed. */
export function splashMascotFit(scale: LayoutScale): {
  width: number;
  height: number;
  cell: "single" | "double";
} {
  if (scale.mode === "wide") {
    return { width: 18, height: 16, cell: "double" };
  }
  if (scale.mode === "normal") {
    return { width: 14, height: 14, cell: "double" };
  }
  return { width: 12, height: 12, cell: "single" };
}
