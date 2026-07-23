/**
 * Project + skill fixtures for CLI e2e tests.
 *
 * SKILL.md shape mirrors packages/core/tests/fixtures/valid-add-readme:
 * YAML front matter with name/description/version/compatibility, then a body.
 */
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { expectExit, type Sandbox } from "./harness.js";

export interface SkillFixtureOptions {
  description?: string;
  version?: string;
  compatibility?: string[];
  body?: string;
}

/** Render a Kit-shaped SKILL.md. */
export function skillMd(name: string, options: SkillFixtureOptions = {}): string {
  const description =
    options.description ??
    `Run the ${name} workflow with clear, reviewable steps.`;
  const version = options.version ?? "0.1.0";
  const compatibility = options.compatibility ?? [
    "claude-code",
    "codex",
    "grok-build",
  ];
  const body =
    options.body ??
    [
      "# Instructions",
      "",
      "1. Read the relevant project files before changing anything.",
      "2. Apply the change in the smallest reviewable increment.",
      "3. Run the project's tests and record the results.",
      "4. Summarize what changed and why in one short paragraph.",
    ].join("\n");

  const compat = compatibility.map((c) => `  - ${c}`).join("\n");
  return `---\nname: ${name}\ndescription: ${description}\nversion: ${version}\ncompatibility:\n${compat}\n---\n\n${body}\n`;
}

/**
 * Body strong enough to score as a unify keeper (S/A band):
 * headings + numbered steps + 200-4000 chars.
 */
export function keeperBody(name: string): string {
  return [
    `# ${name}`,
    "",
    "## When to use",
    "",
    "Use this skill whenever the task matches its name and the repo is in a",
    "reviewable state. Prefer small, verifiable increments over big rewrites.",
    "",
    "## Steps",
    "",
    "1. Inspect the project layout and identify the files involved.",
    "2. Draft the change and note any risky areas before editing.",
    "3. Apply the edit and keep the diff minimal.",
    "4. Run the relevant tests and capture their output.",
    "5. Write a short summary: what changed, why, and how it was verified.",
  ].join("\n");
}

/** Create `<parent>/<name>/SKILL.md`; returns the skill dir. */
export async function seedSkillDir(
  parent: string,
  name: string,
  options: SkillFixtureOptions = {},
): Promise<string> {
  const dir = path.join(parent, name);
  await mkdir(dir, { recursive: true });
  await writeFile(path.join(dir, "SKILL.md"), skillMd(name, options), "utf8");
  return dir;
}

/** A minimal web-ish project (react dependency drives `recommend` → web-app). */
export async function makeWebProject(dir: string): Promise<void> {
  await mkdir(path.join(dir, "src"), { recursive: true });
  await writeFile(
    path.join(dir, "package.json"),
    JSON.stringify(
      {
        name: "e2e-web-fixture",
        version: "0.0.0",
        private: true,
        dependencies: { react: "^18.2.0", "react-dom": "^18.2.0" },
      },
      null,
      2,
    ),
    "utf8",
  );
  await writeFile(
    path.join(dir, "src", "index.tsx"),
    "export const App = () => null;\n",
    "utf8",
  );
}

/** A plain directory project (no signals). */
export async function makePlainProject(dir: string): Promise<void> {
  await mkdir(dir, { recursive: true });
  await writeFile(path.join(dir, "README.md"), "# plain fixture\n", "utf8");
}

/**
 * Install skills into the sandbox Kit library through the real CLI
 * (`kit install <dir>`), so fixtures never poke library internals.
 */
export async function seedLibrary(
  sandbox: Sandbox,
  names: string[],
): Promise<void> {
  const staging = path.join(sandbox.root, "staging-skills");
  for (const name of names) {
    const dir = await seedSkillDir(staging, name);
    const result = await sandbox.runKit(["install", dir]);
    expectExit(result, 0);
  }
}
