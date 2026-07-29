import { describe, expect, it } from "vitest";

import { workbenchLayoutFromTerminal } from "../src/screens/workbenchLayout.js";

describe("Workbench terminal layout", () => {
  it.each([
    [60, 18, "compact"],
    [80, 24, "standard"],
    [100, 28, "standard"],
    [120, 32, "wide"],
    [180, 55, "wide"],
  ] as const)("maps %ix%i to %s", (columns, rows, mode) => {
    const layout = workbenchLayoutFromTerminal(columns, rows);
    expect(layout.mode).toBe(mode);
    expect(layout.outputRows).toBeGreaterThanOrEqual(3);
    expect(layout.runnerRows).toBeGreaterThanOrEqual(1);
    expect(layout.serviceRows).toBeGreaterThanOrEqual(2);
    expect(layout.sidebarWidth).toBeLessThan(columns / 2);
  });

  it("gives live output more rows when the terminal grows", () => {
    const small = workbenchLayoutFromTerminal(80, 24);
    const full = workbenchLayoutFromTerminal(180, 55);
    expect(full.outputRows).toBeGreaterThan(small.outputRows);
    expect(full.serviceRows).toBeGreaterThan(small.serviceRows);
  });

  it("uses fewer runner rows when compact controls need two lines", () => {
    const compact = workbenchLayoutFromTerminal(60, 18);
    expect(compact.runnerRows).toBe(2);
  });
});
