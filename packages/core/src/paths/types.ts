/** Supported agent harnesses for path mapping. */
export type HarnessId =
  | "kit"
  | "claude-code"
  | "codex"
  | "grok-build";

export type PathScope = "personal" | "project";

export interface HarnessSkillPath {
  harness: HarnessId;
  scope: PathScope;
  /** Absolute directory that should contain skill folders. */
  skillsRoot: string;
  /** Absolute path for one skill when name is known. */
  skillDir?: string;
  /** How Kit expects the harness to discover skills. */
  notes: string;
  /** True if the skills root exists on disk. */
  exists: boolean;
}

export interface PathReport {
  kitHome: string;
  projectDir: string;
  globalLibrary: string;
  projectLibrary: string;
  skillName?: string;
  entries: HarnessSkillPath[];
  installedSkillNames: string[];
}

export type LinkMode = "symlink" | "copy";

export interface LinkPlanItem {
  skillName: string;
  sourceDir: string;
  targetDir: string;
  harness: HarnessId;
  scope: PathScope;
  mode: LinkMode;
  action: "create" | "replace" | "skip-same" | "skip-exists";
  reason?: string;
}

export interface LinkResult {
  dryRun: boolean;
  items: LinkPlanItem[];
  linked: number;
  skipped: number;
  failed: Array<{ skillName: string; error: string }>;
}

export type PathsResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: string };
