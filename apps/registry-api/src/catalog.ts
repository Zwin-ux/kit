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
];

export const SEED_PACKS: RegistryPack[] = [
  {
    name: "essentials",
    title: "Essentials",
    description: "Core skills every coding project should have.",
    version: "0.1.0",
    tags: ["starter", "general"],
    projectTypes: ["any"],
    skillCount: 5,
    skills: [
      "add-readme",
      "project-setup",
      "code-review",
      "write-tests",
      "fix-bug",
    ],
    publisher: "kit",
  },
  {
    name: "web-app",
    title: "Web App",
    description: "Starter skills for application and site projects.",
    version: "0.1.0",
    tags: ["starter", "web"],
    projectTypes: ["web", "app", "site"],
    skillCount: 7,
    skills: [
      "add-readme",
      "project-setup",
      "code-review",
      "write-tests",
      "fix-bug",
      "ship-checklist",
      "a11y-pass",
    ],
    publisher: "kit",
  },
  {
    name: "library",
    title: "Library",
    description: "Starter skills for libraries, SDKs, and shared packages.",
    version: "0.1.0",
    tags: ["starter", "library"],
    projectTypes: ["library", "sdk", "package"],
    skillCount: 5,
    skills: [
      "add-readme",
      "api-docs",
      "changelog",
      "write-tests",
      "code-review",
    ],
    publisher: "kit",
  },
];
