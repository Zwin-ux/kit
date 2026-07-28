import type { StdioOptions } from "node:child_process";

import type { LibraryResult } from "../library/types.js";

export interface KitPluginManifest {
  schemaVersion: 1;
  name: string;
  displayName: string;
  description: string;
  version: string;
  command: string;
  defaultArgs?: string[];
  localExecutables?: Partial<
    Record<"win32" | "darwin" | "linux" | "default", string>
  >;
  versionArgs?: string[];
  healthArgs?: string[];
  safety?: {
    summary: string;
    confirmationToken?: string;
  };
}

export interface PluginRegistryEntry {
  name: string;
  root: string;
  manifestPath: string;
  manifestDigest: string;
  addedAt: string;
}

export interface PluginRegistry {
  version: 1;
  plugins: Record<string, PluginRegistryEntry>;
}

export interface RegisteredPlugin {
  manifest: KitPluginManifest;
  entry: PluginRegistryEntry;
}

export interface AddPluginOptions {
  kitHome?: string;
  write?: boolean;
  force?: boolean;
}

export interface AddPluginReport {
  dryRun: boolean;
  manifest: KitPluginManifest;
  root: string;
  executable: string | null;
  executableSource: "local" | "path" | "missing";
}

export interface PluginDoctorReport {
  name: string;
  ready: boolean;
  manifestChanged: boolean;
  executable: string | null;
  executableSource: "local" | "path" | "missing";
  healthArgs: string[];
  safetySummary?: string;
  confirmationToken?: string;
}

export interface RunPluginOptions {
  kitHome?: string;
  stdio?: StdioOptions;
}

export interface PluginRunReport {
  name: string;
  command: string;
  args: string[];
  exitCode: number;
  stdout: string;
  stderr: string;
}

export type PluginResult<T> = LibraryResult<T>;
