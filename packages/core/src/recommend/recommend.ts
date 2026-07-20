import { access, readFile } from "node:fs/promises";
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

export interface RecommendReport {
  projectDir: string;
  signals: string[];
  recommendations: ToolkitRecommendation[];
  /** Best pack name, if any. */
  topPick: string | null;
}

export type RecommendResult =
  | { ok: true; value: RecommendReport }
  | { ok: false; error: string };

/**
 * Recommend starter packs from simple project signals.
 * Offline-first: uses local packs/ catalog when available.
 */
export async function recommendToolkits(
  options: PackLoadOptions & { projectDir?: string } = {},
): Promise<RecommendResult> {
  const projectDir = path.resolve(options.projectDir ?? process.cwd());
  const signals: string[] = [];
  const scores = new Map<string, { score: number; reasons: string[] }>();

  const bump = (name: string, points: number, reason: string) => {
    const cur = scores.get(name) ?? { score: 0, reasons: [] };
    cur.score += points;
    if (!cur.reasons.includes(reason)) cur.reasons.push(reason);
    scores.set(name, cur);
  };

  // Always soft-recommend essentials.
  bump("essentials", 5, "Solid default for any repo");

  const pkgPath = path.join(projectDir, "package.json");
  if (await exists(pkgPath)) {
    signals.push("package.json");
    try {
      const pkg = JSON.parse(await readFile(pkgPath, "utf8")) as {
        name?: string;
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

      const webHints = [
        "react",
        "next",
        "vue",
        "svelte",
        "astro",
        "vite",
        "webpack",
        "express",
        "hono",
        "remix",
      ];
      if (webHints.some((h) => names.some((n) => n === h || n.startsWith(`${h}/`)))) {
        bump("web-app", 20, "Detected web/app framework dependencies");
        signals.push("web-framework");
      }

      if (
        pkg.name &&
        (names.includes("typescript") || names.includes("@types/node")) &&
        !webHints.some((h) => names.includes(h))
      ) {
        bump("library", 12, "Package-style project without a web framework");
        signals.push("library-shaped");
      }

      if (
        names.includes("vitest") ||
        names.includes("jest") ||
        names.includes("mocha")
      ) {
        bump("essentials", 3, "Test runner present — keep review/test skills");
        signals.push("tests");
      }
    } catch {
      signals.push("package.json-unreadable");
    }
  }

  if (await exists(path.join(projectDir, "Cargo.toml"))) {
    signals.push("Cargo.toml");
    bump("library", 10, "Rust crate layout");
    bump("essentials", 4, "Systems project still needs core skills");
  }

  if (await exists(path.join(projectDir, "pyproject.toml")) || await exists(path.join(projectDir, "requirements.txt"))) {
    signals.push("python");
    bump("essentials", 6, "Python project");
    bump("library", 8, "Python package-style layout likely");
  }

  if (await exists(path.join(projectDir, "apps")) || await exists(path.join(projectDir, "packages"))) {
    signals.push("monorepo");
    bump("essentials", 4, "Monorepo layout");
    bump("web-app", 6, "Monorepos often ship apps");
  }

  if (await exists(path.join(projectDir, "src-tauri"))) {
    signals.push("tauri");
    bump("web-app", 15, "Tauri desktop app shell");
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

  // Ensure known official packs appear even with zero signal beyond essentials.
  for (const name of ["essentials", "web-app", "library"]) {
    if (!recommendations.some((r) => r.packName === name)) {
      const pack = packByName.get(name);
      const rec: ToolkitRecommendation = {
        packName: name,
        title: pack?.title ?? name,
        score: name === "essentials" ? 1 : 0,
        reasons: name === "essentials" ? ["Always available"] : ["Available catalog pack"],
      };
      if (pack) rec.pack = pack;
      recommendations.push(rec);
    }
  }

  recommendations.sort((a, b) => b.score - a.score || a.packName.localeCompare(b.packName));

  return {
    ok: true,
    value: {
      projectDir,
      signals,
      recommendations,
      topPick: recommendations[0]?.packName ?? null,
    },
  };
}

async function exists(p: string): Promise<boolean> {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
}
