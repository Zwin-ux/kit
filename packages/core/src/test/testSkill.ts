import { access, readdir } from "node:fs/promises";
import path from "node:path";
import { loadSkill } from "../loadSkill.js";
import type { CheckResult, SkillTestReport, TestResult } from "./types.js";

export interface TestSkillOptions {
  /** Require a non-empty markdown body. Default true. */
  requireBody?: boolean;
  /** Warn when tests/ is missing. Default true. */
  warnMissingTestsDir?: boolean;
}

/**
 * Test one skill folder: schema validation + basic quality checks.
 */
export async function testSkill(
  skillDir: string,
  options: TestSkillOptions = {},
): Promise<TestResult<SkillTestReport>> {
  const requireBody = options.requireBody !== false;
  const warnMissingTestsDir = options.warnMissingTestsDir !== false;
  const root = path.resolve(skillDir);
  const checks: CheckResult[] = [];

  const loaded = await loadSkill(root);
  if (!loaded.ok) {
    checks.push({
      id: "schema",
      level: "fail",
      message: "Skill failed schema validation.",
    });
    return {
      ok: false,
      error: `Skill test failed for ${root}`,
      report: {
        target: root,
        ok: false,
        checks,
        issues: loaded.issues,
      },
    };
  }

  checks.push({
    id: "schema",
    level: "pass",
    message: `Schema OK (${loaded.skill.name}@${loaded.skill.version})`,
  });

  const body = loaded.skill.body.trim();
  if (requireBody && !body) {
    checks.push({
      id: "body",
      level: "fail",
      message: "Skill body is empty. Add clear instructions after the front matter.",
    });
  } else if (body) {
    checks.push({
      id: "body",
      level: "pass",
      message: "Instruction body present.",
    });
  }

  if (loaded.skill.compatibility.length > 0) {
    checks.push({
      id: "compatibility",
      level: "pass",
      message: `Agents: ${loaded.skill.compatibility.join(", ")}`,
    });
  }

  const testsDir = path.join(root, "tests");
  if (await pathExists(testsDir)) {
    try {
      const entries = await readdir(testsDir);
      checks.push({
        id: "tests-dir",
        level: "info",
        message: `tests/ present (${entries.length} entr${entries.length === 1 ? "y" : "ies"}).`,
      });
    } catch {
      checks.push({
        id: "tests-dir",
        level: "warn",
        message: "tests/ exists but could not be read.",
      });
    }
  } else if (warnMissingTestsDir) {
    checks.push({
      id: "tests-dir",
      level: "warn",
      message: "No tests/ folder yet (optional for schema v0).",
    });
  }

  const failed = checks.some((c) => c.level === "fail");
  const report: SkillTestReport = {
    target: root,
    ok: !failed,
    skillName: loaded.skill.name,
    version: loaded.skill.version,
    checks,
    issues: [],
  };

  if (failed) {
    return {
      ok: false,
      error: `Skill test failed for ${loaded.skill.name}`,
      report,
    };
  }

  return { ok: true, value: report };
}

async function pathExists(target: string): Promise<boolean> {
  try {
    await access(target);
    return true;
  } catch {
    return false;
  }
}
