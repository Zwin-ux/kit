#!/usr/bin/env node
/**
 * Completeness inventory — zero npm deps.
 * Exit code is always 0. This is not a gate.
 */
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_EXT = new Set([
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".rs",
  ".py",
]);
const SKIP_DIRS = new Set([
  "node_modules",
  "target",
  "dist",
  "build",
  ".git",
  "vendor",
  ".kit",
  "coverage",
]);
const TREE_ROOTS = ["src", "lib", "app"];
const WEB_DEPS = [
  "react",
  "vue",
  "svelte",
  "next",
  "nuxt",
  "@angular/core",
  "solid-js",
];
const BUG_WORDS = /\b(fix|bug|error|fail|panic|regress)\b/i;

export function parseArgs(argv) {
  const out = { root: process.cwd(), json: false, write: true };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--json") out.json = true;
    else if (arg === "--no-write") out.write = false;
    else if (arg === "--root") {
      out.root = path.resolve(argv[i + 1] ?? process.cwd());
      i += 1;
    } else if (arg.startsWith("--root=")) {
      out.root = path.resolve(arg.slice("--root=".length));
    }
  }
  return out;
}

export function posixRel(root, file) {
  return path.relative(root, file).split(path.sep).join("/");
}

export function langFromExt(file) {
  const ext = path.extname(file).toLowerCase();
  if (ext === ".rs") return "rs";
  if (ext === ".py") return "py";
  if (SOURCE_EXT.has(ext)) return "js";
  return null;
}

export function isTestFile(rel) {
  const base = path.posix.basename(rel);
  if (/\.test\./i.test(base) || /\.spec\./i.test(base)) return true;
  if (/_test\.(rs|py)$/i.test(base)) return true;
  if (/^test_.*\.py$/i.test(base)) return true;
  const parts = rel.split("/");
  return parts.some((p) =>
    ["tests", "__tests__", "benches", "examples"].includes(p),
  );
}

export function isSourceFile(rel) {
  return SOURCE_EXT.has(path.extname(rel).toLowerCase()) && !isTestFile(rel);
}

