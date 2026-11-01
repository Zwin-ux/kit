#!/usr/bin/env node
// Line budgets for Kit's agent core (docs/PRD.md §2, §8.1).
//
// Counts Rust code lines: not blank, not a comment, and not test code
// (files under tests/, and #[cfg(test)] items). A budgeted crate that does
// not exist yet is reported and skipped, so the job runs from A0.0 on.
//
//   node scripts/loc-budget.mjs          check, exit 1 if over budget
//   node scripts/loc-budget.mjs --json   the same, as JSON

import { readdirSync, readFileSync, existsSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

/** Whole-crate budgets, and per-file budgets for adapter crates. */
export const BUDGETS = [
  { crate: "crates/kit-loop", total: 3000 },
  { crate: "crates/kit-ext", total: 1500 },
  { crate: "crates/kit-sandbox", total: 1500 },
  { crate: "crates/kit-providers", perFile: 600, shared: ["lib.rs", "sse.rs", "retry.rs"], sharedTotal: 400 },
];

/**
 * Code lines in one Rust source: blank lines, `//` comments, `/* … *\/`
 * comments and `#[cfg(test)]` items are not counted. Braces inside string
 * literals are not tracked; budgets are a guide, not a parser.
 */
export function countCodeLines(source) {
  const lines = source.replace(/\r\n?/g, "\n").split("\n");
  let count = 0;
  let inBlockComment = false;
  let skipDepth = 0; // > 0 while inside a #[cfg(test)] item
  let pendingTestItem = false;
  for (const raw of lines) {
    let line = raw.trim();
    if (inBlockComment) {
      const end = line.indexOf("*/");
      if (end === -1) continue;
      inBlockComment = false;
      line = line.slice(end + 2).trim();
    }
    if (line.startsWith("/*")) {
      if (!line.includes("*/")) inBlockComment = true;
      continue;
    }
    if (line === "" || line.startsWith("//")) continue;
    if (skipDepth > 0 || pendingTestItem) {
      for (const ch of line) {
        if (ch === "{") {
          skipDepth += 1;
          pendingTestItem = false;
        } else if (ch === "}") {
          skipDepth -= 1;
        }
      }
      // A one-line item such as `#[cfg(test)] use x;` ends at its semicolon.
      if (pendingTestItem && line.endsWith(";")) pendingTestItem = false;
      continue;
    }
    if (line.startsWith("#[cfg(test)]")) {
      const rest = line.slice("#[cfg(test)]".length).trim();
      pendingTestItem = true;
      if (rest) {
        for (const ch of rest) {
          if (ch === "{") {
            skipDepth += 1;
            pendingTestItem = false;
          } else if (ch === "}") skipDepth -= 1;
        }
        if (pendingTestItem && rest.endsWith(";")) pendingTestItem = false;
      }
      continue;
    }
    count += 1;
  }
  return count;
}

function rustFiles(dir) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const full = path.join(dir, entry);
    if (statSync(full).isDirectory()) {
      if (entry === "tests" || entry === "fixtures" || entry === "snapshots") continue;
      out.push(...rustFiles(full));
    } else if (entry.endsWith(".rs")) {
      out.push(full);
    }
  }
  return out;
}

/** Measure every budget against the tree at `base`. */
export function measure(base = root, budgets = BUDGETS) {
  const results = [];
  for (const b of budgets) {
    const src = path.join(base, b.crate, "src");
    if (!existsSync(src)) {
      results.push({ crate: b.crate, status: "absent" });
      continue;
    }
    const files = rustFiles(src).map((f) => ({
      file: path.relative(path.join(base, b.crate), f).split(path.sep).join("/"),
      lines: countCodeLines(readFileSync(f, "utf8")),
    }));
    const total = files.reduce((n, f) => n + f.lines, 0);
    const over = [];
    if (b.total !== undefined && total > b.total) over.push(`${total} lines > ${b.total}`);
    if (b.perFile !== undefined) {
      const shared = new Set((b.shared ?? []).map((s) => `src/${s}`));
      let sharedLines = 0;
      for (const f of files) {
        if (shared.has(f.file)) sharedLines += f.lines;
        else if (f.lines > b.perFile) over.push(`${f.file}: ${f.lines} lines > ${b.perFile}`);
      }
      if (b.sharedTotal !== undefined && sharedLines > b.sharedTotal) {
        over.push(`shared modules: ${sharedLines} lines > ${b.sharedTotal}`);
      }
    }
    results.push({ crate: b.crate, status: over.length ? "over" : "ok", total, over, files });
  }
  return results;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const results = measure();
  if (process.argv.includes("--json")) {
    console.log(JSON.stringify(results, null, 2));
  } else {
    for (const r of results) {
      if (r.status === "absent") console.log(`-    ${r.crate}: not created yet`);
      else if (r.status === "ok") console.log(`ok   ${r.crate}: ${r.total} lines`);
      else for (const o of r.over) console.log(`OVER ${r.crate}: ${o}`);
    }
  }
  process.exit(results.some((r) => r.status === "over") ? 1 : 0);
}
