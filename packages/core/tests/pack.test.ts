import { mkdtemp, mkdir, writeFile, rm, readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vitest";
import {
  applyPack,
  installPack,
  listPacks,
  loadPack,
  validatePack,
} from "../src/pack/mod.js";

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

describe("official packs", () => {
  it("lists all starter packs", async () => {
    const result = await listPacks({ packsRoot, skillsRoot });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    const names = result.value.map((p) => p.name).sort();
    expect(names).toEqual([
      "api-service",
      "cli-tool",
      "data-ml",
      "essentials",
      "full-stack",
      "library",
      "web-app",
    ]);
  });

  it("loads and validates essentials with all skills", async () => {
    const result = await validatePack("essentials", { packsRoot, skillsRoot });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    // base + pr-ready
    expect(result.value.pack.skillNames).toHaveLength(6);
    expect(result.value.skills).toHaveLength(6);
  });

  it("merges extends so dependency skills install with stack packs", async () => {
    const web = await loadPack("web-app", { packsRoot, skillsRoot });
    const lib = await loadPack("library", { packsRoot, skillsRoot });
    const cli = await loadPack("cli-tool", { packsRoot, skillsRoot });
    expect(web.ok).toBe(true);
    expect(lib.ok).toBe(true);
    expect(cli.ok).toBe(true);
    if (!web.ok || !lib.ok || !cli.ok) return;
    // essentials(6) + extras (web:3, lib:3, cli:2) with de-dupe of pr-ready
    expect(web.value.pack.extends).toEqual(["essentials"]);
    expect(web.value.skills.map((s) => s.name)).toContain("project-setup");
    expect(web.value.skills.map((s) => s.name)).toContain("a11y-pass");
    expect(web.value.skills.length).toBe(8);
    expect(lib.value.skills.length).toBe(8);
    expect(cli.value.skills.length).toBe(7);
  });
});

describe("install and apply pack", () => {
  it("installs a pack into a temp library", async () => {
    const kitHome = await tempDir("kit-pack-home-");
    const result = await installPack("essentials", {
      packsRoot,
      skillsRoot,
      kitHome,
      force: true,
    });

    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.installed.length).toBe(6);

    const index = await readFile(
      path.join(kitHome, "library.json"),
      "utf8",
    );
    expect(index).toContain("add-readme");
    expect(index).toContain("fix-bug");

    const packsIndex = await readFile(
      path.join(kitHome, "installed-packs.json"),
      "utf8",
    );
    expect(packsIndex).toContain("essentials");
  });

  it("applies a pack into a project directory", async () => {
    const kitHome = await tempDir("kit-pack-home-");
    const projectDir = await tempDir("kit-pack-proj-");

    const result = await applyPack("library", {
      packsRoot,
      skillsRoot,
      kitHome,
      projectDir,
      force: true,
    });

    expect(result.ok).toBe(true);
    if (!result.ok) return;

    const applied = await readFile(result.value.appliedPath, "utf8");
    expect(applied).toContain("library");

    const skillMd = await readFile(
      path.join(result.value.projectSkillsDir, "api-docs", "SKILL.md"),
      "utf8",
    );
    expect(skillMd).toContain("name: api-docs");
  });

  it("fails when a pack skill is missing", async () => {
    const root = await tempDir("kit-pack-bad-");
    const packDir = path.join(root, "packs", "broken");
    await mkdir(packDir, { recursive: true });
    await writeFile(
      path.join(packDir, "PACK.md"),
      `---
name: broken
title: Broken
description: Missing skill on purpose.
version: 0.1.0
skills:
  - does-not-exist
---

# Broken
`,
      "utf8",
    );

    const result = await loadPack("broken", {
      packsRoot: path.join(root, "packs"),
      skillsRoot: path.join(root, "skills"),
    });
    expect(result.ok).toBe(false);
  });
});
