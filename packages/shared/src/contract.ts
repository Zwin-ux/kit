/**
 * Stable machine-readable output contract for `kit <command> --json`.
 *
 * The envelope is versioned by KIT_SCHEMA_VERSION, independent of the
 * package version: bump it only on breaking shape changes. Currently
 * implemented by `kit doctor --json` and `kit ready --json`.
 * (`kit status --json` and `kit unify --json` predate the envelope and
 * keep their raw report shape for compatibility.)
 */

export const KIT_SCHEMA_VERSION = "1" as const;

/** Typed error codes for machine consumers of --json output. */
export const KIT_ERROR_CODES = {
  /** Bad flags/arguments. */
  USAGE: "E_USAGE",
  /** Refused to write into a protected directory (home/root/etc.). */
  GUARD_REFUSED: "E_GUARD_REFUSED",
  /** A pipeline step (install/apply/link/…) failed outright. */
  STEP_FAILED: "E_STEP_FAILED",
  /** The command ran but did not reach a complete/green state. */
  INCOMPLETE: "E_INCOMPLETE",
  /** A doctor health check failed. */
  CHECK_FAILED: "E_CHECK_FAILED",
  /** Unhandled error (CLI exits 2). */
  UNEXPECTED: "E_UNEXPECTED",
} as const;

export type KitErrorCode =
  (typeof KIT_ERROR_CODES)[keyof typeof KIT_ERROR_CODES];

export interface KitJsonError {
  code: KitErrorCode;
  message: string;
}

export interface KitJsonEnvelope<TData> {
  schemaVersion: typeof KIT_SCHEMA_VERSION;
  command: string;
  ok: boolean;
  data: TData | null;
  warnings: string[];
  errors: KitJsonError[];
}

export function makeKitEnvelope<TData>(input: {
  command: string;
  ok: boolean;
  data: TData | null;
  warnings?: string[];
  errors?: KitJsonError[];
}): KitJsonEnvelope<TData> {
  return {
    schemaVersion: KIT_SCHEMA_VERSION,
    command: input.command,
    ok: input.ok,
    data: input.data,
    warnings: input.warnings ?? [],
    errors: input.errors ?? [],
  };
}
