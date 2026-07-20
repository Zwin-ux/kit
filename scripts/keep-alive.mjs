#!/usr/bin/env node
/**
 * Kit keep-alive: catalog sync + optional skill promotion from catalog/queue.
 *
 * Usage:
 *   node scripts/keep-alive.mjs              # sync + promote next queue skill
 *   node scripts/keep-alive.mjs --sync-only  # refresh skills/README.md only
 *   node scripts/keep-alive.mjs --check      # validate queue + live skills, no writes
 *
 * Exit codes:
 *   0  ok (writes may or may not have happened)
 *   1  validation / I/O failure
 *   2  --check found problems
 */
import {
  readdir,
  readFile,
  writeFile,
  mkdir,
  rename,
  access,
  rm,
} from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { constants as fsConstants } from "node:fs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const SKILLS_DIR = path.join(ROOT, "skills");
const QUEUE_DIR = path.join(ROOT, "catalog", "queue");
const SKILLS_README = path.join(SKILLS_DIR, "README.md");
const REPORT_PATH = path.join(ROOT, "catalog", "KEEP_ALIVE_REPORT.md");

const args = new Set(process.argv.slice(2));
const SYNC_ONLY = args.has("--sync-only");
const CHECK_ONLY = args.has("--check");

const NAME_RE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

/** @param {string} p */
async function exists(p) {
  try {
    await access(p, fsConstants.F_OK);
    return true;
  } catch {
    return false;
  }
}

/**
 * Minimal front-matter parse (no yaml dependency).
 * @param {string} text
 */
