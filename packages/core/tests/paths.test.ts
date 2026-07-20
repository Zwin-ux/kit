import { access, mkdtemp, rm, writeFile, readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { installSkill } from "../src/library/library.js";
import {
  describePaths,
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
