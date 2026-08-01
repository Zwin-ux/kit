import { describe, expect, it } from "vitest";
import {
  MENU_ICON_IDS,
  getMenuIconBitmap,
  menuIconGlyph,
  laneMenuIcon,
  opsMenuIcon,
  runnerMenuIcon,
  MENU_ICON_SIZE,
} from "../src/mascot/menuIcons.js";
import { HitMap } from "../src/mouse/HitMap.js";
import { fillWorkbenchHits } from "../src/screens/workbenchHits.js";
import { workbenchGeometry } from "../src/screens/workbenchGeometry.js";

describe("menu icons (game assets)", () => {
  it("defines one bitmap asset per menu id", () => {
    for (const id of MENU_ICON_IDS) {
      const bmp = getMenuIconBitmap(id);
      expect(bmp).toHaveLength(MENU_ICON_SIZE * MENU_ICON_SIZE);
      expect(menuIconGlyph(id).length).toBeGreaterThan(0);
    }
  });

  it("maps lanes, runners, and ops to icons", () => {
    expect(laneMenuIcon("skills")).toBe("skills");
    expect(runnerMenuIcon("ollama")).toBe("ollama");
    expect(runnerMenuIcon("codex")).toBe("codex");
    expect(opsMenuIcon("ready")).toBe("ready");
  });
});

describe("workbench hit boxes", () => {
  it("registers absolute pack indices on list rows", () => {
    const map = new HitMap();
    const geo = workbenchGeometry(100, 30);
    fillWorkbenchHits(map, {
      geo,
      lane: "skills",
      itemCount: 7,
      listOffset: 2,
      visibleCount: 5,
    });
    // First visible row is absolute index 2
    const first = map.hit(geo.listCol0 + 2, geo.listStartRow);
    expect(first?.data?.index).toBe(2);
    expect(first?.id).toBe("term-pack:2");

    // Tab row
    const lane = map.hit(geo.paddingX + 2, geo.tabRow);
    expect(lane?.data?.action).toMatch(/^lane:/);

    // Action bar install
    let foundInstall = false;
    for (let x = 1; x < 100; x++) {
      const h = map.hit(x, geo.actionRow);
      if (h?.data?.action === "install") foundInstall = true;
    }
    expect(foundInstall).toBe(true);
  });
});
