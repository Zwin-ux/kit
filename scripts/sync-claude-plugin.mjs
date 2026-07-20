#!/usr/bin/env node
/**
 * Build a local Claude Code plugin package from Kit skills.
 * Output: dist/claude-plugin/
 */
import { cp, mkdir, rm, writeFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const out = path.join(root, "dist", "claude-plugin");
const skillsSrc = path.join(root, "skills");

async function exists(p) {
  try {
    await access(p);
    return true;
  } catch {
    return false;
  }
}

await rm(out, { recursive: true, force: true });
await mkdir(path.join(out, ".claude-plugin"), { recursive: true });
await mkdir(path.join(out, "skills"), { recursive: true });

if (!(await exists(skillsSrc))) {
  throw new Error("skills/ missing");
}

await cp(skillsSrc, path.join(out, "skills"), {
  recursive: true,
  filter: (src) => {
    const base = path.basename(src);
    return base !== "README.md" && base !== "node_modules" && base !== ".git";
  },
});

const pluginJson = {
  name: "kit",
  version: "0.1.0",
  description:
    "Portable Kit agent skills — essentials workflows for Claude Code",
  author: { name: "Zwin-ux" },
};

await writeFile(
  path.join(out, ".claude-plugin", "plugin.json"),
  JSON.stringify(pluginJson, null, 2) + "\n",
  "utf8",
);

await writeFile(
  path.join(out, "README.md"),
  `# Kit Claude plugin

Local Claude Code plugin built from the Kit skill catalog.

## Install (local path)

From Claude Code, add this plugin from the repo:

\`\`\`text
${out}
\`\`\`

Or after clone:

\`\`\`bash
node scripts/sync-claude-plugin.mjs
# point Claude plugin install at dist/claude-plugin
\`\`\`

Skills also install via:

\`\`\`bash
npm i -g @mzwin/kit
kit link --to claude-code --write
\`\`\`
`,
  "utf8",
);

console.log("wrote", out);
