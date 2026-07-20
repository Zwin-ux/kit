import type { NormalizedSkillMd } from "./normalize.js";
import type { UnifySourceHit } from "./unify.js";

export type SkillGrade = "S" | "A" | "B" | "C" | "D";

export interface ScoreInput {
  normalized: NormalizedSkillMd;
  sources: UnifySourceHit[];
  rawBody: string;
  /** True if original SKILL.md already had Kit-valid shape (front matter + fields). */
  wasKitShaped: boolean;
}

export interface ScoreResult {
  score: number;
  grade: SkillGrade;
  /** Default filter: bulk junk / empty stubs. */
  isNoise: boolean;
  noiseReasons: string[];
  /** Safe to adopt under default --write (S/A, not noise). */
  isKeeper: boolean;
  signals: string[];
}

/** Bulk vendor / installer dumps that dominate Codex skill folders. */
const NOISE_NAME_RE =
  /(?:^|-)(automation|mcp-server|mcp$)(?:-|$)|-automation$|^mcp-/i;

/**
 * Honest quality scoring for unify.
 * Multi-agent presence and real structure beat "we added compatibility".
 */
export function scoreUnifyCandidate(input: ScoreInput): ScoreResult {
  const { normalized, sources, rawBody, wasKitShaped } = input;
  const body = normalized.body.trim();
  const noiseReasons: string[] = [];
  const signals: string[] = [];
  let score = 0;

  const hasStructure =
    /\n\s*\d+\.\s+\S/.test(body) || /\n\s*[-*]\s+\S/.test(body);

  // --- noise classification ---
  if (NOISE_NAME_RE.test(normalized.name)) {
    noiseReasons.push("automation/mcp bulk name pattern");
  }
  // Thin stubs are noise; short but structured checklists are not.
  if (body.length < 60 && !hasStructure) {
    noiseReasons.push("body too thin");
  }
  if (/^#\s+\S+\s*$/m.test(body) && body.length < 100 && !hasStructure) {
    noiseReasons.push("stub body");
  }

  const isNoise = noiseReasons.length > 0;

  // --- positive signals ---
  const harnessSet = new Set(
    sources.map((s) => s.harness).filter((h) => h !== "kit" && h !== "project"),
  );
  // count distinct agent families
  const agentSources = sources.filter(
    (s) => s.harness === "claude-code" || s.harness === "codex" || s.harness === "grok-build",
  );
  const distinctAgents = new Set(agentSources.map((s) => s.harness));

  if (distinctAgents.size >= 2) {
    score += 28;
    signals.push(`multi-agent (${[...distinctAgents].join("+")})`);
  } else if (distinctAgents.size === 1) {
    score += 8;
    signals.push(`single-agent (${[...distinctAgents][0]})`);
  }

  if (sources.length >= 2) {
    score += Math.min(12, (sources.length - 1) * 4);
    signals.push(`${sources.length} copies`);
  }

  if (wasKitShaped) {
    score += 14;
    signals.push("kit-shaped source");
  }

  if (hasStructure) {
    score += 16;
    signals.push("structured steps");
  }

  if (/^#{1,3}\s+/m.test(body)) {
    score += 6;
    signals.push("headings");
  }

  if (body.length >= 200 && body.length <= 4000) {
    score += 12;
    signals.push("solid body length");
  } else if (body.length > 4000 && body.length < 12000) {
    score += 4;
  } else if (body.length >= 12000) {
    score -= 6;
  }

  const desc = normalized.description;
  if (desc.length >= 40 && desc.length <= 180) {
    score += 10;
    signals.push("clear description");
  } else if (desc.length >= 20) {
    score += 4;
  }

  // Normalize fixes are not free S-tier points
  if (normalized.fixes.length === 0) {
    score += 8;
    signals.push("clean front matter");
  } else if (normalized.fixes.length >= 4) {
    score -= 8;
  } else {
    score -= normalized.fixes.length; // small tax
  }

  if (isNoise) {
    score = Math.min(score, 35);
    score -= 20;
  }

  score = Math.max(0, Math.min(100, Math.round(score)));
  const grade = gradeFor(score, isNoise);
  const isKeeper = !isNoise && (grade === "S" || grade === "A");

  return {
    score,
    grade,
    isNoise,
    noiseReasons,
    isKeeper,
    signals,
  };
}

function gradeFor(score: number, isNoise: boolean): SkillGrade {
  if (isNoise || score < 40) return "D";
  if (score >= 85) return "S";
  if (score >= 70) return "A";
  if (score >= 55) return "B";
  return "C";
}

export function looksKitShaped(raw: string): boolean {
  if (!raw.trimStart().startsWith("---")) return false;
  return (
    /\nname:\s*\S+/i.test(raw) &&
    /\ndescription:\s*\S+/i.test(raw) &&
    /\ncompatibility:\s*\n/i.test(raw)
  );
}
