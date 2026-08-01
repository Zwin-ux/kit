import { HitMap } from "../mouse/HitMap.js";
import type { WorkbenchGeometry } from "./workbenchGeometry.js";
import { TERMINAL_LANES, type TerminalLane } from "./Workbench.js";

/**
 * Build click zones from the same geometry the UI renders.
 * List hit indices are ABSOLUTE (offset + visible), not window-local.
 */
export function fillWorkbenchHits(
  map: HitMap,
  options: {
    geo: WorkbenchGeometry;
    lane: TerminalLane;
    /** Absolute item count in the active list */
    itemCount: number;
    /** Window offset into the list */
    listOffset: number;
    /** Visible row count */
    visibleCount: number;
  },
): void {
  const { geo, lane } = options;
  const chipW = Math.max(
    12,
    Math.floor((geo.columns - geo.paddingX * 2) / 4),
  );

  // Lane tabs — full width chips on tabRow
  map.addChipRow({
    idPrefix: "lane",
    count: 4,
    row: geo.tabRow,
    col0: geo.paddingX + 1,
    chipWidth: chipW,
    actions: TERMINAL_LANES.map((l) => `lane:${l}`),
  });

  // List rows — store absolute index in data.index
  const idPrefix =
    lane === "skills"
      ? "term-pack"
      : lane === "agents"
        ? "term-runner"
        : lane === "services"
          ? "term-task"
          : "term-ops";

  const visible = Math.min(options.visibleCount, options.itemCount);
  for (let v = 0; v < visible; v++) {
    const absolute = options.listOffset + v;
    if (absolute >= options.itemCount) break;
    map.add({
      id: `${idPrefix}:${absolute}`,
      row0: geo.listStartRow + v,
      row1: geo.listStartRow + v,
      col0: geo.listCol0,
      col1: geo.listCol1,
      data: { index: absolute, action: "select" },
    });
  }

  // Action buttons — full-width thirds on actionRow
  const barW = Math.max(
    14,
    Math.floor((geo.columns - geo.paddingX * 2) / 3),
  );
  const barCol0 = geo.paddingX + 1;
  const actions: Array<{ id: string; action: string }> =
    lane === "skills"
      ? [
          { id: "btn-install", action: "install" },
          { id: "btn-apply", action: "apply" },
          { id: "btn-help", action: "help" },
        ]
      : lane === "agents"
        ? [
            { id: "btn-run", action: "run" },
            { id: "btn-pull", action: "ollama-pull" },
            { id: "btn-history", action: "history" },
          ]
        : lane === "services"
          ? [
              { id: "btn-run", action: "run-service" },
              { id: "btn-help", action: "help" },
            ]
          : [
              { id: "btn-run", action: "run-ops" },
              { id: "btn-help", action: "help" },
            ];

  actions.forEach((btn, i) => {
    map.addButton({
      id: btn.id,
      row: geo.actionRow,
      col0: barCol0 + i * barW,
      col1: barCol0 + (i + 1) * barW - 1,
      action: btn.action,
    });
  });
}
