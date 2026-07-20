import {
  access,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
  readFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { installSkill, listSkills } from "../src/library/library.js";
import {
  describePaths,
  importSkillsFromHarness,
  linkSkills,
  resolveHarnessSkillsRoot,
} from "../src/paths/mod.js";

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

describe("resolveHarnessSkillsRoot", () => {
  it("maps claude-code personal and project roots", () => {
    // Use absolute paths so path.resolve is OS-stable in CI (Linux + Windows).
    const home = path.resolve(os.tmpdir(), "kit-harness-home-claude");
    const project = path.resolve(os.tmpdir(), "kit-harness-proj-claude");
    const kitHome = path.join(home, ".kit");

    expect(
      resolveHarnessSkillsRoot("claude-code", "personal", {
        projectDir: project,
        kitHome,
        homeDir: home,
      }),
    ).toBe(path.join(home, ".claude", "skills"));

    expect(
      resolveHarnessSkillsRoot("claude-code", "project", {
        projectDir: project,
        kitHome,
        homeDir: home,
      }),
    ).toBe(path.join(project, ".claude", "skills"));
  });

  it("maps codex and grok-build roots", () => {
    const home = path.resolve(os.tmpdir(), "kit-harness-home-codex");
    const project = path.resolve(os.tmpdir(), "kit-harness-proj-codex");
    const kitHome = path.join(home, ".kit");

    expect(
      resolveHarnessSkillsRoot("codex", "personal", {
        projectDir: project,
        kitHome,
        homeDir: home,
      }),
    ).toBe(path.join(home, ".codex", "skills"));

    expect(
      resolveHarnessSkillsRoot("grok-build", "project", {
        projectDir: project,
        kitHome,
        homeDir: home,
      }),
    ).toBe(path.join(project, ".grok", "skills"));
  });
});

describe("describePaths + linkSkills", () => {
  it("describes kit and harness paths", async () => {
    const kitHome = await tempDir("kit-paths-home-");
    const projectDir = await tempDir("kit-paths-proj-");
    const homeDir = await tempDir("kit-paths-user-");

    const report = await describePaths({
      kitHome,
      projectDir,
      homeDir,
      skillName: "add-readme",
    });

    expect(report.ok).toBe(true);
    if (!report.ok) return;
    expect(report.value.entries.length).toBeGreaterThanOrEqual(8);
    const claude = report.value.entries.find(
      (e) => e.harness === "claude-code" && e.scope === "project",
    );
    expect(claude?.skillsRoot).toBe(
      path.join(projectDir, ".claude", "skills"),
    );
    expect(claude?.skillDir).toBe(
      path.join(projectDir, ".claude", "skills", "add-readme"),
    );
  });

  it("dry-run link plans without writing", async () => {
    const kitHome = await tempDir("kit-link-home-");
    const projectDir = await tempDir("kit-link-proj-");
    const homeDir = await tempDir("kit-link-user-");
    const skillSrc = await tempDir("kit-link-src-");

    await writeFile(
      path.join(skillSrc, "SKILL.md"),
      `---
name: path-demo
description: Demo skill for path linking.
version: 0.1.0
compatibility:
  - claude-code
  - grok-build
---

# Demo
`,
      "utf8",
    );

    const installed = await installSkill(skillSrc, {
      kitHome,
      force: true,
    });
    expect(installed.ok).toBe(true);

    const dry = await linkSkills({
      kitHome,
      projectDir,
      homeDir,
      harnesses: ["claude-code"],
      scope: "project",
      write: false,
    });

    expect(dry.ok).toBe(true);
    if (!dry.ok) return;
    expect(dry.value.dryRun).toBe(true);
    expect(dry.value.items.length).toBe(1);
    expect(dry.value.items[0]?.action).toBe("create");
    expect(dry.value.items[0]?.targetDir).toBe(
      path.join(projectDir, ".claude", "skills", "path-demo"),
    );

    // nothing written yet
    await expect(
      access(path.join(projectDir, ".claude", "skills", "path-demo")),
    ).rejects.toBeTruthy();
  });

  it("writes links with --write using copy mode", async () => {
    const kitHome = await tempDir("kit-linkw-home-");
    const projectDir = await tempDir("kit-linkw-proj-");
    const homeDir = await tempDir("kit-linkw-user-");
    const skillSrc = await tempDir("kit-linkw-src-");

    await writeFile(
      path.join(skillSrc, "SKILL.md"),
      `---
name: path-write
description: Demo skill for write linking.
version: 0.1.0
compatibility:
  - codex
---

# Demo
`,
      "utf8",
    );

    await installSkill(skillSrc, { kitHome, force: true });

    const result = await linkSkills({
      kitHome,
      projectDir,
      homeDir,
      harnesses: ["codex"],
      scope: "project",
      mode: "copy",
      write: true,
    });

    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.dryRun).toBe(false);
    expect(result.value.linked).toBe(1);

    const skillMd = await readFile(
      path.join(projectDir, ".codex", "skills", "path-write", "SKILL.md"),
      "utf8",
    );
    expect(skillMd).toContain("name: path-write");
  });
});

describe("importSkillsFromHarness", () => {
  it("dry-runs and installs valid harness skills into Kit library", async () => {
    const kitHome = await tempDir("kit-import-home-");
    const homeDir = await tempDir("kit-import-user-");
    const projectDir = await tempDir("kit-import-proj-");
    const skillDir = path.join(homeDir, ".claude", "skills", "capture-me");
    await mkdir(skillDir, { recursive: true });
    await writeFile(
      path.join(skillDir, "SKILL.md"),
      `---
name: capture-me
description: Demo skill captured from Claude harness.
version: 0.1.0
compatibility:
  - claude-code
  - codex
---

# Capture me

1. Do the thing.
2. Report done.
`,
      "utf8",
    );

    const dry = await importSkillsFromHarness({
      kitHome,
      homeDir,
      projectDir,
      harnesses: ["claude-code"],
      scope: "personal",
      write: false,
    });
    expect(dry.ok).toBe(true);
    if (!dry.ok) return;
    expect(dry.value.dryRun).toBe(true);
    expect(dry.value.imported).toBe(1);
    expect(dry.value.items.some((i) => i.skillName === "capture-me")).toBe(
      true,
    );

    const listedBefore = await listSkills({ kitHome });
    expect(listedBefore.ok).toBe(true);
    if (listedBefore.ok) {
      expect(listedBefore.value.find((s) => s.name === "capture-me")).toBe(
        undefined,
      );
    }

    const written = await importSkillsFromHarness({
      kitHome,
      homeDir,
      projectDir,
      harnesses: ["claude-code"],
      scope: "personal",
      write: true,
    });
    expect(written.ok).toBe(true);
    if (!written.ok) return;
    expect(written.value.dryRun).toBe(false);
    expect(written.value.imported).toBe(1);

    const listed = await listSkills({ kitHome });
    expect(listed.ok).toBe(true);
    if (!listed.ok) return;
    expect(listed.value.some((s) => s.name === "capture-me")).toBe(true);
  });
});
