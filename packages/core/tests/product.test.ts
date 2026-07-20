import { mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vitest";
import { listSkills } from "../src/library/library.js";
import { detectSituation } from "../src/product/situation.js";
import { pickStory } from "../src/product/stories.js";
import { runReady } from "../src/product/ready.js";

const tempDirs: string[] = [];
const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const packsRoot = path.join(repoRoot, "packs");
const skillsRoot = path.join(repoRoot, "skills");

async function tempDir(prefix: string): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

afterEach(async () => {
  while (tempDirs.length > 0) {
    const d = tempDirs.pop();
    if (d) await rm(d, { recursive: true, force: true });
  }
});

describe("pickStory", () => {
  it("routes chaos cleanup when harness pile is large and library thin", () => {
    const s = pickStory({
      libraryCount: 2,
      harnessSkillEstimate: 80,
      keeperEstimate: 8,
      noiseEstimate: 60,
      hasProjectSignals: true,
      recommendedPack: "web-app",
      recommendSummary: "looks like web",
      projectLinkedLikely: false,
    });
    expect(s.id).toBe("chaos-cleanup");
    expect(s.primary).toContain("unify");
  });

  it("routes empty-start for blank library", () => {
    const s = pickStory({
      libraryCount: 0,
      harnessSkillEstimate: 0,
      keeperEstimate: 0,
      noiseEstimate: 0,
      hasProjectSignals: false,
      recommendedPack: null,
      recommendSummary: null,
      projectLinkedLikely: false,
    });
    expect(s.id).toBe("empty-start");
  });
});

describe("detectSituation + runReady", () => {
  it("detects empty library and plans ready without writing", async () => {
    const kitHome = await tempDir("kit-sit-home-");
    const projectDir = await tempDir("kit-sit-proj-");
    await writeFile(
      path.join(projectDir, "package.json"),
      JSON.stringify({
        name: "demo-app",
        dependencies: { next: "14.0.0", react: "18.0.0" },
      }),
      "utf8",
    );

    const sit = await detectSituation({ kitHome, projectDir });
    expect(sit.snapshot.libraryCount).toBe(0);
    expect(sit.headline.length).toBeGreaterThan(10);

    const ready = await runReady({
      kitHome,
      projectDir,
      write: false,
      pack: "essentials",
      packsRoot,
      skillsRoot,
    });
    expect(ready.ok).toBe(true);
    if (!ready.ok) return;
    expect(ready.value.dryRun).toBe(true);
    expect(ready.value.packName).toBe("essentials");
    expect(ready.value.steps.some((s) => s.id === "pack-install")).toBe(true);
  });

  it("ready --write installs essentials into library", async () => {
    const kitHome = await tempDir("kit-ready-home-");
    const projectDir = await tempDir("kit-ready-proj-");
    const homeDir = await tempDir("kit-ready-user-");
    await writeFile(
      path.join(projectDir, "package.json"),
      JSON.stringify({ name: "x", private: true }),
      "utf8",
    );

    const result = await runReady({
      kitHome,
      projectDir,
      homeDir,
      write: true,
      pack: "essentials",
      packsRoot,
      skillsRoot,
    });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.dryRun).toBe(false);
    expect(result.value.steps.find((s) => s.id === "pack-install")?.status).toBe(
      "done",
    );
    expect(result.value.steps.find((s) => s.id === "link")?.status).toBe("done");

    const listed = await listSkills({ kitHome });
    expect(listed.ok).toBe(true);
    if (!listed.ok) return;
    expect(listed.value.length).toBeGreaterThanOrEqual(5);
  });
});
