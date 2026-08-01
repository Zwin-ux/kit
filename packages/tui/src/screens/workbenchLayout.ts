export type WorkbenchLayoutMode = "compact" | "standard" | "wide";

export interface WorkbenchLayout {
  mode: WorkbenchLayoutMode;
  columns: number;
  rows: number;
  paddingX: number;
  sidebarWidth: number;
  outputRows: number;
  runnerRows: number;
  serviceRows: number;
  showProjectPath: boolean;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.max(minimum, Math.min(maximum, value));
}

/**
 * Workbench has denser breakpoints than Kit's mascot-led screens.
 * The job and its live output own the available terminal area.
 */
export function workbenchLayoutFromTerminal(
  columns: number,
  rows: number,
): WorkbenchLayout {
  const safeColumns = Math.max(40, columns);
  const safeRows = Math.max(14, rows);
  const compact = safeColumns < 78 || safeRows < 22;
  const wide = safeColumns >= 120 && safeRows >= 32;
  const mode: WorkbenchLayoutMode = compact
    ? "compact"
    : wide
      ? "wide"
      : "standard";

  const paddingX = compact ? 1 : 2;
  const sidebarWidth =
    mode === "compact"
      ? 0
      : clamp(Math.round(safeColumns * (wide ? 0.24 : 0.3)), 24, 36);
  const reservedRows =
    mode === "compact"
      ? 12
      : mode === "wide"
        ? 10
        : 11;
  const outputRows = clamp(
    safeRows - reservedRows,
    mode === "compact" ? 3 : 5,
    mode === "wide" ? 28 : 16,
  );
  const runnerRows =
    mode === "compact" ? clamp(safeRows - 16, 1, 3) : 4;
  const serviceRows = clamp(
    safeRows - 11,
    mode === "compact" ? 2 : 3,
    mode === "wide" ? 12 : 7,
  );

  return {
    mode,
    columns: safeColumns,
    rows: safeRows,
    paddingX,
    sidebarWidth,
    outputRows,
    runnerRows,
    serviceRows,
    showProjectPath: safeColumns >= 64 && safeRows >= 18,
  };
}
