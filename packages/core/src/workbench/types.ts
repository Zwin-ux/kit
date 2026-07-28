import type { LibraryResult } from "../library/types.js";

export type CodingRunnerId =
  | "codex"
  | "claude-code"
  | "grok-build"
  | "ollama";
export type CodingJobMode = "inspect" | "build";

export interface LocalModel {
  name: string;
  size: number;
  parameterSize?: string;
  quantization?: string;
}

export interface RunnerCommand {
  executable: string;
  prefixArgs: string[];
}

export interface CodingRunnerStatus {
  id: CodingRunnerId;
  label: string;
  available: boolean;
  executable: string | null;
  detail?: string;
  models?: LocalModel[];
}

export interface CodingJobRequest {
  runner: CodingRunnerId;
  mode: CodingJobMode;
  projectDir: string;
  prompt: string;
  model?: string;
  confirmBuild?: boolean;
}

export interface CodingInvocation {
  runner: CodingRunnerId;
  mode: CodingJobMode;
  executable: string;
  args: string[];
  cwd: string;
  model?: string;
}

export interface CodingJobReport extends CodingInvocation {
  exitCode: number;
  stdout: string;
  stderr: string;
  durationMs: number;
  timedOut: boolean;
  cancelled: boolean;
  truncated: boolean;
}

export interface OllamaDiscoveryOptions {
  baseUrl?: string;
  timeoutMs?: number;
  fetchImpl?: typeof fetch;
}

export interface RunCodingJobOptions {
  timeoutMs?: number;
  maxOutputBytes?: number;
  signal?: AbortSignal;
  onOutput?: (chunk: string, stream: "stdout" | "stderr") => void;
  /** Used by tests and embedders that supply a trusted runner shim. */
  commandOverride?: RunnerCommand;
}

export type WorkbenchResult<T> = LibraryResult<T>;
