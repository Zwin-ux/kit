import { access, readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { listPacks, type PackLoadOptions } from "../pack/loadPack.js";
import type { PackListItem } from "../pack/types.js";

export interface ToolkitRecommendation {
  packName: string;
  title: string;
  score: number;
  reasons: string[];
  pack?: PackListItem;
}

/** Individual skills Kit thinks this project should have. */
export interface SkillRecommendation {
  skillName: string;
  score: number;
  reasons: string[];
  /** Pack that usually provides this skill (if known). */
  fromPack?: string;
}

export interface RecommendReport {
  projectDir: string;
  /** Human one-liner: "Next.js app → web-app" */
  summary: string;
  signals: string[];
  recommendations: ToolkitRecommendation[];
  /** Skill-level picks for this project (auto, from signals). */
  skillRecommendations: SkillRecommendation[];
  /** Best pack name, if any. */
  topPick: string | null;
}

export type RecommendResult =
  | { ok: true; value: RecommendReport }
  | { ok: false; error: string };

/**
 * Point Kit at a project directory and get pack + skill recommendations.
 * Offline-first: scans local files only; uses packs/ catalog when available.
 */
export async function recommendToolkits(
  options: PackLoadOptions & { projectDir?: string } = {},
): Promise<RecommendResult> {
  const projectDir = path.resolve(options.projectDir ?? process.cwd());

  try {
    await access(projectDir);
  } catch {
    return {
      ok: false,
      error: `Project path not found: ${projectDir}`,
    };
  }

  const signals: string[] = [];
  const scores = new Map<string, { score: number; reasons: string[] }>();
  const skillScores = new Map<string, { score: number; reasons: string[] }>();

  const bump = (name: string, points: number, reason: string) => {
    const cur = scores.get(name) ?? { score: 0, reasons: [] };
    cur.score += points;
    if (!cur.reasons.includes(reason)) cur.reasons.push(reason);
    scores.set(name, cur);
  };

  const bumpSkill = (name: string, points: number, reason: string) => {
    const cur = skillScores.get(name) ?? { score: 0, reasons: [] };
    cur.score += points;
    if (!cur.reasons.includes(reason)) cur.reasons.push(reason);
    skillScores.set(name, cur);
  };

  // Baseline — every project benefits from essentials + PR craft.
  bump("essentials", 5, "Solid default for any repo");
  bumpSkill("project-setup", 4, "Baseline project hygiene");
  bumpSkill("add-readme", 3, "Every project needs a clear README");
  bumpSkill("code-review", 3, "Review loop for agent work");
  bumpSkill("pr-ready", 4, "Ship changes with a clean PR write-up");

  // --- package.json / JS-TS ecosystem ---
  const pkgPath = path.join(projectDir, "package.json");
  if (await exists(pkgPath)) {
    signals.push("package.json");
    try {
      const pkg = JSON.parse(await readFile(pkgPath, "utf8")) as {
        name?: string;
        bin?: string | Record<string, string>;
        scripts?: Record<string, string>;
        dependencies?: Record<string, string>;
        devDependencies?: Record<string, string>;
        peerDependencies?: Record<string, string>;
      };
      const deps = {
        ...pkg.dependencies,
        ...pkg.devDependencies,
        ...pkg.peerDependencies,
      };
      const names = Object.keys(deps);
      const scripts = Object.keys(pkg.scripts ?? {});

      const webHints = [
        "react",
        "next",
        "vue",
        "svelte",
        "astro",
        "vite",
        "webpack",
        "remix",
        "nuxt",
        "@angular/core",
        "solid-js",
      ];
      if (
        webHints.some((h) => names.some((n) => n === h || n.startsWith(`${h}/`)))
      ) {
        bump("web-app", 20, "Detected web/app framework dependencies");
        signals.push("web-framework");
        bumpSkill("a11y-pass", 12, "UI stack benefits from an a11y pass");
        bumpSkill("ship-checklist", 10, "Apps need a pre-ship checklist");
      }

      if (
        pkg.name &&
        (names.includes("typescript") || names.includes("@types/node")) &&
        !webHints.some((h) => names.some((n) => n === h || n.startsWith(`${h}/`)))
      ) {
        bump("library", 12, "Package-style project without a web framework");
        signals.push("library-shaped");
        bumpSkill("api-docs", 10, "Packages need public API docs");
        bumpSkill("changelog", 8, "Libraries need release notes");
      }

      if (
        names.includes("vitest") ||
        names.includes("jest") ||
        names.includes("mocha") ||
        names.includes("@playwright/test") ||
        scripts.some((s) => /test|spec/.test(s))
      ) {
        bump("essentials", 3, "Test runner present — keep review/test skills");
        signals.push("tests");
        bumpSkill("write-tests", 10, "Project already has a test culture");
      }

      const cliHints = ["commander", "yargs", "oclif", "cac", "clipanion", "meow"];
      if (cliHints.some((h) => names.includes(h)) || Boolean(pkg.bin)) {
        bump("cli-tool", 18, "CLI framework or package.bin detected");
        signals.push("cli");
        bumpSkill("cli-help", 14, "CLI projects need strong help/usage");
      }

      const apiHints = [
        "express",
        "hono",
        "fastify",
        "koa",
        "@nestjs/core",
        "@trpc/server",
        "trpc",
      ];
      if (
        apiHints.some((h) => names.some((n) => n === h || n.startsWith(`${h}/`)))
      ) {
        bump("api-service", 16, "HTTP/API framework dependencies");
        signals.push("api-framework");
        bumpSkill("api-docs", 12, "Services need clear API documentation");
        bumpSkill("ship-checklist", 8, "APIs still need a ship checklist");
      }

      if (names.includes("react-native") || names.includes("expo")) {
        signals.push("mobile");
        bump("web-app", 10, "Mobile UI stack — closest starter is web-app");
        bumpSkill("a11y-pass", 6, "Mobile UI still needs accessibility care");
      }

      // Full-stack: both UI and API frameworks present
      const hasWeb = webHints.some((h) =>
        names.some((n) => n === h || n.startsWith(`${h}/`)),
      );
      const hasApi = apiHints.some((h) =>
        names.some((n) => n === h || n.startsWith(`${h}/`)),
      );
      if (hasWeb && hasApi) {
        bump("full-stack", 22, "UI + API frameworks in the same project");
        signals.push("full-stack");
      }

      // Data / ML npm libs
      const dataHints = [
        "pandas",
        "numpy",
        "@tensorflow/tfjs",
        "ml-matrix",
        "brain.js",
      ];
      if (dataHints.some((h) => names.includes(h))) {
        bump("data-ml", 14, "Data/ML libraries in package.json");
        signals.push("data-ml");
        bumpSkill("data-check", 12, "Data projects need hygiene checks");
      }
    } catch {
      signals.push("package.json-unreadable");
    }
  }

  // --- other ecosystems ---
  if (await exists(path.join(projectDir, "Cargo.toml"))) {
    signals.push("Cargo.toml");
    bump("library", 10, "Rust crate layout");
    bump("essentials", 4, "Systems project still needs core skills");
    bumpSkill("api-docs", 6, "Crates often need public API notes");
  }

  if (
    (await exists(path.join(projectDir, "pyproject.toml"))) ||
    (await exists(path.join(projectDir, "requirements.txt"))) ||
    (await exists(path.join(projectDir, "setup.py")))
  ) {
    signals.push("python");
    bump("essentials", 6, "Python project");
    bump("library", 8, "Python package-style layout likely");
    bumpSkill("write-tests", 6, "Python projects usually need tests");

    // Heuristic: ML-ish paths
    if (
      (await exists(path.join(projectDir, "notebooks"))) ||
      (await exists(path.join(projectDir, "models"))) ||
      (await exists(path.join(projectDir, "data")))
    ) {
      bump("data-ml", 16, "Python project with data/ml layout");
      signals.push("data-ml");
      bumpSkill("data-check", 12, "Data folders present");
    }
  }

  if (
    (await exists(path.join(projectDir, "go.mod"))) ||
    (await exists(path.join(projectDir, "go.sum")))
  ) {
    signals.push("go");
    bump("essentials", 6, "Go module");
    bump("api-service", 8, "Go modules often ship services/CLIs");
    bump("cli-tool", 6, "Go is common for CLI binaries");
  }

  if (
    (await exists(path.join(projectDir, "Gemfile"))) ||
    (await exists(path.join(projectDir, "composer.json")))
  ) {
    signals.push("app-ecosystem");
    bump("essentials", 6, "Application ecosystem detected");
    bump("web-app", 8, "Likely an app-style repo");
  }

  if (
    (await exists(path.join(projectDir, "Dockerfile"))) ||
    (await exists(path.join(projectDir, "docker-compose.yml"))) ||
    (await exists(path.join(projectDir, "compose.yaml")))
  ) {
    signals.push("docker");
    bump("api-service", 6, "Containerized project — often a service");
    bumpSkill("ship-checklist", 6, "Deployed projects need a ship checklist");
  }

  if (
    (await exists(path.join(projectDir, "apps"))) ||
    (await exists(path.join(projectDir, "packages"))) ||
    (await exists(path.join(projectDir, "pnpm-workspace.yaml"))) ||
    (await exists(path.join(projectDir, "lerna.json")))
  ) {
    signals.push("monorepo");
    bump("essentials", 4, "Monorepo layout");
    bump("web-app", 6, "Monorepos often ship apps");
    bumpSkill("project-setup", 5, "Monorepos need clear setup docs");
  }

  if (await exists(path.join(projectDir, "src-tauri"))) {
    signals.push("tauri");
    bump("web-app", 15, "Tauri desktop app shell");
  }

  if (
    (await exists(path.join(projectDir, "android"))) ||
    (await exists(path.join(projectDir, "ios")))
  ) {
    signals.push("native-mobile");
    bump("web-app", 8, "Native mobile folders present");
  }

  // Loose file names in root (cheap scan)
  try {
    const entries = await readdir(projectDir);
    if (entries.some((e) => /^readme/i.test(e))) {
      signals.push("has-readme");
    } else {
      bumpSkill("add-readme", 8, "No README detected at project root");
    }
    if (entries.some((e) => e === "AGENTS.md" || e === "CLAUDE.md")) {
      signals.push("agent-docs");
      bumpSkill("project-setup", 4, "Agent instruction files present");
    }
  } catch {
    // ignore
  }

  const packsResult = await listPacks(options);
  const packByName = new Map<string, PackListItem>();
  if (packsResult.ok) {
    for (const p of packsResult.value) packByName.set(p.name, p);
  }

  const recommendations: ToolkitRecommendation[] = [...scores.entries()]
    .map(([packName, data]) => {
      const pack = packByName.get(packName);
      const rec: ToolkitRecommendation = {
        packName,
        title: pack?.title ?? packName,
        score: data.score,
        reasons: data.reasons,
      };
      if (pack) rec.pack = pack;
      return rec;
    })
    .sort((a, b) => b.score - a.score || a.packName.localeCompare(b.packName));

  for (const name of [
    "essentials",
    "web-app",
    "library",
    "cli-tool",
    "api-service",
    "full-stack",
    "data-ml",
  ]) {
    if (!recommendations.some((r) => r.packName === name)) {
      const pack = packByName.get(name);
      const rec: ToolkitRecommendation = {
        packName: name,
        title: pack?.title ?? name,
        score: name === "essentials" ? 1 : 0,
        reasons:
          name === "essentials"
            ? ["Always available"]
            : ["Available catalog pack"],
      };
      if (pack) rec.pack = pack;
      recommendations.push(rec);
    }
  }

  recommendations.sort(
    (a, b) => b.score - a.score || a.packName.localeCompare(b.packName),
  );

  const topPick = recommendations[0]?.packName ?? null;

  // Attribute top skills to packs when possible
  const skillFromPack: Record<string, string> = {
    "add-readme": "essentials",
    "project-setup": "essentials",
    "code-review": "essentials",
    "write-tests": "essentials",
    "fix-bug": "essentials",
    "pr-ready": "essentials",
    "ship-checklist": "web-app",
    "a11y-pass": "web-app",
    "api-docs": "library",
    changelog: "library",
    "cli-help": "cli-tool",
    "data-check": "data-ml",
  };

  const skillRecommendations: SkillRecommendation[] = [...skillScores.entries()]
    .map(([skillName, data]) => {
      const fromPack = skillFromPack[skillName];
      const rec: SkillRecommendation = {
        skillName,
        score: data.score,
        reasons: data.reasons,
      };
      if (fromPack) rec.fromPack = fromPack;
      return rec;
    })
    .sort((a, b) => b.score - a.score || a.skillName.localeCompare(b.skillName))
    .slice(0, 8);

  const summary = buildSummary(projectDir, signals, topPick);

  return {
    ok: true,
    value: {
      projectDir,
      summary,
      signals,
      recommendations,
      skillRecommendations,
      topPick,
    },
  };
}

function buildSummary(
  projectDir: string,
  signals: string[],
  topPick: string | null,
): string {
  const base = path.basename(projectDir) || projectDir;
  if (!topPick) {
    return `${base} · no strong pack signal — try essentials`;
  }
  if (signals.includes("full-stack")) {
    return `${base} looks full-stack → ${topPick}`;
  }
  if (signals.includes("data-ml")) {
    return `${base} looks like data/ML → ${topPick}`;
  }
  if (signals.includes("web-framework")) {
    return `${base} looks like a web app → ${topPick}`;
  }
  if (signals.includes("cli")) {
    return `${base} looks like a CLI → ${topPick}`;
  }
  if (signals.includes("api-framework")) {
    return `${base} looks like an API/service → ${topPick}`;
  }
  if (signals.includes("library-shaped")) {
    return `${base} looks like a library → ${topPick}`;
  }
  if (signals.includes("python") || signals.includes("go") || signals.includes("Cargo.toml")) {
    return `${base} (${signals[0]}) → ${topPick}`;
  }
  if (signals.length === 0 || (signals.length === 1 && signals[0] === "package.json")) {
    return `${base} · start with ${topPick}`;
  }
  return `${base} → ${topPick}`;
}

async function exists(p: string): Promise<boolean> {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
}
