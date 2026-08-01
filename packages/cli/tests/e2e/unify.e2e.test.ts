/**
 * E2E: kit unify — scan harness dumps, adopt keepers, filter noise —
 * against the compiled dist/bin.js in isolated sandboxes.
 */
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
import { keeperBody, seedSkillDir } from "./fixtures.js";

interface UnifyJsonReport {
  dryRun: boolean;
  scanned: number;
  unique: number;
  noiseCount: number;
  keeperCount: number;
  adopted: number;
  linked: number;
  adoptedNames: string[];
  keepers: Array<{ name: string; grade: string; score: number }>;
}

let realHome: RealHomeSnapshot;
const sandboxes: Sandbox[] = [];

async function sandbox(): Promise<Sandbox> {
  const sb = await createSandbox();
  sandboxes.push(sb);
  return sb;
}

/**
 * The canonical mess: one good skill duplicated across two agents
 * (multi-agent keeper) plus one thin automation stub (noise).
 */
async function seedMess(sb: Sandbox): Promise<void> {
  const keeper = { body: keeperBody("review-pr") };
  await seedSkillDir(sb.claudeSkills, "review-pr", keeper);
  await seedSkillDir(sb.codexSkills, "review-pr", keeper);
  await seedSkillDir(sb.claudeSkills, "mcp-automation", { body: "TODO" });
}

beforeAll(async () => {
  realHome = await snapshotRealHome();
});

afterAll(async () => {
  for (const sb of sandboxes) await sb.cleanup();
  await assertRealHomeUntouched(realHome);
});

describe("kit unify", () => {
  it("dry-run ranks keepers, filters noise, and writes nothing", async () => {
    const sb = await sandbox();
    await seedMess(sb);

    const result = await sb.runKit(["unify"]);
    expectExit(result, 0);
    expectOutput(result, "UNIFY  skill OS  (dry-run)");
    expectOutput(result, "review-pr");

    // Nothing adopted on a dry-run.
    const list = await sb.runKit(["list"]);
    expectExit(list, 0);
    expectOutput(list, "No skills installed.");
  });

  it("--json emits a parseable report with exact counts", async () => {
    const sb = await sandbox();
    await seedMess(sb);

    const result = await sb.runKit(["unify", "--json"]);
    expectExit(result, 0);

    const report = JSON.parse(result.stdout) as UnifyJsonReport;
    expect(report.dryRun).toBe(true);
    expect(report.scanned).toBe(3); // review-pr x2 + mcp-automation
    expect(report.unique).toBeGreaterThanOrEqual(1);
    expect(report.noiseCount).toBe(1);
    expect(report.keeperCount).toBe(1);
    expect(report.keepers[0]?.name).toBe("review-pr");
    expect(["S", "A"]).toContain(report.keepers[0]?.grade);
    expect(report.adopted).toBe(0);
  });

  it("--write adopts keepers only, then a rerun adopts nothing new", async () => {
    const sb = await sandbox();
    await seedMess(sb);

    const write = await sb.runKit(["unify", "--write", "--json"]);
    expectExit(write, 0);
    const report = JSON.parse(write.stdout) as UnifyJsonReport;
    expect(report.dryRun).toBe(false);
    expect(report.adopted).toBe(1);
    expect(report.adoptedNames).toEqual(["review-pr"]);

    // Keeper is in the sandbox library; noise never is.
    expect(
      await pathExists(path.join(sb.kitHome, "skills", "review-pr", "SKILL.md")),
    ).toBe(true);
    expect(
      await pathExists(path.join(sb.kitHome, "skills", "mcp-automation")),
    ).toBe(false);

    // Idempotent: rerun adopts nothing.
    const rerun = await sb.runKit(["unify", "--write", "--json"]);
    expectExit(rerun, 0);
    const rerunReport = JSON.parse(rerun.stdout) as UnifyJsonReport;
    expect(rerunReport.adopted).toBe(0);
  });

  it("--write --link also wires keepers into the project harnesses", async () => {
    const sb = await sandbox();
    await seedMess(sb);

    const result = await sb.runKit(["unify", "--write", "--link", "--json"]);
    expectExit(result, 0);
    const report = JSON.parse(result.stdout) as UnifyJsonReport;
    expect(report.adopted).toBe(1);
    expect(report.linked).toBe(3); // 1 keeper x 3 harnesses

    for (const harnessDir of [".claude", ".codex", ".grok"]) {
      expect(
        await pathExists(
          path.join(sb.projectDir, harnessDir, "skills", "review-pr", "SKILL.md"),
        ),
        `expected project link in ${harnessDir}\n${result.describe()}`,
      ).toBe(true);
    }
  });

  it("--link without --write is refused", async () => {
    const sb = await sandbox();
    const result = await sb.runKit(["unify", "--link"]);
    expectExit(result, 1);
    expectOutput(result, "requires --write");
  });
});
