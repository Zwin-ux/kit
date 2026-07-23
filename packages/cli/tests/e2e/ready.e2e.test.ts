/**
 * E2E: kit ready / status / recommend / pack — the product pipeline —
 * against the compiled dist/bin.js in isolated sandboxes.
 */
import { readdir } from "node:fs/promises";
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
import { makeWebProject } from "./fixtures.js";

/** Skills pinned by packs/essentials/PACK.md — stable assertion targets. */
const ESSENTIALS_SKILL = "add-readme";

let realHome: RealHomeSnapshot;
const sandboxes: Sandbox[] = [];

async function sandbox(projectDirName?: string): Promise<Sandbox> {
  const sb = await createSandbox(
    projectDirName !== undefined ? { projectDirName } : {},
  );
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

describe("kit ready", () => {
  it("defaults to a dry-run that writes nothing", async () => {
    const sb = await sandbox();
    await makeWebProject(sb.projectDir);

    const result = await sb.runKit(["ready"]);
    expectExit(result, 0);
    expectOutput(result, "READY  (dry-run)");
    expectOutput(result, "Would install pack");

    // No project harness dirs, no project kit dir, no library skills.
    expect(await pathExists(path.join(sb.projectDir, ".kit"))).toBe(false);
    expect(await pathExists(path.join(sb.projectDir, ".claude"))).toBe(false);
    expect(await pathExists(path.join(sb.projectDir, ".codex"))).toBe(false);
    expect(await pathExists(path.join(sb.projectDir, ".grok"))).toBe(false);
    const librarySkills = await readdirSafe(path.join(sb.kitHome, "skills"));
    expect(librarySkills).toEqual([]);
  });

  it("--write wires library, project skills, and all three harnesses", async () => {
    const sb = await sandbox();
    await makeWebProject(sb.projectDir);

    const result = await sb.runKit([
      "ready",
      "--write",
      "--pack",
      "essentials",
    ]);
    expectExit(result, 0);
    expectOutput(result, "READY  ✓");

    // Library + config live inside the sandbox KIT_HOME.
    expect(
      await pathExists(path.join(sb.kitHome, "skills", ESSENTIALS_SKILL)),
    ).toBe(true);
    expect(await pathExists(path.join(sb.kitHome, "config.json"))).toBe(true);

    // Project copies + one link per harness.
    expect(
      await pathExists(
        path.join(sb.projectDir, ".kit", "skills", ESSENTIALS_SKILL, "SKILL.md"),
      ),
    ).toBe(true);
    for (const harnessDir of [".claude", ".codex", ".grok"]) {
      expect(
        await pathExists(
          path.join(
            sb.projectDir,
            harnessDir,
            "skills",
            ESSENTIALS_SKILL,
            "SKILL.md",
          ),
        ),
        `expected ${harnessDir}/skills/${ESSENTIALS_SKILL} after ready --write\n${result.describe()}`,
      ).toBe(true);
    }

    // Second run is a no-op for links: everything already wired.
    const rerun = await sb.runKit(["ready", "--write", "--pack", "essentials"]);
    expectExit(rerun, 0);
    expectOutput(rerun, "READY  ✓");
    expect(rerun.stdout).toMatch(/Linked 0 \(skipped \d+\)/);
  });

  it("refuses --write into the (sandbox) home dir without --force", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["ready", "--write"], { cwd: sb.home });
    expectExit(result, 1);
    expectOutput(result, "Refusing to write kit state");

    // Guard fired before any writes into the fake home.
    expect(await pathExists(path.join(sb.home, ".claude"))).toBe(false);
    expect(await pathExists(path.join(sb.home, ".kit", "skills", ESSENTIALS_SKILL))).toBe(
      false,
    );
  });

  it("handles project paths with spaces and unicode", async () => {
    const sb = await sandbox("prøjéct 目录 space");
    await makeWebProject(sb.projectDir);

    const result = await sb.runKit([
      "ready",
      "--write",
      "--pack",
      "essentials",
    ]);
    expectExit(result, 0);
    expectOutput(result, "READY  ✓");

    for (const harnessDir of [".claude", ".codex", ".grok"]) {
      expect(
        await pathExists(
          path.join(
            sb.projectDir,
            harnessDir,
            "skills",
            ESSENTIALS_SKILL,
            "SKILL.md",
          ),
        ),
        `harness link missing under unicode path\n${result.describe()}`,
      ).toBe(true);
    }
  });
});

describe("kit status", () => {
  it("exits 0 on a fresh sandbox with an empty library", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["status"]);
    expectExit(result, 0);
    expectOutput(result, "STATUS");
  });

  it("reports allOk via --json after ready --write", async () => {
    const sb = await sandbox();
    await makeWebProject(sb.projectDir);
    expectExit(
      await sb.runKit(["ready", "--write", "--pack", "essentials"]),
      0,
    );

    const result = await sb.runKit(["status", "--json"]);
    expectExit(result, 0);

    const report = JSON.parse(result.stdout) as {
      allOk: boolean;
      libraryCount: number;
      libraryNames: string[];
      rows: Array<{ harness: string; scope: string; state: string }>;
    };
    expect(report.allOk).toBe(true);
    expect(report.libraryCount).toBeGreaterThanOrEqual(6);
    expect(report.libraryNames).toContain(ESSENTIALS_SKILL);
    expect(report.rows).toHaveLength(3);
    for (const row of report.rows) {
      expect(row.scope).toBe("project");
      expect(row.state).toBe("ok");
    }
  });
});

describe("kit recommend", () => {
  it("recommends the web-app pack for a react project", async () => {
    const sb = await sandbox();
    await makeWebProject(sb.projectDir);

    const result = await sb.runKit(["recommend"]);
    expectExit(result, 0);
    expectOutput(result, "Top pick: web-app");
  });
});

describe("kit pack", () => {
  it("lists the official packs offline", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["pack", "list"]);
    expectExit(result, 0);
    expectOutput(result, "essentials@");
    expectOutput(result, "web-app@");
  });

  it("validates the essentials pack", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["pack", "validate", "essentials"]);
    expectExit(result, 0);
    expectOutput(result, "OK  pack essentials@");
  });

  it("applies essentials into the project and syncs the library", async () => {
    const sb = await sandbox();
    await makeWebProject(sb.projectDir);

    const result = await sb.runKit([
      "pack",
      "apply",
      "essentials",
      "--dir",
      sb.projectDir,
    ]);
    expectExit(result, 0);
    expectOutput(result, "pack essentials@");

    const projectSkills = await readdirSafe(
      path.join(sb.projectDir, ".kit", "skills"),
    );
    expect(projectSkills).toContain(ESSENTIALS_SKILL);
    expect(
      await pathExists(path.join(sb.kitHome, "skills", ESSENTIALS_SKILL)),
    ).toBe(true);

    // The applied record stays inside the project.
    const kitDirEntries = await readdirSafe(path.join(sb.projectDir, ".kit"));
    expect(kitDirEntries.length).toBeGreaterThan(0);
  });

  it("fails cleanly for an unknown pack", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["pack", "validate", "no-such-pack"]);
    expectExit(result, 1);
  });
});

async function readdirSafe(dir: string): Promise<string[]> {
  try {
    return (await readdir(dir)).sort();
  } catch {
    return [];
  }
}
