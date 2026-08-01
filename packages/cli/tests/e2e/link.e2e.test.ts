/**
 * E2E: kit link + kit import (inverse operations over harness skill dirs),
 * against the compiled dist/bin.js in isolated sandboxes.
 */
import { mkdir, readFile, symlink, writeFile } from "node:fs/promises";
import path from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  assertRealHomeUntouched,
  createSandbox,
  expectExit,
  expectOutput,
  pathExists,
  snapshotRealHome,
  type RealHomeSnapshot,
  type Sandbox,
} from "./harness.js";
import { seedLibrary, seedSkillDir } from "./fixtures.js";

let realHome: RealHomeSnapshot;
const sandboxes: Sandbox[] = [];

async function sandbox(): Promise<Sandbox> {
  const sb = await createSandbox();
  sandboxes.push(sb);
  return sb;
}

beforeAll(async () => {
  realHome = await snapshotRealHome();
});

afterAll(async () => {
  for (const sb of sandboxes) await sb.cleanup();
  await assertRealHomeUntouched(realHome);
});

describe("kit link", () => {
  it("fails with guidance when the library is empty", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["link", "--to", "all", "--write"]);
    expectExit(result, 1);
    expectOutput(result, "No skills to link");
  });

  it("defaults to dry-run, then --write links into all three harnesses, then skips as same", async () => {
    const sb = await sandbox();
    await seedLibrary(sb, ["alpha-skill", "beta-skill"]);

    // 1. Dry-run plans 6 links (2 skills x 3 harnesses) but writes nothing.
    const dry = await sb.runKit(["link", "--to", "all"]);
    expectExit(dry, 0);
    expectOutput(dry, "Link plan (dry-run)");
    expectOutput(dry, "No files written");
    expect(dry.stdout).toMatch(/linked: 6\s+skipped: 0\s+failed: 0/);
    expect(await pathExists(path.join(sb.projectDir, ".claude"))).toBe(false);

    // 2. --write creates them.
    const write = await sb.runKit(["link", "--to", "all", "--write"]);
    expectExit(write, 0);
    expectOutput(write, "Link result");
    expect(write.stdout).toMatch(/linked: 6\s+skipped: 0\s+failed: 0/);
    for (const harnessDir of [".claude", ".codex", ".grok"]) {
      for (const skill of ["alpha-skill", "beta-skill"]) {
        expect(
          await pathExists(
            path.join(sb.projectDir, harnessDir, "skills", skill, "SKILL.md"),
          ),
          `expected ${harnessDir}/skills/${skill}\n${write.describe()}`,
        ).toBe(true);
      }
    }

    // 3. Idempotent: a second --write run skips every existing correct link.
    const rerun = await sb.runKit(["link", "--to", "all", "--write"]);
    expectExit(rerun, 0);
    expect(rerun.stdout).toMatch(/linked: 0\s+skipped: 6\s+failed: 0/);
    expectOutput(rerun, "already linked to Kit source");
  });

  it("reports a conflict without --force and replaces with --force", async () => {
    const sb = await sandbox();
    await seedLibrary(sb, ["alpha-skill"]);

    // A real directory sits where the link would go.
    const conflictDir = path.join(
      sb.projectDir,
      ".claude",
      "skills",
      "alpha-skill",
    );
    await mkdir(conflictDir, { recursive: true });
    await writeFile(
      path.join(conflictDir, "SKILL.md"),
      "# not managed by kit\n",
      "utf8",
    );

    // Without --force: reported, untouched, exit 0.
    const blocked = await sb.runKit(["link", "--to", "claude-code", "--write"]);
    expectExit(blocked, 0);
    expectOutput(blocked, "target exists (use --force to replace)");
    expect(await readFile(path.join(conflictDir, "SKILL.md"), "utf8")).toBe(
      "# not managed by kit\n",
    );

    // With --force: replaced by the library version.
    const forced = await sb.runKit([
      "link",
      "--to",
      "claude-code",
      "--write",
      "--force",
    ]);
    expectExit(forced, 0);
    expect(forced.stdout).toMatch(/linked: 1\s+skipped: 0\s+failed: 0/);
    const replaced = await readFile(path.join(conflictDir, "SKILL.md"), "utf8");
    expect(replaced).toContain("alpha-skill");
    expect(replaced).not.toContain("not managed by kit");
  });

  it("repairs or reports a broken link, never crashes", async () => {
    const sb = await sandbox();
    await seedLibrary(sb, ["alpha-skill"]);

    const targetRoot = path.join(sb.projectDir, ".claude", "skills");
    const target = path.join(targetRoot, "alpha-skill");
    await mkdir(targetRoot, { recursive: true });
    // Dangling link: points at a path that does not exist.
    await symlink(path.join(sb.root, "gone"), target, "junction");

    const result = await sb.runKit([
      "link",
      "--to",
      "claude-code",
      "--write",
      "--force",
    ]);

    // Contract: repaired (exit 0, target resolves to the real skill) or
    // reported as a failure (exit 1) — never an unexpected crash (exit 2).
    // Windows sees dangling junctions as "existing" (access succeeds on the
    // reparse point) and replaces them; POSIX access() follows the dead link,
    // so the create hits EEXIST and is reported.
    expect([0, 1]).toContain(result.code);
    if (result.code === 0) {
      const content = await readFile(path.join(target, "SKILL.md"), "utf8");
      expect(content).toContain("alpha-skill");
    } else {
      expectOutput(result, "failure(s)");
      expectOutput(result, "alpha-skill");
    }
  });

  it("links into the sandbox personal harness dirs with --scope personal", async () => {
    const sb = await sandbox();
    await seedLibrary(sb, ["alpha-skill"]);

    const result = await sb.runKit([
      "link",
      "--to",
      "all",
      "--scope",
      "personal",
      "--write",
    ]);
    expectExit(result, 0);
    expect(result.stdout).toMatch(/linked: 3\s+skipped: 0\s+failed: 0/);

    for (const dir of [sb.claudeSkills, sb.codexSkills, sb.grokSkills]) {
      expect(
        await pathExists(path.join(dir, "alpha-skill", "SKILL.md")),
        `expected personal link under sandbox home: ${dir}\n${result.describe()}`,
      ).toBe(true);
    }
  });
});

