/** User-level Kit config stored at ~/.kit/config.json */

export interface KitConfig {
  version: 1;
  /** True after the user finishes or skips first-run. */
  firstRunCompleted: boolean;
  /** Preferred starter pack name (default essentials). */
  preferredPack: string;
  /**
   * Project Kit is “pointed at” for auto-recommend / apply / link.
   * Absolute or relative path; empty means process.cwd().
   */
  targetProjectDir?: string;
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

/**
 * Official packs on first-run (v1 ship set — 7 packs).
 * Stack packs extend essentials so dependency skills always install.
 */
export const FIRST_RUN_PACK_OPTIONS = [
  {
    key: "1",
    name: "essentials",
    title: "Essentials",
    blurb: "Best default — setup, docs, review, tests, fix, PRs.",
  },
  {
    key: "2",
    name: "web-app",
    title: "Web App",
    blurb: "Essentials + ship, a11y, PR craft for apps/sites.",
  },
  {
    key: "3",
    name: "library",
    title: "Library",
    blurb: "Essentials + API docs and changelog for packages.",
  },
  {
    key: "4",
    name: "cli-tool",
    title: "CLI Tool",
    blurb: "Essentials + CLI help for terminal tools.",
  },
  {
    key: "5",
    name: "api-service",
    title: "API Service",
    blurb: "Essentials + API docs and ship for backends.",
  },
  {
    key: "6",
    name: "full-stack",
    title: "Full Stack",
    blurb: "Essentials + web + API skills for full products.",
  },
  {
    key: "7",
    name: "data-ml",
    title: "Data / ML",
    blurb: "Essentials + data checks for notebooks and pipelines.",
  },
] as const;

export type FirstRunPackName =
  (typeof FIRST_RUN_PACK_OPTIONS)[number]["name"];