function git(root, args) {
  const result = spawnSync("git", ["-C", root, ...args], {
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  if (result.status !== 0) return null;
  return result.stdout ?? "";
}

export function isGitRepo(root) {
  const out = git(root, ["rev-parse", "--is-inside-work-tree"]);
  return Boolean(out && out.trim() === "true");
}

export function gitChangedFiles(root) {
  const chunks = [
    git(root, ["diff", "--name-only"]),
    git(root, ["diff", "--cached", "--name-only"]),
    git(root, ["ls-files", "--others", "--exclude-standard"]),
  ];
  if (chunks.some((c) => c === null)) return [];
  const files = new Set();
  for (const chunk of chunks) {
    for (const line of chunk.split(/\r?\n/)) {
      if (line.trim()) files.add(line.trim().replace(/\\/g, "/"));
    }
  }
  return [...files];
}

function walkDir(abs, rel, files) {
  let entries;
  try {
    entries = readdirSync(abs, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry.name)) continue;
    const childAbs = path.join(abs, entry.name);
    const childRel = rel ? `${rel}/${entry.name}` : entry.name;
    if (entry.isDirectory()) walkDir(childAbs, childRel, files);
    else if (entry.isFile()) files.push(childRel);
  }
}

export function listTreeFiles(root) {
  const files = [];
  const roots = [];
  for (const name of TREE_ROOTS) {
    if (existsSync(path.join(root, name))) roots.push(name);
  }
  const crates = path.join(root, "crates");
  if (existsSync(crates)) {
    try {
      for (const entry of readdirSync(crates, { withFileTypes: true })) {
        if (!entry.isDirectory()) continue;
        const src = path.join(crates, entry.name, "src");
        if (existsSync(src)) roots.push(`crates/${entry.name}/src`);
      }
    } catch {
      /* skip */
    }
  }
  const apps = path.join(root, "apps");
  if (existsSync(apps)) {
    try {
      for (const entry of readdirSync(apps, { withFileTypes: true })) {
        if (!entry.isDirectory()) continue;
        const src = path.join(apps, entry.name, "src");
        if (existsSync(src)) roots.push(`apps/${entry.name}/src`);
      }
    } catch {
      /* skip */
    }
  }
  if (roots.length === 0) {
    walkDir(root, "", files);
    return files;
  }
  for (const rel of roots) walkDir(path.join(root, rel), rel, files);
  return files;
}

export function listAllFiles(root) {
  const files = [];
  walkDir(root, "", files);
  return files;
}

function stripComments(body, lang) {
  let text = body;
  if (lang === "py") {
    text = text.replace(/#.*$/gm, "");
  } else {
    text = text.replace(/\/\*[\s\S]*?\*\//g, " ");
    text = text.replace(/\/\/.*$/gm, "");
  }
  return text;
}

function significantLines(body, lang) {
  return stripComments(body, lang)
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0 && line !== "{" && line !== "}");
}

export function isStub(body, lang) {
  const lines = significantLines(body, lang);
  if (lines.length === 0) return { stub: true, reason: "empty body" };

  const joined = lines.join("\n");
  const only = lines.length === 1 ? lines[0] : null;

  if (lang === "rs") {
    if (lines.some((l) => /^(todo!|unimplemented!)\s*\(/.test(l))) {
      const hit = lines.find((l) => /^(todo!|unimplemented!)\s*\(/.test(l));
      return { stub: true, reason: hit };
    }
    if (
      only &&
      /^Ok\s*\(\s*\(\s*\)\s*\)\s*;?$/.test(only) &&
      /todo/i.test(body)
    ) {
      return { stub: true, reason: only };
    }
  }

  if (lang === "py") {
    if (only === "pass") return { stub: true, reason: "pass" };
    if (only && /^raise\s+NotImplementedError/.test(only)) {
      return { stub: true, reason: only };
    }
    if (only && /^return\s+None\s*$/.test(only)) {
      return { stub: true, reason: only };
    }
  }

  if (lang === "js") {
    if (only && /^throw\s+new\s+Error\s*\(/.test(only)) {
      return { stub: true, reason: only.replace(/;?\s*$/, "") };
    }
    if (
      only &&
      /^return\s+(null|undefined)\s*;?$/.test(only)
    ) {
      return { stub: true, reason: only.replace(/;?\s*$/, "") };
    }
    if (only && /^return\s*;$/.test(only)) {
      return { stub: true, reason: "return;" };
    }
  }

  if (
    lines.every((l) => /todo|fixme stub|not implemented/i.test(l)) &&
    lines.length <= 2
  ) {
    return { stub: true, reason: lines[0] };
  }

  const raw = body.trim();
  if (
    /^(?:\/\/|#|\s)*(?:TODO implement|FIXME stub)\s*$/im.test(raw) &&
    lines.length <= 1
  ) {
    return { stub: true, reason: "TODO/FIXME only" };
  }

  void joined;
  return { stub: false, reason: null };
}

function takeBraceBody(lines, startLine, startCol) {
  let depth = 0;
  let started = false;
  const collected = [];
  for (let i = startLine; i < lines.length && collected.length < 30; i += 1) {
    const line = lines[i];
    const from = i === startLine ? startCol : 0;
    let chunk = "";
    for (let c = from; c < line.length; c += 1) {
      const ch = line[c];
      if (ch === "{") {
        depth += 1;
        started = true;
      } else if (ch === "}") {
        depth -= 1;
      }
      if (started) chunk += ch;
      if (started && depth === 0) {
        collected.push(chunk);
        return collected.join("\n");
      }
    }
    if (started) collected.push(chunk || line.slice(from));
  }
  return collected.join("\n");
}

function takeIndentedBody(lines, defIndex, defIndent) {
  const collected = [];
  for (let i = defIndex + 1; i < lines.length && collected.length < 30; i += 1) {
    const line = lines[i];
    if (line.trim() === "") {
      collected.push(line);
      continue;
    }
    const indent = line.match(/^\s*/)[0].length;
    if (indent <= defIndent) break;
    collected.push(line);
  }
  return collected.join("\n");
}

const JS_FN =
  /^\s*export\s+(?:async\s+)?function\s+([A-Za-z_$][\w$]*)\s*\(/;
const JS_CONST =
  /^\s*export\s+(?:const|let)\s+([A-Za-z_$][\w$]*)\s*=\s*(.*)$/;
const JS_CLASS = /^\s*export\s+class\s+([A-Za-z_$][\w$]*)\b/;
const JS_EXPORTS =
  /^\s*(?:module\.)?exports\.([A-Za-z_$][\w$]*)\s*=\s*(.*)$/;
const JS_TYPE = /^\s*export\s+(?:type|interface)\s+/;
const JS_REEXPORT = /^\s*export\s+\{/;
const JS_METHOD =
  /^\s+(?:async\s+)?(?:get\s+|set\s+)?([A-Za-z_$][\w$]*)\s*\(/;

function looksLikeFnRhs(rhs) {
  const t = rhs.trim();
  return (
    /^(?:async\s*)?(?:function\b|\([^)]*\)\s*=>|[A-Za-z_$][\w$]*\s*=>)/.test(t)
  );
}

export function extractJsSymbols(file, source) {
  const lines = source.split(/\r?\n/);
  const symbols = [];
  let classIndent = null;

  const pushFn = (name, line, lineIndex, extra) => {
    let body = "";
    const braceAt = extra.indexOf("{");
    const arrow = extra.match(/=>\s*(.*)$/);
    if (braceAt >= 0) {
      body = takeBraceBody(lines, lineIndex, extra.indexOf("{"));
    } else if (arrow && !arrow[1].includes("{")) {
      body = arrow[1];
    } else {
      body = takeBraceBody(lines, lineIndex, 0);
    }
    symbols.push({ name, line, lang: "js", body });
  };

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    const indent = line.match(/^\s*/)[0].length;

    if (JS_TYPE.test(line) || JS_REEXPORT.test(line)) {
      classIndent = null;
      continue;
    }

    const fn = line.match(JS_FN);
    if (fn) {
      classIndent = null;
      pushFn(fn[1], i + 1, i, line);
      continue;
    }

    const klass = line.match(JS_CLASS);
    if (klass) {
      classIndent = indent;
      continue;
    }

    if (classIndent !== null && indent <= classIndent && line.trim()) {
      classIndent = null;
    }

    if (classIndent !== null && indent > classIndent) {
      const method = line.match(JS_METHOD);
      if (
        method &&
        !method[1].startsWith("_") &&
        method[1] !== "if" &&
        method[1] !== "for" &&
        method[1] !== "while" &&
        method[1] !== "switch" &&
        method[1] !== "catch"
      ) {
        pushFn(method[1], i + 1, i, line);
        continue;
      }
    }

    const cst = line.match(JS_CONST);
    if (cst && looksLikeFnRhs(cst[2])) {
      classIndent = null;
      pushFn(cst[1], i + 1, i, line);
      continue;
    }

    const exp = line.match(JS_EXPORTS);
    if (exp && looksLikeFnRhs(exp[2])) {
      classIndent = null;
      pushFn(exp[1], i + 1, i, line);
    }
  }
  return symbols;
}

const RS_FN = /^\s*pub\s+(?:async\s+)?fn\s+([A-Za-z_][\w]*)\s*[<(]/;

export function extractRsSymbols(file, source) {
  const lines = source.split(/\r?\n/);
  const symbols = [];
  let testDepth = 0;
  let braceDepth = 0;
  let pendingTest = false;

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (/#\[cfg\s*\(\s*test\s*\)\]/.test(line)) pendingTest = true;
    if (pendingTest && /^\s*mod\s+/.test(line)) {
      testDepth = braceDepth + 1;
      pendingTest = false;
    }

    const fn = line.match(RS_FN);
    if (fn && !/pub\s*\(\s*crate\s*\)/.test(line) && testDepth === 0) {
      const body = takeBraceBody(lines, i, line.indexOf("{") >= 0 ? line.indexOf("{") : 0);
      symbols.push({ name: fn[1], line: i + 1, lang: "rs", body });
    }

    for (const ch of line) {
      if (ch === "{") braceDepth += 1;
      else if (ch === "}") {
        braceDepth -= 1;
        if (testDepth > 0 && braceDepth < testDepth) testDepth = 0;
      }
    }
  }
  void file;
  return symbols;
}

const PY_DEF = /^(\s*)(?:async\s+)?def\s+([A-Za-z_][\w]*)\s*\(/;
const PY_CLASS = /^(\s*)class\s+([A-Za-z_][\w]*)\s*[:(]/;

export function extractPySymbols(file, source) {
  const lines = source.split(/\r?\n/);
  const symbols = [];
  let classIndent = null;

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (!line.trim() || line.trim().startsWith("#")) continue;
    const indent = line.match(/^\s*/)[0].length;

    const cls = line.match(PY_CLASS);
    if (cls && indent === 0) {
      classIndent = 0;
      continue;
    }

    const def = line.match(PY_DEF);
    if (!def) {
      if (classIndent !== null && indent === 0) classIndent = null;
      continue;
    }
    const name = def[2];
    if (name.startsWith("_")) continue;
    const defIndent = def[1].length;
    if (defIndent === 0) {
      const body = takeIndentedBody(lines, i, 0);
      symbols.push({ name, line: i + 1, lang: "py", body });
      continue;
    }
    if (classIndent !== null && defIndent > classIndent) {
      const body = takeIndentedBody(lines, i, defIndent);
      symbols.push({ name, line: i + 1, lang: "py", body });
    }
  }
  void file;
  return symbols;
}

export function extractSymbols(file, source) {
  const lang = langFromExt(file);
  if (lang === "js") return extractJsSymbols(file, source);
  if (lang === "rs") return extractRsSymbols(file, source);
  if (lang === "py") return extractPySymbols(file, source);
  return [];
}

export function wordMentioned(haystack, name) {
  const re = new RegExp(`\\b${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`);
  return re.test(haystack);
}

export function verdictFor(counts) {
  if (counts.public === 0) return "empty";
  if (counts.stub > 0) return "not_done";
  if (counts.untested > 0) return "partial";
  return "done";
}

function readJsonSafe(file) {
  try {
    return JSON.parse(readFileSync(file, "utf8"));
  } catch {
    return null;
  }
}

export function detectSignals(root) {
  const signals = {
    web: false,
    cli: false,
    library: false,
    data: false,
    bugTask: false,
  };

  const pkgPath = path.join(root, "package.json");
  if (existsSync(pkgPath)) {
    const pkg = readJsonSafe(pkgPath) ?? {};
    const deps = {
      ...(pkg.dependencies ?? {}),
      ...(pkg.devDependencies ?? {}),
      ...(pkg.peerDependencies ?? {}),
    };
    const names = Object.keys(deps);
    signals.web = WEB_DEPS.some((h) =>
      names.some((n) => n === h || n.startsWith(`${h}/`)),
    );
    signals.cli = Boolean(pkg.bin);
    signals.library =
      !signals.web &&
      (names.includes("typescript") || names.includes("@types/node"));
  }

  const cargoPath = path.join(root, "Cargo.toml");
  if (existsSync(cargoPath)) {
    const cargo = readFileSync(cargoPath, "utf8");
    if (/\[\[bin\]\]/.test(cargo)) signals.cli = true;
    if (/\[lib\]/.test(cargo) || !/\[\[bin\]\]/.test(cargo)) {
      signals.library = signals.library || !signals.web;
    }
  }

  if (
    existsSync(path.join(root, "notebooks")) ||
    (existsSync(path.join(root, "pyproject.toml")) &&
      existsSync(path.join(root, "data")))
  ) {
    signals.data = true;
  }

  const task = process.env.KIT_TASK ?? "";
  const subject = isGitRepo(root)
    ? (git(root, ["log", "-1", "--pretty=%s"]) ?? "")
    : "";
  signals.bugTask = BUG_WORDS.test(task) || BUG_WORDS.test(subject);
  return signals;
}

export function buildSkillLadder(counts, symbols, signals, verdict) {
  const ladder = [
    { skill: "completeness-qa", role: "current", reason: "inventory" },
  ];

  const add = (skill, reason) => {
    if (ladder.length >= 4) return;
    if (ladder.some((row) => row.skill === skill)) return;
    ladder.push({ skill, role: "next", reason });
  };

  if (counts.untested > 0 || counts.stub > 0) {
    add(
      "write-tests",
      counts.untested > 0
        ? `${counts.untested} untested public symbol${counts.untested === 1 ? "" : "s"}`
        : `${counts.stub} stubbed public symbol${counts.stub === 1 ? "" : "s"}`,
    );
  }

  const halfFix = symbols.some(
    (s) =>
      s.status === "stub" &&
      s.tested &&
      /unimplemented|not implemented/i.test(s.reason ?? ""),
  );
  if (signals.bugTask || halfFix) {
    add("fix-bug", halfFix ? "stub on a symbol that already has tests" : "task looks like a bugfix");
  }

  if (counts.public > 0) {
    add(
      "code-review",
      verdict === "done"
        ? "ship/no-ship on the diff"
        : "ship/no-ship after holes are closed",
    );
  }

  if (signals.web) add("a11y-pass", "UI stack detected");
  if (signals.cli) add("cli-help", "CLI binary detected");
  if (signals.library) add("api-docs", "library-shaped project");
  if (signals.data) add("data-check", "data/notebook layout");

  return ladder;
}

export function formatReport(report) {
  const { counts, verdict, testNext, skillLadder } = report;
  const verdictLine =
    verdict === "done"
      ? "DONE — public surface is implemented and mentioned in tests"
      : verdict === "partial"
        ? `PARTIAL — ${counts.untested} public function${counts.untested === 1 ? " is" : "s are"} untested`
        : verdict === "empty"
          ? "EMPTY — no public symbols in scan set"
          : `NOT DONE — ${counts.stub} public function${counts.stub === 1 ? " is a stub" : "s are stubs"}`;

  const lines = [
    "KIT completeness-qa",
    "",
    `Surface    ${counts.public} public  ·  ${counts.stub} stub  ·  ${counts.untested} untested  ·  ${counts.ok} ok`,
    `Verdict    ${verdictLine}`,
  ];

  const printed = testNext.slice(0, 5);
  if (printed.length > 0) {
    lines.push("", "Test next");
    printed.forEach((row, i) => {
      const tag =
        row.status === "stub" && !row.tested
          ? "stub + untested"
          : row.status;
      lines.push(`  ${i + 1}. ${row.file}:${row.name}   ${tag}`);
    });
  }

  lines.push("", "Skill ladder");
  skillLadder.slice(0, 4).forEach((row, i) => {
    const note = row.role === "current" ? "you are here" : row.reason;
    lines.push(`  ${i + 1}. ${row.skill.padEnd(18)} ${note}`);
  });

  const next = skillLadder.find((row) => row.role === "next");
  const nextText =
    verdict === "empty"
      ? "nothing to QA in the scan set"
      : next
        ? next.skill === "write-tests" && printed.length
          ? `load write-tests and cover ${printed
              .slice(0, 2)
              .map((r) => r.name)
              .join(", ")}`
          : `load ${next.skill}`
        : "no follow-on skill";
  lines.push("", `Next    ${nextText}`, "");
  return lines.join("\n");
}

export function inventory(root, options = {}) {
  const warnings = [];
  const absRoot = path.resolve(root);
  const inGit = isGitRepo(absRoot);
  let mode = "tree";
  let sourceRels = [];

  if (inGit) {
    const changed = gitChangedFiles(absRoot).filter(isSourceFile);
    if (changed.length > 0) {
      mode = "diff";
      sourceRels = changed;
    }
  }
  if (mode === "tree") {
    sourceRels = listTreeFiles(absRoot).filter(isSourceFile);
  }

  const allRels = listAllFiles(absRoot);
  const testRels = allRels.filter(
    (rel) => SOURCE_EXT.has(path.extname(rel).toLowerCase()) && isTestFile(rel),
  );
  let testHaystack = "";
  for (const rel of testRels) {
    try {
      testHaystack += `\n${readFileSync(path.join(absRoot, rel), "utf8")}`;
    } catch {
      warnings.push(rel);
    }
  }

  const symbols = [];
  let scannedFiles = 0;
  for (const rel of sourceRels) {
    const abs = path.join(absRoot, rel);
    let source;
    try {
      if (!existsSync(abs) || !statSync(abs).isFile()) continue;
      source = readFileSync(abs, "utf8");
    } catch {
      warnings.push(rel);
      continue;
    }
    scannedFiles += 1;
    const extracted = extractSymbols(rel, source);
    for (const sym of extracted) {
      const stub = isStub(sym.body, sym.lang);
      const tested = wordMentioned(testHaystack, sym.name);
      const status = stub.stub ? "stub" : tested ? "ok" : "untested";
      symbols.push({
        file: rel.replace(/\\/g, "/"),
        line: sym.line,
        name: sym.name,
        lang: sym.lang,
        status,
        reason: stub.reason,
        tested,
      });
    }
  }

  const counts = {
    public: symbols.length,
    stub: symbols.filter((s) => s.status === "stub").length,
    untested: symbols.filter((s) => s.status === "untested").length,
    ok: symbols.filter((s) => s.status === "ok").length,
  };
  const verdict = verdictFor(counts);
  const testNext = [...symbols]
    .filter((s) => s.status === "stub" || s.status === "untested")
    .sort((a, b) => {
      if (a.status !== b.status) return a.status === "stub" ? -1 : 1;
      return a.file.localeCompare(b.file) || a.name.localeCompare(b.name);
    })
    .slice(0, 10)
    .map((s) => ({
      file: s.file,
      name: s.name,
      status: s.status,
      tested: s.tested,
    }));

  const signals = detectSignals(absRoot);
  const skillLadder = buildSkillLadder(counts, symbols, signals, verdict);

  return {
    version: 1,
    root: absRoot,
    mode,
    scannedFiles,
    counts,
    verdict,
    symbols,
    testNext,
    skillLadder,
    warnings,
  };
}

export function run(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const report = inventory(args.root);
  if (args.write) {
    try {
      const dir = path.join(args.root, ".kit");
      mkdirSync(dir, { recursive: true });
      writeFileSync(
        path.join(dir, "completeness.json"),
        `${JSON.stringify(report, null, 2)}\n`,
        "utf8",
      );
    } catch {
      report.warnings.push(".kit/ not writable");
    }
  }
  if (args.json) {
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  } else {
    process.stdout.write(formatReport(report));
  }
  return 0;
}

const invoked = process.argv[1]
  ? path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
  : false;
if (invoked) {
  process.exitCode = run();
}
