/**
 * Back-compat wrapper — prefer workbenchGeometry for new code.
 */
import {
  workbenchGeometry,
  type WorkbenchGeometry,
  type WorkbenchLayoutMode,
} from "./workbenchGeometry.js";

export type { WorkbenchLayoutMode };

/** @deprecated Use WorkbenchGeometry */
export interface WorkbenchLayout {
  mode: WorkbenchLayoutMode;
  columns: number;
  rows: number;
  paddingX: number;
  sidebarWidth: number;
  outputRows: number;
  runnerRows: number;
  serviceRows: number;
  listRows: number;
  showProjectPath: boolean;
  promptRows: number;
}

export function workbenchLayoutFromTerminal(
  columns: number,
  rows: number,
): WorkbenchLayout {
  const g = workbenchGeometry(columns, rows);
  return geometryToLegacy(g);
}

export function geometryToLegacy(g: WorkbenchGeometry): WorkbenchLayout {
  return {
    mode: g.mode,
    columns: g.columns,
    rows: g.rows,
    paddingX: g.paddingX,
    sidebarWidth: 0,
    outputRows: g.logRows,
    runnerRows: g.listRows,
    serviceRows: g.listRows,
    listRows: g.listRows,
    showProjectPath: g.showStatus,
    promptRows: 1,
  };
}
