import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { installSkill } from "../src/library/library.js";
import { runStatus } from "../src/product/status.js";
import { fileURLToPath } from "node:url";

const tempDirs: string[] = [];
const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../../..",
);
const skillFixture = path.join(
  repoRoot,
  "packages/core/tests/fixtures/valid-add-readme",
);

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

describe("runStatus", () => {
  it("reports no-root when harness folders missing", async () => {
    const kitHome = await tempDir("kit-status-home-");
    const projectDir = await tempDir("kit-status-proj-");
    await installSkill(skillFixture, { kitHome, force: true });

    const result = await runStatus({ kitHome, projectDir });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.value.libraryCount).toBe(1);
    const claude = result.value.rows.find(
      (r) => r.harness === "claude-code" && r.scope === "project",
    );
    expect(claude?.state).toBe("no-root");
    expect(result.value.allOk).toBe(false);
    expect(result.value.nextCommand).toContain("link");
  });

  it("reports ok when library skill exists under claude project root", async () => {
    const kitHome = await tempDir("kit-status-home2-");
    const projectDir = await tempDir("kit-status-proj2-");
    await installSkill(skillFixture, { kitHome, force: true });

    const claudeSkills = path.join(projectDir, ".claude", "skills", "add-readme");
    await mkdir(claudeSkills, { recursive: true });
    await writeFile(
      path.join(claudeSkills, "SKILL.md"),
      "---\nname: add-readme\n---\n",
      "utf8",
    );

    const result = await runStatus({
      kitHome,
      projectDir,
      harnesses: ["claude-code"],
    });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    const claude = result.value.rows.find((r) => r.harness === "claude-code");
    expect(claude?.state).toBe("ok");
    expect(claude?.linkedNames).toContain("add-readme");
  });

  it("counts symlinked skills — what kit link/ready actually create", async () => {
    const kitHome = await tempDir("kit-status-home3-");
    const projectDir = await tempDir("kit-status-proj3-");
    const installed = await installSkill(skillFixture, { kitHome, force: true });
    expect(installed.ok).toBe(true);
    if (!installed.ok) return;

    const claudeRoot = path.join(projectDir, ".claude", "skills");
    await mkdir(claudeRoot, { recursive: true });
    await symlink(
      installed.value.installPath,
      path.join(claudeRoot, "add-readme"),
      "junction",
    );

    const result = await runStatus({
      kitHome,
      projectDir,
      harnesses: ["claude-code"],
    });
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    const claude = result.value.rows.find((r) => r.harness === "claude-code");
    expect(claude?.state).toBe("ok");
    expect(claude?.linkedNames).toContain("add-readme");
    expect(result.value.allOk).toBe(true);
  });
});
