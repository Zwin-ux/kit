import { describe, expect, it } from "vitest";
import {
  exploreListPacks,
  exploreSearch,
  exploreShowPack,
} from "../src/explore/mod.js";

const LIVE = process.env.KIT_EXPLORE_LIVE === "1";

describe("explore client (live registry)", () => {
  it.skipIf(!LIVE)("lists packs from production registry", async () => {
    const result = await exploreListPacks();
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.count).toBeGreaterThanOrEqual(3);
  });

  it.skipIf(!LIVE)("shows essentials pack", async () => {
    const result = await exploreShowPack("essentials");
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.pack.name).toBe("essentials");
    expect(result.value.skills.length).toBe(5);
  });

  it.skipIf(!LIVE)("searches readme", async () => {
    const result = await exploreSearch("readme");
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.count).toBeGreaterThan(0);
  });

  it("rejects empty search without network", async () => {
    const result = await exploreSearch("   ");
    expect(result.ok).toBe(false);
  });
});
