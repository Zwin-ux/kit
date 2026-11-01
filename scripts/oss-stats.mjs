#!/usr/bin/env node
// How close Kit is to the maintainer-program tracks in docs/PRD.md §11.1.
// Counts the last 12 months, the window Claude for Open Source uses.
//
//   node scripts/oss-stats.mjs [--repo Zwin-ux/kit] [--exclude a,b] [--json]
//
// Needs `gh` signed in. `--exclude` drops more handles from the external
// count (the owner's other accounts, say). Downloads come from crates.io
// (`kitctl`) and npm (`@mzwin/kit`).
import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const TRACKS = { externalContributors: 20, upstreamPrs: 100, monthlyDownloads: 200_000 };

function die(msg) {
  console.error(`oss-stats: ${msg}`);
  process.exit(1);
}

function arg(name, fallback) {
  const i = process.argv.indexOf(name);
  if (i < 0) return fallback;
  const value = process.argv[i + 1];
  if (!value || value.startsWith("--")) die(`${name} needs a value`);
  return value;
}

/** `YYYY-MM-DD`, twelve months before `now`. */
export function windowStart(now = new Date()) {
  const start = new Date(now);
  start.setUTCFullYear(start.getUTCFullYear() - 1);
  return start.toISOString().slice(0, 10);
}

/** Unique PR authors that count as external: not the owner, not a bot, not excluded. */
export function externalAuthors(logins, owner, exclude = []) {
  // GitHub logins are case-insensitive.
  const skip = new Set([owner, ...exclude].map((l) => l.toLowerCase()));
  const out = new Map();
  for (const login of logins) {
    const l = login.toLowerCase();
    if (skip.has(l) || l.endsWith("[bot]") || l === "claude" || out.has(l)) continue;
    out.set(l, login);
  }
  return [...out.values()].sort((a, b) => a.localeCompare(b));
}

function searchLogins(query) {
  const raw = execFileSync(
    "gh",
    ["api", "-X", "GET", "search/issues", "--paginate", "-f", `q=${query}`, "-f", "per_page=100", "-q", ".items[].user.login"],
    { encoding: "utf8" },
  );
  return raw.split(/\r?\n/).filter(Boolean);
}

async function getJson(url) {
  const res = await fetch(url, { headers: { "user-agent": "kit-oss-stats (github.com/Zwin-ux/kit)" } });
  if (!res.ok) return null;
  return res.json();
}

async function main() {
  const repo = arg("--repo", "Zwin-ux/kit");
  const exclude = arg("--exclude", "").split(",").filter(Boolean);
  const owner = repo.split("/")[0];
  const since = windowStart();

  const authors = searchLogins(`repo:${repo} is:pr is:merged merged:>=${since}`);
  const external = externalAuthors(authors, owner, exclude);
  const upstream = searchLogins(`is:pr is:merged author:${owner} -user:${owner} merged:>=${since}`).length;

  const crate = await getJson("https://crates.io/api/v1/crates/kitctl");
  const npm = await getJson("https://api.npmjs.org/downloads/point/last-month/@mzwin%2Fkit");
  // crates.io reports the last 90 days; a third of that is one month.
  const crateMonthly = crate ? Math.round(crate.crate.recent_downloads / 3) : null;
  const npmMonthly = npm ? npm.downloads : null;
  const monthly = (crateMonthly ?? 0) + (npmMonthly ?? 0);

  const result = {
    since,
    externalContributors: { count: external.length, need: TRACKS.externalContributors, logins: external },
    upstreamPrs: { count: upstream, need: TRACKS.upstreamPrs },
    monthlyDownloads: { count: monthly, need: TRACKS.monthlyDownloads, crates: crateMonthly, npm: npmMonthly },
  };
  if (process.argv.includes("--json")) {
    console.log(JSON.stringify(result, null, 2));
    return;
  }
  const row = (label, { count, need }) =>
    `${count >= need ? "met " : "    "} ${label.padEnd(34)} ${String(count).padStart(7)} / ${need}`;
  console.log(`Since ${since} (PRD §11.1)\n`);
  console.log(row(`External contributors to ${repo}`, result.externalContributors));
  console.log(row(`Merged PRs to repos ${owner} doesn't own`, result.upstreamPrs));
  console.log(row("Monthly downloads (crates.io + npm)", result.monthlyDownloads));
  if (external.length) console.log(`\nExternal: ${external.join(", ")}`);
  if (crateMonthly === null || npmMonthly === null) console.log("\nA registry didn't answer; its downloads count as 0.");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((err) => die(err.message));
}
