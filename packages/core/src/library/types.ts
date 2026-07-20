/** One skill entry in the local library. */
export interface InstalledSkill {
  name: string;
  version: string;
  description: string;
  compatibility: string[];
  /** Absolute path to the installed skill folder. */
  installPath: string;
  /** ISO timestamp when the skill was installed or last replaced. */
  installedAt: string;
  /** Original source folder path, if known. */
  sourcePath?: string;
}

/** On-disk library index (offline-first). */
export interface LibraryIndex {
  version: 1;
  skills: Record<string, LibraryIndexEntry>;
}

export interface LibraryIndexEntry {
  name: string;
  version: string;
  description: string;
  compatibility: string[];
  installedAt: string;
  sourcePath?: string;
}

export type LibraryResult<T> =
  | { ok: true; value: T }
  | { ok: false; error: string };
