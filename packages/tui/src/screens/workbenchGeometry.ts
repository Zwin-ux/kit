/**
 * Single source of truth for Workbench layout + mouse hit rows.
 * Render and hit-testing must use the same numbers.
 */

export type WorkbenchLayoutMode = "compact" | "standard" | "wide";

export interface WorkbenchGeometry {
  mode: WorkbenchLayoutMode;
  columns: number;
  rows: number;
  paddingX: number;
  /** 1-based terminal row for title */
  titleRow: number;
  /** 1-based row for lane tabs */
  tabRow: number;
  /** 1-based row for NEXT / setup strip */
  statusRow: number;
  /** 1-based row for action flash */
  flashRow: number;
  /** 1-based row of first list item */
  listStartRow: number;
  /** How many list rows fit */
  listRows: number;
  /** Full-width list (no split sidebar) for reliable clicks */
  listCol0: number;
  listCol1: number;
  /** 1-based row for primary action buttons */
  actionRow: number;
  /** 1-based first log line */
  logStartRow: number;
  logRows: number;
  showStatus: boolean;
}

function clamp(n: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, n));
}

/**
 * Simple top-to-bottom layout (no dual-column sidebar).
 * Rows are 1-based to match SGR mouse coordinates.
 */
export function workbenchGeometry(
  columns: number,
  rows: number,
): WorkbenchGeometry {
  const cols = Math.max(40, columns);
  const r = Math.max(14, rows);
  const compact = cols < 72 || r < 20;
  const wide = cols >= 120 && r >= 32;
  const mode: WorkbenchLayoutMode = compact
    ? "compact"
    : wide
      ? "wide"
      : "standard";
  const paddingX = compact ? 1 : 2;

  // Fixed chrome stack (1-based):
  // 1 title, 2 tabs, 3 status, 4 flash, 5 blank, 6+ list ...
  const titleRow = 1;
  const tabRow = 2;
  const statusRow = 3;
  const flashRow = 4;
  const listStartRow = 6;

  // Bottom: footer keys (2) + action bar (1) + gap
  const footerReserve = 3;
  const actionRow = Math.max(listStartRow + 3, r - footerReserve);
  // Log sits between list and action bar
  const listRows = clamp(actionRow - listStartRow - 2, 3, wide ? 14 : 10);
  const logStartRow = listStartRow + listRows + 1;
  const logRows = clamp(actionRow - logStartRow - 1, 2, wide ? 12 : 6);

  return {
    mode,
    columns: cols,
    rows: r,
    paddingX,
    titleRow,
    tabRow,
    statusRow,
    flashRow,
    listStartRow,
    listRows,
    listCol0: paddingX + 1,
    listCol1: cols - paddingX,
    actionRow,
    logStartRow,
    logRows,
    showStatus: r >= 16,
  };
}

/** Window start index for a selected item in a capped list. */
export function windowOffset(
  length: number,
  selected: number,
  maximum: number,
): number {
  if (length <= maximum) return 0;
  return Math.max(
    0,
    Math.min(selected - Math.floor(maximum / 2), length - maximum),
  );
}

export function windowSlice<T>(
  items: T[],
  selected: number,
  maximum: number,
): { items: T[]; offset: number } {
  const offset = windowOffset(items.length, selected, maximum);
  return {
    offset,
    items: items.slice(offset, offset + maximum),
  };
}
