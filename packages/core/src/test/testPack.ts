import { loadPack, listPacks, type PackLoadOptions } from "../pack/loadPack.js";
import { testSkill, type TestSkillOptions } from "./testSkill.js";
import type {
  CheckResult,
  MultiPackTestReport,
  PackTestReport,
  SkillTestReport,
  TestResult,
} from "./types.js";

export interface TestPackOptions extends PackLoadOptions, TestSkillOptions {}

/**
 * Test a pack: PACK.md + every resolved skill.
 */
export async function testPack(
  packDirOrName: string,
  options: TestPackOptions = {},
): Promise<TestResult<PackTestReport>> {
  const checks: CheckResult[] = [];
  const loaded = await loadPack(packDirOrName, options);

  if (!loaded.ok) {
    checks.push({
      id: "pack-load",
      level: "fail",
      message: loaded.error,
    });
    return {
      ok: false,
      error: `Pack test failed for ${packDirOrName}`,
      report: {
        target: packDirOrName,
        ok: false,
        skillReports: [],
        checks,
      },
    };
  }

  checks.push({
    id: "pack-load",
    level: "pass",
    message: `Pack OK (${loaded.value.pack.name}@${loaded.value.pack.version})`,
  });

  const skillReports: SkillTestReport[] = [];
  for (const entry of loaded.value.skills) {
    const result = await testSkill(entry.sourceDir, options);
    if (result.ok) {
      skillReports.push(result.value);
    } else if (result.report) {
      skillReports.push(result.report);
    } else {
      skillReports.push({
        target: entry.sourceDir,
        ok: false,
        skillName: entry.name,
        checks: [
          {
            id: "skill",
            level: "fail",
            message: result.error,
          },
        ],
        issues: [],
      });
    }
  }

  const failedSkills = skillReports.filter((r) => !r.ok).length;
  if (failedSkills === 0) {
    checks.push({
      id: "skills",
      level: "pass",
      message: `All ${skillReports.length} skills passed.`,
    });
  } else {
    checks.push({
      id: "skills",
      level: "fail",
      message: `${failedSkills} of ${skillReports.length} skills failed.`,
    });
  }

  const ok = failedSkills === 0;
  const report: PackTestReport = {
    target: loaded.value.pack.rootDir,
    ok,
    packName: loaded.value.pack.name,
    version: loaded.value.pack.version,
    skillReports,
    checks,
  };

  if (!ok) {
    return {
      ok: false,
      error: `Pack test failed for ${loaded.value.pack.name}`,
      report,
    };
  }

  return { ok: true, value: report };
}

/**
 * Test every pack under packs/.
 */
export async function testAllPacks(
  options: TestPackOptions = {},
): Promise<TestResult<MultiPackTestReport>> {
  const listed = await listPacks(options);
  if (!listed.ok) {
    return { ok: false, error: listed.error };
  }

  if (listed.value.length === 0) {
    return {
      ok: false,
      error: "No packs found. Set KIT_PACKS or run from the Kit repository.",
    };
  }

  const packs: PackTestReport[] = [];
  for (const item of listed.value) {
    const result = await testPack(item.rootDir, options);
    if (result.ok) {
      packs.push(result.value);
    } else if (result.report) {
      packs.push(result.report);
    }
  }

  const failed = packs.filter((p) => !p.ok).length;
  const passed = packs.length - failed;
  const report: MultiPackTestReport = {
    ok: failed === 0,
    packs,
    passed,
    failed,
  };

  if (!report.ok) {
    return {
      ok: false,
      error: `${failed} pack(s) failed testing.`,
      report,
    };
  }

  return { ok: true, value: report };
}