describe("kit import", () => {
  it("reports a missing harness root without failing", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["import", "--from", "codex"]);
    expectExit(result, 0);
    expectOutput(result, "no skills root");
  });

  it("dry-runs by default, installs with --write, then skips existing", async () => {
    const sb = await sandbox();
    await seedSkillDir(sb.claudeSkills, "imported-skill");

    // 1. Dry run: planned but not installed.
    const dry = await sb.runKit(["import", "--from", "claude-code"]);
    expectExit(dry, 0);
    expectOutput(dry, "Import plan (dry-run)");
    expectOutput(dry, "imported-skill");
    const listBefore = await sb.runKit(["list"]);
    expectExit(listBefore, 0);
    expectOutput(listBefore, "No skills installed.");

    // 2. --write installs into the sandbox library.
    const write = await sb.runKit(["import", "--from", "claude-code", "--write"]);
    expectExit(write, 0);
    expectOutput(write, "Import result");
    expect(
      await pathExists(
        path.join(sb.kitHome, "skills", "imported-skill", "SKILL.md"),
      ),
    ).toBe(true);
    const listAfter = await sb.runKit(["list"]);
    expectExit(listAfter, 0);
    expectOutput(listAfter, "imported-skill@0.1.0");

    // 3. Idempotent: the same import is skipped without --force.
    const rerun = await sb.runKit(["import", "--from", "claude-code", "--write"]);
    expectExit(rerun, 0);
    expectOutput(rerun, "already in Kit library (use --force to replace)");
  });

  it("skips folders without a valid SKILL.md", async () => {
    const sb = await sandbox();
    const junkDir = path.join(sb.claudeSkills, "not-a-skill");
    await mkdir(junkDir, { recursive: true });
    await writeFile(path.join(junkDir, "notes.txt"), "junk\n", "utf8");

    const result = await sb.runKit(["import", "--from", "claude-code", "--write"]);
    expectExit(result, 0);
    expectOutput(result, "no SKILL.md");
    expect(
      await pathExists(path.join(sb.kitHome, "skills", "not-a-skill")),
    ).toBe(false);
  });
});