function parseFrontMatter(text) {
  const m = text.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?([\s\S]*)$/);
  if (!m) return { ok: false, error: "missing YAML front matter" };
  const raw = m[1];
  const body = m[2].trim();
  /** @type {Record<string, string | string[]>} */
  const fm = {};
  let listKey = null;
  for (const line of raw.split(/\r?\n/)) {
    if (/^\s*-\s+/.test(line) && listKey) {
      const item = line.replace(/^\s*-\s+/, "").trim();
      if (!Array.isArray(fm[listKey])) fm[listKey] = [];
      /** @type {string[]} */ (fm[listKey]).push(item);
      continue;
    }
    listKey = null;
    const kv = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (!kv) continue;
    const key = kv[1];
    const val = kv[2].trim();
    if (val === "" || val === "|" || val === ">") {
      listKey = key;
      fm[key] = [];
    } else {
      fm[key] = val.replace(/^["']|["']$/g, "");
    }
  }
  return { ok: true, frontMatter: fm, body };
}

/**
 * @param {string} dir
 * @param {string} skillPath
 */
async function validateSkillFile(skillPath, expectedName) {
  const text = await readFile(skillPath, "utf8");
  const parsed = parseFrontMatter(text);
  if (!parsed.ok) return { ok: false, error: parsed.error };
  const { frontMatter, body } = parsed;
  const name = String(frontMatter.name ?? "");
  const description = String(frontMatter.description ?? "");
  const version = String(frontMatter.version ?? "");
  const compatibility = frontMatter.compatibility;

  if (!NAME_RE.test(name)) {
    return { ok: false, error: `invalid name "${name}"` };
  }
  if (expectedName && name !== expectedName) {
    return {
      ok: false,
      error: `name "${name}" does not match folder "${expectedName}"`,
    };
  }
  if (!description || description.length < 12) {
    return { ok: false, error: "description too short" };
  }
  if (!/^\d+\.\d+\.\d+$/.test(version)) {
    return { ok: false, error: `bad version "${version}"` };
  }
  if (!Array.isArray(compatibility) || compatibility.length === 0) {
    return { ok: false, error: "compatibility list required" };
  }
  if (!body || body.length < 40) {
    return { ok: false, error: "body too short" };
  }
  return {
    ok: true,
    name,
    description,
    version,
    compatibility,
    bodyLength: body.length,
  };
}

/** @param {string} dir */
async function listSkillFolders(dir) {
  if (!(await exists(dir))) return [];
  const entries = await readdir(dir, { withFileTypes: true });
  const out = [];
  for (const e of entries) {
    if (!e.isDirectory()) continue;
    if (e.name.startsWith(".") || e.name.startsWith("_")) continue;
    const skillMd = path.join(dir, e.name, "SKILL.md");
    if (await exists(skillMd)) out.push(e.name);
  }
  return out.sort();
}

async function loadLiveCatalog() {
  const names = await listSkillFolders(SKILLS_DIR);
  /** @type {{ name: string, description: string }[]} */
  const rows = [];
  const errors = [];
  for (const name of names) {
    const skillPath = path.join(SKILLS_DIR, name, "SKILL.md");
    const v = await validateSkillFile(skillPath, name);
    if (!v.ok) {
      errors.push(`skills/${name}: ${v.error}`);
      continue;
    }
    rows.push({ name: v.name, description: v.description });
  }
  return { rows, errors };
}

async function loadQueue() {
  const names = await listSkillFolders(QUEUE_DIR);
  /** @type {{ name: string, description: string, dir: string }[]} */
  const items = [];
  const errors = [];
  for (const name of names) {
    const dir = path.join(QUEUE_DIR, name);
    const skillPath = path.join(dir, "SKILL.md");
    const v = await validateSkillFile(skillPath, name);
    if (!v.ok) {
      errors.push(`catalog/queue/${name}: ${v.error}`);
      continue;
    }
    items.push({ name: v.name, description: v.description, dir });
  }
  return { items, errors };
}

/**
 * @param {{ name: string, description: string }[]} rows
 */
function renderSkillsReadme(rows) {
  const table = [
    "| Skill | Purpose |",
    "|-------|---------|",
    ...rows.map((r) => `| \`${r.name}\` | ${r.description} |`),
  ].join("\n");

  return `# Skill catalog

Shared skills used by starter packs and direct installs.

Each skill is a folder with \`SKILL.md\`.  
Schema details for contributors: [docs/dev/SKILL_SCHEMA.md](../docs/dev/SKILL_SCHEMA.md).

## Catalog

${table}

## Install

\`\`\`sh
pnpm kit -- install ./skills/add-readme
pnpm kit -- pack install essentials
\`\`\`

> This table is regenerated by \`node scripts/keep-alive.mjs --sync-only\`.
`;
}

/**
 * @param {object} report
 */
function renderReport(report) {
  const lines = [
    `# Keep-alive report`,
    ``,
    `- Generated: ${report.generatedAt}`,
    `- Mode: ${report.mode}`,
    `- Live skills: ${report.liveCount}`,
    `- Queue remaining: ${report.queueRemaining}`,
    `- Promoted: ${report.promoted ?? "_none_"}`,
    `- Catalog sync: ${report.synced ? "yes" : "no"}`,
    ``,
  ];
  if (report.errors?.length) {
    lines.push(`## Errors`, ``, ...report.errors.map((e) => `- ${e}`), ``);
  }
  if (report.notes?.length) {
    lines.push(`## Notes`, ``, ...report.notes.map((n) => `- ${n}`), ``);
  }
  return lines.join("\n");
}

async function promoteNext(queueItems, liveNames, notes) {
  for (const item of queueItems) {
    if (liveNames.has(item.name)) {
      notes.push(`skip queue ${item.name} (already in skills/)`);
      // Drop stale queue copy so it does not block forever
      const destCheck = path.join(SKILLS_DIR, item.name);
      if (await exists(destCheck)) {
        await rm(item.dir, { recursive: true, force: true });
        notes.push(`removed duplicate queue entry ${item.name}`);
      }
      continue;
    }
    const dest = path.join(SKILLS_DIR, item.name);
    await mkdir(path.dirname(dest), { recursive: true });
    await rename(item.dir, dest);
    notes.push(`promoted catalog/queue/${item.name} → skills/${item.name}`);
    return item.name;
  }
  notes.push("queue empty or nothing to promote");
  return null;
}

async function main() {
  const notes = [];
  const mode = CHECK_ONLY ? "check" : SYNC_ONLY ? "sync-only" : "promote";
  const generatedAt = new Date().toISOString();

  const live = await loadLiveCatalog();
  const queue = await loadQueue();
  const errors = [...live.errors, ...queue.errors];

  if (CHECK_ONLY) {
    if (errors.length) {
      console.error("keep-alive check failed:");
      for (const e of errors) console.error(" -", e);
      process.exit(2);
    }
    console.log(
      `keep-alive check ok · ${live.rows.length} skills · ${queue.items.length} queued`,
    );
    process.exit(0);
  }

  if (errors.length) {
    console.error("validation errors (refusing to write):");
    for (const e of errors) console.error(" -", e);
    process.exit(1);
  }

  let promoted = null;
  if (!SYNC_ONLY) {
    const liveNames = new Set(live.rows.map((r) => r.name));
    promoted = await promoteNext(queue.items, liveNames, notes);
  } else {
    notes.push("sync-only: no promotion");
  }

  // Reload after possible promotion
  const after = await loadLiveCatalog();
  if (after.errors.length) {
    console.error("post-promote validation failed:");
    for (const e of after.errors) console.error(" -", e);
    process.exit(1);
  }

  const readme = renderSkillsReadme(after.rows);
  const prev = (await exists(SKILLS_README))
    ? await readFile(SKILLS_README, "utf8")
    : "";
  const synced = prev !== readme;
  if (synced) {
    await writeFile(SKILLS_README, readme, "utf8");
    notes.push("updated skills/README.md");
  } else {
    notes.push("skills/README.md already current");
  }

  const remaining = (await listSkillFolders(QUEUE_DIR)).length;
  const report = {
    generatedAt,
    mode,
    liveCount: after.rows.length,
    queueRemaining: remaining,
    promoted,
    synced,
    errors: [],
    notes,
  };
  await mkdir(path.dirname(REPORT_PATH), { recursive: true });
  await writeFile(REPORT_PATH, renderReport(report), "utf8");
  notes.push("wrote catalog/KEEP_ALIVE_REPORT.md");

  console.log(JSON.stringify({ ...report, notes }, null, 2));
  if (promoted) {
    // GitHub Actions can branch on this
    console.log(`::notice::Promoted skill ${promoted}`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
