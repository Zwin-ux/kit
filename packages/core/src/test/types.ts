import type { ValidationIssue } from "../types.js";

export type CheckLevel = "pass" | "fail" | "warn" | "info";

export interface CheckResult {
  id: string;
  level: CheckLevel;
  message: string;
  detail?: string;
}

export interface SkillTestReport {
  target: string;
  ok: boolean;
  skillName?: string;
  version?: string;
  checks: CheckResult[];
  issues: ValidationIssue[];
}

export interface PackTestReport {
  target: string;
  ok: boolean;
  packName?: string;
  version?: string;
  skillReports: SkillTestReport[];
  checks: CheckResult[];
}

export interface MultiPackTestReport {
  ok: boolean;
  packs: PackTestReport[];
  passed: number;
  failed: number;
}

export type TestResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: string; report?: T };
