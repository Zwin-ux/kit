import type { LibraryResult } from "../library/types.js";

export type CodingRunnerId = "codex" | "claude-code" | "grok-build";
export type CodingJobMode = "inspect" | "build";

export interface RunnerCommand {
  executable: string;
  prefixArgs: string[];
}

export interface CodingRunnerStatus {
  id: CodingRunnerId;
  label: string;
  available: boolean;
  executable: string | null;
}

export interface CodingJobRequest {
  runner: CodingRunnerId;
  mode: CodingJobMode;
  projectDir: string;
  prompt: string;
  confirmBuild?: boolean;
}

export interface CodingInvocation {
  runner: CodingRunnerId;
  mode: CodingJobMode;
  executable: string;
  args: string[];
  cwd: string;
}

export interface CodingJobReport extends CodingInvocation {
  exitCode: number;
  stdout: string;
  stderr: string;
  durationMs: number;
  timedOut: boolean;
  truncated: boolean;
}

export interface RunCodingJobOptions {
  timeoutMs?: number;
  maxOutputBytes?: number;
  /** Used by tests and embedders that supply a trusted runner shim. */
  commandOverride?: RunnerCommand;
}

export type WorkbenchResult<T> = LibraryResult<T>;

