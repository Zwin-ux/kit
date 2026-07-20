/** User-level Kit config stored at ~/.kit/config.json */

export interface KitConfig {
  version: 1;
  /** True after the user finishes or skips first-run. */
  firstRunCompleted: boolean;
  /** Preferred starter pack name (default essentials). */
  preferredPack: string;
  /** ISO timestamp when first-run was completed or skipped. */
  firstRunCompletedAt?: string;
  /** How first-run ended. */
  firstRunOutcome?: "installed" | "skipped";
}

export const DEFAULT_KIT_CONFIG: KitConfig = {
  version: 1,
  firstRunCompleted: false,
  preferredPack: "essentials",
};

/** Official packs offered on first-run (stable order for key bindings). */
export const FIRST_RUN_PACK_OPTIONS = [
  {
    key: "1",
    name: "essentials",
    title: "Essentials",
    blurb: "Docs, setup, review, tests, bugfix — any project.",
  },
  {
    key: "2",
    name: "web-app",
    title: "Web App",
    blurb: "Essentials plus ship checklist and accessibility pass.",
  },
  {
    key: "3",
    name: "library",
    title: "Library",
    blurb: "API docs, changelog, tests, and careful review.",
  },
] as const;

export type FirstRunPackName =
  (typeof FIRST_RUN_PACK_OPTIONS)[number]["name"];
