import { mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vitest";
import { runDoctor } from "../src/doctor/mod.js";
import { testAllPacks, testPack, testSkill } from "../src/test/mod.js";

const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const packsRoot = path.join(repoRoot, "packs");
const skillsRoot = path.join(repoRoot, "skills");

const tempDirs: string[] = [];

async function tempDir(prefix: string): Promise<string> {
  const dir = await mkdtemp(path.join(os.tmpdir(), prefix));
  tempDirs.push(dir);
  return dir;
}

afterEach(async () => {
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) await rm(dir, { recursive: true, force: true });
  }
});

describe("testSkill / testPack", () => {
  it("passes official add-readme skill", async () => {
    const result = await testSkill(path.join(skillsRoot, "add-readme"));
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.skillName).toBe("add-readme");
  });

  it("fails empty body skill", async () => {
    const dir = await tempDir("kit-test-empty-");
    await writeFile(
      path.join(dir, "SKILL.md"),
      `---
name: empty-body
description: Body missing on purpose.
version: 0.1.0
compatibility:
  - codex
---
`,
      "utf8",
    );
    const result = await testSkill(dir);
    expect(result.ok).toBe(false);
  });

  it("passes essentials pack", async () => {
    const result = await testPack("essentials", { packsRoot, skillsRoot });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.skillReports.length).toBe(6);
  });

  it("passes all official packs", async () => {
    const result = await testAllPacks({ packsRoot, skillsRoot });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.passed).toBeGreaterThanOrEqual(3);
    expect(result.value.failed).toBe(0);
  });
});

describe("runDoctor", () => {
  it("returns a structured health report from the repo", async () => {
    const kitHome = await tempDir("kit-doc-home-");
    const report = await runDoctor({
      kitHome,
      projectDir: repoRoot,
    });

    expect(report.kitHome).toBe(kitHome);
    expect(report.checks.length).toBeGreaterThan(5);
    expect(report.summary.failed).toBe(0);
    expect(report.ok).toBe(true);

    const packs = report.checks.find((c) => c.id === "packs-root");
    expect(packs?.level).toBe("pass");

    const assets = report.checks.find((c) => c.id === "assets");
    expect(assets?.level).toBe("pass");
  });
});
