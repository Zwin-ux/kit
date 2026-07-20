/**
 * Shared types and utilities for Kit packages.
 * Keep this package free of skill business logic and TUI code.
 */

/** Semantic version string of the Kit monorepo packages. */
export const KIT_PACKAGE_VERSION = "0.1.5" as const;

/** Result of a fallible operation without throwing. */
export type Result<T, E = string> =
  | { ok: true; value: T }
  | { ok: false; error: E };

export function ok<T>(value: T): Result<T, never> {
  return { ok: true, value };
}

export function err<E>(error: E): Result<never, E> {
  return { ok: false, error };
}
