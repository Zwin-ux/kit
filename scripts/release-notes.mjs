#!/usr/bin/env node
// Print the GitHub Release notes for a version: its CHANGELOG.md section,
// then how to install and verify it. Fails when CHANGELOG.md has no
// section for the version, so a tag cannot ship without release notes.
//
//   node scripts/release-notes.mjs 2.0.0 [--repo Zwin-ux/kit]
//
// A section is a `## <version>` heading (anything after the version, such
// as ` — Kits`, is its title) up to the next `## ` heading.
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function die(msg) {
  console.error(`release-notes: ${msg}`);
  process.exit(1);
}

export function section(changelog, version) {
  const lines = changelog.split(/\r?\n/);
  const escaped = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const head = new RegExp(`^## \\[?v?${escaped}\\]?(\\s|$)`);
  const start = lines.findIndex((l) => head.test(l));
  if (start < 0) return null;
  let end = lines.findIndex((l, i) => i > start && l.startsWith("## "));
  if (end < 0) end = lines.length;
  return lines.slice(start + 1, end).join("\n").trim();
}

function main() {
  const args = process.argv.slice(2);
  const version = args[0];
  if (!version || version.startsWith("-")) die("usage: release-notes.mjs <version> [--repo owner/name]");
  const i = args.indexOf("--repo");
  const repo = i > 0 ? args[i + 1] : "Zwin-ux/kit";
  if (!/^[\w.-]+\/[\w.-]+$/.test(repo ?? "")) die(`--repo needs owner/name, got "${repo ?? ""}"`);
  const body = section(readFileSync(path.join(ROOT, "CHANGELOG.md"), "utf8"), version);
  if (!body) die(`CHANGELOG.md has no "## ${version}" section`);

  const tag = `v${version}`;
  const pre = version.includes("-");
  const raw = `https://raw.githubusercontent.com/${repo}/${tag}/scripts`;
  const npm = pre ? `@mzwin/kit@${version}` : "@mzwin/kit";
  console.log(`${body}

## Install

| | |
|---|---|
| macOS, Linux | \`curl -fsSL ${raw}/install.sh \\| sh -s -- --version ${version}\` |
| Windows | \`& ([scriptblock]::Create((irm ${raw}/install.ps1))) -Version ${version}\` |
| npm | \`npm install -g ${npm}\` |
| Cargo | \`cargo install kitctl --version ${version} --locked\` |

Then run \`kit setup\`.

## Verify

The installers check each archive against \`SHA256SUMS\` and refuse a mismatch. Every archive also has a GitHub build attestation, so you can check it was built by this repository's release workflow from tag \`${tag}\`:

\`\`\`sh
gh attestation verify kit-${version}-<target>.tar.gz --repo ${repo}
\`\`\`

The binaries are not code-signed or notarized. The curl, PowerShell, npm and Cargo installs are not affected. If you download an archive with a browser, macOS may refuse to run \`kit\` until you clear the quarantine flag (\`xattr -d com.apple.quarantine ./kit\`), and Windows SmartScreen may warn. See [docs/dev/RELEASING.md](https://github.com/${repo}/blob/${tag}/docs/dev/RELEASING.md#signing).`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
