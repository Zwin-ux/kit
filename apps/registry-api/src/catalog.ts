/**
 * Seed catalog for the public registry MVP.
 * Later this becomes Postgres + storage on Railway.
 */

export interface RegistrySkill {
  name: string;
  description: string;
  version: string;
  compatibility: string[];
}

export interface RegistryPack {
  name: string;
  title: string;
  description: string;
  version: string;
  tags: string[];
  projectTypes: string[];
  skillCount: number;
  skills: string[];
  /** Base packs merged in (dependency skills). */
  extends?: string[];
  publisher: string;
}

export const SEED_SKILLS: RegistrySkill[] = [
  {
    name: "add-readme",
    description: "Create a clear README for a new or existing project.",
    version: "0.1.1",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "project-setup",
    description: "Set up a clean project baseline for agents and humans.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "code-review",
    description: "Review a change for correctness, risk, and clarity.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "write-tests",
    description: "Add focused tests for important project behavior.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "fix-bug",
    description: "Find root cause and fix a bug without drive-by refactors.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "pr-ready",
    description: "Prepare a clear pull request summary, test plan, and risk notes.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "ship-checklist",
    description: "Run a practical pre-ship checklist for an app release.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "a11y-pass",
    description: "Improve basic accessibility for UI and web flows.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "api-docs",
    description: "Document a library or service API with clear usage examples.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "changelog",
    description: "Write a clear changelog entry for a release or notable change.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "cli-help",
    description: "Improve CLI help text, usage examples, and flag documentation.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "data-check",
    description: "Review data scripts and notebooks for clarity, leakage, and reproducibility.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
  {
    name: "workspace-setup",
    description: "Set up monorepo and multi-package workspaces for agents and humans.",
    version: "0.1.0",
    compatibility: ["claude-code", "grok-build", "codex"],
  },
];

const ESSENTIALS = [
  "add-readme",
  "project-setup",
  "code-review",
  "write-tests",
  "fix-bug",
  "pr-ready",
] as const;

export const SEED_PACKS: RegistryPack[] = [
  {
    name: "essentials",
    title: "Essentials",
    description: "Core skills every coding project should have.",
    version: "0.1.0",
    tags: ["starter", "general"],
    projectTypes: ["any"],
    skillCount: ESSENTIALS.length,
    skills: [...ESSENTIALS],
    publisher: "kit",
  },
  {
    name: "web-app",
    title: "Web App",
    description: "Full starter for apps and sites. Includes Essentials plus ship and a11y skills.",
    version: "0.2.0",
    tags: ["starter", "web"],
    projectTypes: ["web", "app", "site"],
    extends: ["essentials"],
    skillCount: 8,
    skills: [
      ...ESSENTIALS,
      "ship-checklist",
      "a11y-pass",
      // pr-ready already in essentials
    ],
    publisher: "kit",
  },
  {
    name: "library",
    title: "Library",
    description: "Full starter for packages and SDKs. Includes Essentials plus API docs and changelog.",
    version: "0.2.0",
    tags: ["starter", "library"],
    projectTypes: ["library", "sdk", "package"],
    extends: ["essentials"],
    skillCount: 8,
    skills: [...ESSENTIALS, "api-docs", "changelog"],
    publisher: "kit",
  },
  {
    name: "cli-tool",
    title: "CLI Tool",
    description: "Full starter for command-line tools. Includes Essentials plus CLI help and PR craft.",
    version: "0.1.0",
    tags: ["starter", "cli"],
    projectTypes: ["cli", "tool", "binary"],
    extends: ["essentials"],
    skillCount: 7,
    skills: [...ESSENTIALS, "cli-help"],
    publisher: "kit",
  },
  {
    name: "api-service",
    title: "API Service",
    description: "Full starter for HTTP services. Includes Essentials plus API docs, ship, and PR craft.",
    version: "0.1.0",
    tags: ["starter", "api", "backend"],
    projectTypes: ["api", "service", "backend"],
    extends: ["essentials"],
    skillCount: 8,
    skills: [...ESSENTIALS, "api-docs", "ship-checklist"],
    publisher: "kit",
  },
  {
    name: "full-stack",
    title: "Full Stack",
    description: "Full starter for apps with a UI and an API. Includes Essentials plus web and service skills.",
    version: "0.1.0",
    tags: ["starter", "fullstack", "web", "api"],
    projectTypes: ["full-stack", "web", "api", "app"],
    extends: ["essentials"],
    skillCount: 9,
    skills: [...ESSENTIALS, "ship-checklist", "a11y-pass", "api-docs"],
    publisher: "kit",
  },
  {
    name: "data-ml",
    title: "Data / ML",
    description: "Full starter for data and ML work. Includes Essentials plus data checks and tests.",
    version: "0.1.0",
    tags: ["starter", "data", "ml", "python"],
    projectTypes: ["data", "ml", "notebook", "research"],
    extends: ["essentials"],
    skillCount: 7,
    skills: [...ESSENTIALS, "data-check"],
    publisher: "kit",
  },
];
