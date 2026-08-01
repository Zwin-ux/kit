import { describe, expect, it } from "vitest";
import { workbenchGeometry, windowOffset } from "../src/screens/workbenchGeometry.js";
import { workbenchLayoutFromTerminal } from "../src/screens/workbenchLayout.js";

describe("Workbench geometry", () => {
  it.each([
    [60, 18, "compact"],
    [80, 24, "standard"],
    [100, 28, "standard"],
    [120, 32, "wide"],
    [180, 55, "wide"],
  ] as const)("maps %ix%i to %s", (columns, rows, mode) => {
    const geo = workbenchGeometry(columns, rows);
    expect(geo.mode).toBe(mode);
    expect(geo.listRows).toBeGreaterThanOrEqual(3);
    expect(geo.listStartRow).toBe(6);
    expect(geo.tabRow).toBe(2);
    expect(geo.actionRow).toBeGreaterThan(geo.listStartRow);
  });

  it("keeps list and action rows inside the terminal", () => {
    const geo = workbenchGeometry(80, 24);
    expect(geo.listStartRow + geo.listRows).toBeLessThanOrEqual(geo.actionRow);
    expect(geo.actionRow).toBeLessThanOrEqual(geo.rows);
  });

  it("windowOffset tracks selection", () => {
    expect(windowOffset(10, 0, 4)).toBe(0);
    expect(windowOffset(10, 9, 4)).toBe(6);
  });

  it("legacy layout wrapper still works", () => {
    const layout = workbenchLayoutFromTerminal(80, 24);
    expect(layout.listRows).toBeGreaterThan(0);
    expect(layout.mode).toBe("standard");
  });
});
