#!/usr/bin/env node
import {
  applyPack,
  completeFirstRun,
  describePaths,
  exploreListPacks,
  exploreListSkills,
  exploreSearch,
  exploreShowPack,
  formatIssues,
  getFirstRunStatus,
  getLoggedInUser,
  getRegistryUrl,
  installPack,
  installSkill,
  isFirstRunPackName,
  importSkillsFromHarness,
  linkSkills,
  listFirstRunPackOptions,
  listPacks,
  listSkills,
  loadPack,
  loadSkill,
  loginWithDeviceFlow,
  logout,
  recommendToolkits,
  removeSkill,
  runDoctor,
  runReady,
  runUnify,
  detectSituation,
  testAllPacks,
  testPack,
  testSkill,
  validatePack,
  type CheckResult,
  type HarnessId,
  type LinkMode,
  type PathScope,
} from "@mzwin/kit-core";
import { KIT_PACKAGE_VERSION } from "@mzwin/kit-shared";
import { normalizeArgv } from "./argv.js";

const args = normalizeArgv(process.argv.slice(2));
const command = args[0];

/** Exit 1 = user/validation; 2 = unexpected (handled in main catch). */
function fail(message: string, code = 1): never {
  console.error(message);
  process.exit(code);
}

async function main(): Promise<void> {
  if (command === "--help" || command === "-h" || command === "help") {
    printHelp();
    process.exit(0);
  }

  if (command === undefined || command === "home") {
    await runHome(args.slice(command === "home" ? 1 : 0));
    return;
  }

  if (command === "--version" || command === "-v" || command === "version") {
    console.log(KIT_PACKAGE_VERSION);
    process.exit(0);
  }

  if (command === "ready") {
    await runReadyCmd(args.slice(1));
    return;
  }

  if (command === "init") {
    await runInit(args.slice(1));
    return;
  }

  if (command === "validate") {
    await runValidate(args.slice(1));
    return;
  }

  if (command === "install") {
    await runInstall(args.slice(1));
    return;
  }

  if (command === "list" || command === "ls") {
    await runList();
    return;
  }

  if (command === "remove" || command === "rm" || command === "uninstall") {
    await runRemove(args.slice(1));
    return;
  }

  if (command === "pack") {
    await runPack(args.slice(1));
    return;
  }

  if (command === "paths") {
    await runPaths(args.slice(1));
    return;
  }

  if (command === "link") {
    await runLink(args.slice(1));
    return;
  }

  if (command === "import") {
    await runImport(args.slice(1));
    return;
  }

  if (command === "unify") {
    await runUnifyCmd(args.slice(1));
    return;
  }

  if (command === "test") {
    await runTest(args.slice(1));
    return;
  }

  if (command === "doctor") {
    await runDoctorCmd(args.slice(1));
    return;
  }

  if (command === "login") {
    await runLogin(args.slice(1));
    return;
  }

  if (command === "logout") {
    await runLogout();
    return;
  }

  if (command === "whoami") {
    await runWhoami();
    return;
  }

  if (command === "explore") {
    await runExplore(args.slice(1));
    return;
  }

  if (command === "recommend") {
    await runRecommend(args.slice(1));
    return;
  }

  if (command === "tui" || command === "ui" || command === "start") {
    const { startTui } = await import("@mzwin/kit-tui");
    startTui();
    return;
  }

  fail(`kit: unknown command: ${command}\nRun \`kit --help\` for usage.`);
}

function printHelp(): void {
  console.log(`kit ${KIT_PACKAGE_VERSION}`);
  console.log("");
  console.log("One library. Many agents. Built for the vibe-coding boom.");
  console.log("");
  console.log("Start here:");
  console.log("  kit                         # your situation + next move");
  console.log("  kit ready --write           # make THIS repo agent-ready");
  console.log("  kit unify --write --link    # clean skill mess → portable library");
  console.log("  kit tui                     # keyboard console (interactive terminal)");
  console.log("");
  console.log("Everyday:");
  console.log("  kit recommend --dir .");
  console.log("  kit init --pack essentials");
  console.log("  kit pack list | install | apply");
  console.log("  kit link --to claude-code|codex|grok-build|all --write");
  console.log("  kit import --from claude-code --write");
  console.log("  kit unify [--write] [--link] [--all] [--json]");
  console.log("  kit ready [--write] [--unify] [--pack <name>] [--dir <path>]");
  console.log("  kit doctor | kit list");
  console.log("");
  console.log("Also: validate, install, remove, paths, test, login, explore");
  console.log("");
  console.log("From this monorepo:");
  console.log("  pnpm build");
  console.log("  pnpm kit tui                # preferred (no extra -- needed)");
  console.log("  pnpm kit -- tui             # also works (npm-style pass-through)");
  console.log("  pnpm tui                    # same as kit tui");
  console.log("");
  console.log("Install:  npm i -g @mzwin/kit");
  console.log("Docs:     https://github.com/Zwin-ux/kit");
}

async function runInit(rest: string[]): Promise<void> {
  const apply = rest.includes("--apply");
  const skip = rest.includes("--skip");
  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit init [--pack <name>] [--apply] [--dir <path>] [--skip]");
    }
  }

  const packFlag = rest.indexOf("--pack");
  let packName = "essentials";
  if (packFlag >= 0) {
    const value = rest[packFlag + 1];
    if (!value || value.startsWith("-")) {
      fail(
        "Usage: kit init [--pack essentials|web-app|library|cli-tool|api-service|full-stack|data-ml]",
      );
    }
    packName = value;
  }

  if (skip) {
    await completeFirstRun("skipped");
    console.log("First-run skipped. You can run `kit init` again later.");
    console.log("Or open the TUI: kit tui");
    return;
  }

  if (!isFirstRunPackName(packName)) {
    const names = listFirstRunPackOptions()
      .map((p) => p.name)
      .join(", ");
    fail(`Unknown pack "${packName}". Choose one of: ${names}`);
  }

  const status = await getFirstRunStatus();
  if (!status.shouldOffer && status.skillCount > 0) {
    console.log(
      `Library already has ${status.skillCount} skill(s). Installing pack anyway.`,
    );
  }

  console.log(`Initializing with pack: ${packName}`);
  console.log("");

  if (apply) {
    const result = await applyPack(packName, {
      force: true,
      ...(projectDir ? { projectDir } : {}),
    });
    if (!result.ok) fail(result.error);
    printApplySuccess(result.value);
  } else {
    const result = await installPack(packName, { force: true });
    if (!result.ok) fail(result.error);
    printInstallSuccess(result.value);
  }

  await completeFirstRun("installed", { preferredPack: packName });
  console.log("");
  console.log("First-run complete.");
  console.log("Next: kit list · kit tui · kit pack apply " + packName + " --dir .");
}

function printInstallSuccess(value: {
  pack: { name: string; version: string; title: string };
  installed: { name: string; version: string }[];
  skipped: string[];
}): void {
  console.log(`Installed pack ${value.pack.name}@${value.pack.version}`);
  console.log(`  ${value.pack.title}`);
  console.log(`  ${value.installed.length} skill(s) in your library`);
  for (const skill of value.installed) {
    console.log(`  + ${skill.name}@${skill.version}`);
  }
  if (value.skipped.length > 0) {
    console.log(`  skipped: ${value.skipped.join(", ")}`);
  }
  console.log("");
  console.log("What now");
  console.log("  kit list");
  console.log("  kit pack apply " + value.pack.name + " --dir .");
  console.log("  kit tui");
}

function printApplySuccess(value: {
  pack: { name: string; version: string; title: string };
  projectDir: string;
  projectSkillsDir: string;
  appliedPath: string;
  installed: { name: string; version: string }[];
  gitWarning?: string;
  reapplied?: boolean;
  versionChanged?: boolean;
}): void {
  const mode =
    value.reapplied && value.versionChanged
      ? "Updated"
      : value.reapplied
        ? "Reapplied"
        : "Applied";
  console.log(
    `${mode} pack ${value.pack.name}@${value.pack.version}`,
  );
  console.log(`  ${value.pack.title}`);
  console.log(`  project: ${value.projectDir}`);
  console.log(`  skills:  ${value.projectSkillsDir}`);
  console.log(`  record:  ${value.appliedPath}`);
  console.log(`  ${value.installed.length} skill(s) synced to library`);
  for (const skill of value.installed) {
    console.log(`  + ${skill.name}@${skill.version}`);
  }
  if (value.gitWarning) {
    console.log("");
    console.log(`Note: ${value.gitWarning}`);
  }
  console.log("");
  console.log("What now");
  console.log("  kit paths");
  console.log("  kit link --to claude-code --write   # dry-run without --write");
  console.log("  kit list · kit tui");
}

async function runValidate(rest: string[]): Promise<void> {
  const skillDir = rest[0];
  if (!skillDir) fail("Usage: kit validate <skill-dir>");

  const result = await loadSkill(skillDir);
  if (!result.ok) {
    fail(`Validation failed:\n${formatIssues(result.issues)}`);
  }

  console.log(`OK  ${result.skill.name}@${result.skill.version}`);
  console.log(`    ${result.skill.description}`);
  console.log(`    agents: ${result.skill.compatibility.join(", ")}`);
}

async function runInstall(rest: string[]): Promise<void> {
  const force = rest.includes("--force");
  const skillDir = rest.find((arg) => !arg.startsWith("-"));
  if (!skillDir) fail("Usage: kit install <skill-dir> [--force]");

  const result = await installSkill(skillDir, { force });
  if (!result.ok) fail(result.error);

  console.log(`Installed ${result.value.name}@${result.value.version}`);
  console.log(`  path: ${result.value.installPath}`);
}

async function runList(): Promise<void> {
  const result = await listSkills();
  if (!result.ok) fail(result.error);

  if (result.value.length === 0) {
    console.log("No skills installed.");
    console.log("Next: kit init   or   kit pack install essentials");
    return;
  }

  for (const skill of result.value) {
    console.log(`${skill.name}@${skill.version}`);
    console.log(`  ${skill.description}`);
    console.log(`  ${skill.installPath}`);
  }
}

async function runRemove(rest: string[]): Promise<void> {
  const name = rest[0];
  if (!name) fail("Usage: kit remove <skill-name>");

  const result = await removeSkill(name);
  if (!result.ok) fail(result.error);
  console.log(`Removed ${result.value.name}`);
}

async function runPack(rest: string[]): Promise<void> {
  const sub = rest[0];
  if (!sub || sub === "--help" || sub === "-h") {
    printPackHelp();
    process.exit(0);
  }

  if (sub === "list" || sub === "ls") {
    const result = await listPacks();
    if (!result.ok) fail(result.error);
    if (result.value.length === 0) {
      console.log("No packs found.");
      process.exit(0);
    }
    for (const pack of result.value) {
      console.log(`${pack.name}@${pack.version}  (${pack.skillCount} skills)`);
      console.log(`  ${pack.title} — ${pack.description}`);
      if (pack.projectTypes.length > 0) {
        console.log(`  project types: ${pack.projectTypes.join(", ")}`);
      }
    }
    return;
  }

  if (sub === "show") {
    const name = rest[1];
    if (!name) fail("Usage: kit pack show <pack>");
    const result = await loadPack(name);
    if (!result.ok) fail(result.error);
    const { pack, skills } = result.value;
    console.log(`${pack.name}@${pack.version}`);
    console.log(pack.title);
    console.log(pack.description);
    console.log("");
    console.log("Skills:");
    for (const skill of skills) {
      console.log(
        `  - ${skill.name}@${skill.skill.version} (${skill.origin})`,
      );
      console.log(`    ${skill.skill.description}`);
    }
    return;
  }

  if (sub === "validate") {
    const name = rest[1];
    if (!name) fail("Usage: kit pack validate <pack>");
    const result = await validatePack(name);
    if (!result.ok) fail(result.error);
    console.log(
      `OK  pack ${result.value.pack.name}@${result.value.pack.version}`,
    );
    console.log(`    ${result.value.skills.length} skills valid`);
    return;
  }

  if (sub === "install") {
    const packName = rest.slice(1).find((arg) => !arg.startsWith("-"));
    if (!packName) fail("Usage: kit pack install <pack>");
    const installResult = await installPack(packName, { force: true });
    if (!installResult.ok) fail(installResult.error);
    printInstallSuccess(installResult.value);
    return;
  }

  if (sub === "apply") {
    const dirFlag = rest.indexOf("--dir");
    let projectDir: string | undefined;
    if (dirFlag >= 0) {
      projectDir = rest[dirFlag + 1];
      if (!projectDir || projectDir.startsWith("-")) {
        fail("Usage: kit pack apply <pack> [--dir <project>]");
      }
    }
    const packName = rest
      .slice(1)
      .filter((arg, i, arr) => {
        if (arg.startsWith("-")) return false;
        if (i > 0 && arr[i - 1] === "--dir") return false;
        return true;
      })[0];
    if (!packName) fail("Usage: kit pack apply <pack> [--dir <project>]");
    const result = await applyPack(packName, {
      ...(projectDir ? { projectDir } : {}),
      force: true,
    });
    if (!result.ok) fail(result.error);
    printApplySuccess(result.value);
    return;
  }

  fail(`kit pack: unknown command: ${sub}`);
}

function printPackHelp(): void {
  console.log("Usage:");
  console.log("  kit pack list");
  console.log("  kit pack show <pack>");
  console.log("  kit pack validate <pack>");
  console.log("  kit pack install <pack>");
  console.log("  kit pack apply <pack> [--dir <project>]");
}

async function runPaths(rest: string[]): Promise<void> {
  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit paths [--dir <project>] [--skill <name>]");
    }
  }

  const skillFlag = rest.indexOf("--skill");
  let skillName: string | undefined;
  if (skillFlag >= 0) {
    skillName = rest[skillFlag + 1];
    if (!skillName || skillName.startsWith("-")) {
      fail("Usage: kit paths [--dir <project>] [--skill <name>]");
    }
  }

  const result = await describePaths({
    ...(projectDir ? { projectDir } : {}),
    ...(skillName ? { skillName } : {}),
  });
  if (!result.ok) fail(result.error);

  const report = result.value;
  console.log(`Kit home:     ${report.kitHome}`);
  console.log(`Project:      ${report.projectDir}`);
  console.log(`Global lib:   ${report.globalLibrary}`);
  console.log(`Project lib:  ${report.projectLibrary}`);
  if (report.skillName) {
    console.log(`Skill focus:  ${report.skillName}`);
  }
  console.log(
    `Installed:    ${
      report.installedSkillNames.length === 0
        ? "(none)"
        : report.installedSkillNames.join(", ")
    }`,
  );
  console.log("");
  console.log("Harness skill roots");
  for (const entry of report.entries) {
    const mark = entry.exists ? "✓" : "·";
    console.log(
      `${mark} ${entry.harness.padEnd(12)} ${entry.scope.padEnd(9)} ${entry.skillsRoot}`,
    );
    if (entry.skillDir) {
      console.log(`    skill → ${entry.skillDir}`);
    }
    console.log(`    ${entry.notes}`);
  }
  console.log("");
  console.log("Next: kit link --to claude-code          # dry-run");
  console.log("      kit link --to claude-code --write  # create links");
}

async function runLink(rest: string[]): Promise<void> {
  if (rest.includes("--help") || rest.includes("-h")) {
    console.log(
      "Usage: kit link [--to claude-code|codex|grok-build|all] [--scope personal|project] [--mode symlink|copy] [--dir <project>] [--write] [--force] [--skill <name>]",
    );
    console.log("");
    console.log("Default is a dry-run. Pass --write to create links or copies.");
    return;
  }

  const write = rest.includes("--write");
  const force = rest.includes("--force");
  const toFlag = rest.indexOf("--to");
  let harnesses: HarnessId[] | undefined;
  if (toFlag >= 0) {
    const value = rest[toFlag + 1];
    if (!value || value.startsWith("-")) {
      fail("Usage: kit link --to <claude-code|codex|grok-build|all>");
    }
    if (value === "all") {
      harnesses = ["claude-code", "codex", "grok-build"];
    } else if (
      value === "claude-code" ||
      value === "codex" ||
      value === "grok-build"
    ) {
      harnesses = [value];
    } else {
      fail(`Unknown harness "${value}". Use claude-code, codex, grok-build, or all.`);
    }
  }

  const scopeFlag = rest.indexOf("--scope");
  let scope: PathScope = "project";
  if (scopeFlag >= 0) {
    const value = rest[scopeFlag + 1];
    if (value !== "personal" && value !== "project") {
      fail("Usage: kit link --scope personal|project");
    }
    scope = value;
  }

  const modeFlag = rest.indexOf("--mode");
  let mode: LinkMode = "symlink";
  if (modeFlag >= 0) {
    const value = rest[modeFlag + 1];
    if (value !== "symlink" && value !== "copy") {
      fail("Usage: kit link --mode symlink|copy");
    }
    mode = value;
  }

  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit link --dir <project>");
    }
  }

  const skillFlag = rest.indexOf("--skill");
  let skillNames: string[] | undefined;
  if (skillFlag >= 0) {
    const value = rest[skillFlag + 1];
    if (!value || value.startsWith("-")) {
      fail("Usage: kit link --skill <name>");
    }
    skillNames = [value];
  }

  const result = await linkSkills({
    write,
    force,
    scope,
    mode,
    ...(harnesses ? { harnesses } : {}),
    ...(projectDir ? { projectDir } : {}),
    ...(skillNames ? { skillNames } : {}),
  });
  if (!result.ok) fail(result.error);

  const plan = result.value;
  console.log(plan.dryRun ? "Link plan (dry-run)" : "Link result");
  console.log(`  scope:  ${scope}`);
  console.log(`  mode:   ${mode}`);
  console.log(`  linked: ${plan.linked}  skipped: ${plan.skipped}  failed: ${plan.failed.length}`);
  console.log("");

  for (const item of plan.items) {
    const tag =
      item.action === "create"
        ? "+"
        : item.action === "replace"
          ? "!"
          : item.action === "skip-same"
            ? "="
            : "~";
    console.log(
      `${tag} ${item.harness} ${item.skillName}`,
    );
    console.log(`    ${item.sourceDir}`);
    console.log(`    → ${item.targetDir}`);
    if (item.reason) {
      console.log(`    ${item.reason}`);
    }
  }

  if (plan.failed.length > 0) {
    console.log("");
    console.log("Failures:");
    for (const f of plan.failed) {
      console.log(`  ${f.skillName}: ${f.error}`);
    }
  }

  if (plan.dryRun) {
    console.log("");
    console.log("No files written. Re-run with --write to apply.");
  }
}

async function runImport(rest: string[]): Promise<void> {
  if (rest.includes("--help") || rest.includes("-h")) {
    console.log(
      "Usage: kit import [--from claude-code|codex|grok-build|all] [--scope personal|project] [--dir <project>] [--skill <name>] [--write] [--force]",
    );
    console.log("");
    console.log("Capture skills from agent harness folders into the Kit library.");
    console.log("Default is a dry-run. Pass --write to install into ~/.kit.");
    return;
  }

  const write = rest.includes("--write");
  const force = rest.includes("--force");

  const fromFlag = rest.indexOf("--from");
  let harnesses: HarnessId[] | undefined;
  if (fromFlag >= 0) {
    const value = rest[fromFlag + 1];
    if (!value || value.startsWith("-")) {
      fail("Usage: kit import --from <claude-code|codex|grok-build|all>");
    }
    if (value === "all") {
      harnesses = ["claude-code", "codex", "grok-build"];
    } else if (
      value === "claude-code" ||
      value === "codex" ||
      value === "grok-build"
    ) {
      harnesses = [value];
    } else {
      fail(
        `Unknown harness "${value}". Use claude-code, codex, grok-build, or all.`,
      );
    }
  }

  const scopeFlag = rest.indexOf("--scope");
  let scope: PathScope = "personal";
  if (scopeFlag >= 0) {
    const value = rest[scopeFlag + 1];
    if (value !== "personal" && value !== "project") {
      fail("Usage: kit import --scope personal|project");
    }
    scope = value;
  }

  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit import --dir <project>");
    }
  }

  const skillFlag = rest.indexOf("--skill");
  let skillNames: string[] | undefined;
  if (skillFlag >= 0) {
    const value = rest[skillFlag + 1];
    if (!value || value.startsWith("-")) {
      fail("Usage: kit import --skill <name>");
    }
    skillNames = [value];
  }

  const result = await importSkillsFromHarness({
    write,
    force,
    scope,
    ...(harnesses ? { harnesses } : {}),
    ...(projectDir ? { projectDir } : {}),
    ...(skillNames ? { skillNames } : {}),
  });
  if (!result.ok) fail(result.error);

  const plan = result.value;
  console.log(plan.dryRun ? "Import plan (dry-run)" : "Import result");
  console.log(`  scope:    ${scope}`);
  console.log(
    `  imported: ${plan.imported}  skipped: ${plan.skipped}  failed: ${plan.failed.length}`,
  );
  console.log("");

  for (const item of plan.items) {
    if (item.action === "skip-missing-root") {
      console.log(`· ${item.harness} (no skills root)`);
      if (item.reason) console.log(`    ${item.reason}`);
      continue;
    }
    const tag =
      item.action === "import"
        ? "+"
        : item.action === "replace"
          ? "!"
          : item.action === "skip-exists"
            ? "="
            : "~";
    console.log(`${tag} ${item.harness} ${item.skillName}`);
    console.log(`    ${item.sourceDir}`);
    if (item.reason) console.log(`    ${item.reason}`);
  }

  if (plan.failed.length > 0) {
    console.log("");
    console.log("Failures:");
    for (const f of plan.failed) {
      console.log(`  ${f.skillName}: ${f.error}`);
    }
  }

  if (plan.dryRun) {
    console.log("");
    console.log("No files written. Re-run with --write to install into ~/.kit.");
  }
}

async function runHome(rest: string[]): Promise<void> {
  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit [--dir <project>]");
    }
  }

  const sit = await detectSituation({
    ...(projectDir ? { projectDir } : {}),
  });
  const { story, snapshot, headline } = sit;

  console.log("");
  console.log(`kit ${KIT_PACKAGE_VERSION}`);
  console.log("");
  console.log(`  ${headline}`);
  console.log("");
  console.log(
    `  library  ${snapshot.libraryCount}    agents  ~${snapshot.harnessSkillEstimate}`,
  );
  if (snapshot.recommendedPack) {
    console.log(`  project  ${snapshot.recommendedPack}`);
  }
  console.log("");
  console.log(`  →  ${story.primary}`);
  for (const n of story.next.slice(0, 2)) {
    console.log(`     ${n}`);
  }
  console.log("");
}

async function runReadyCmd(rest: string[]): Promise<void> {
  if (rest.includes("--help") || rest.includes("-h")) {
    console.log(
      "Usage: kit ready [--dir <project>] [--pack <name>] [--write] [--unify] [--force]",
    );
    console.log("");
    console.log("Make THIS repo agent-ready in one shot:");
    console.log("  recommend pack → install → apply → (optional unify) → link → doctor");
    console.log("");
    console.log("  kit ready                # dry-run plan");
    console.log("  kit ready --write        # do it");
    console.log("  kit ready --write --unify  # also clean personal skill dumps");
    return;
  }

  const write = rest.includes("--write");
  const unify = rest.includes("--unify");
  const force = rest.includes("--force");

  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit ready --dir <project>");
    }
  }

  const packFlag = rest.indexOf("--pack");
  let pack: string | undefined;
  if (packFlag >= 0) {
    pack = rest[packFlag + 1];
    if (!pack || pack.startsWith("-")) fail("Usage: kit ready --pack <name>");
  }

  const result = await runReady({
    write,
    unify,
    force,
    onProgress: (msg: string) => {
      process.stderr.write(`  … ${msg}\n`);
    },
    ...(projectDir ? { projectDir } : {}),
    ...(pack ? { pack } : {}),
  });
  if (!result.ok) fail(result.error);

  const r = result.value;
  console.log("");
  console.log(r.dryRun ? "READY" : "READY  ✓");
  console.log("");
  console.log(`  ${r.projectDir}`);
  console.log(`  pack     ${r.packName}`);
  console.log(`  ${r.recommendSummary}`);
  console.log("");
  for (const s of r.steps) {
    const mark =
      s.status === "done"
        ? "✓"
        : s.status === "planned"
          ? "·"
          : s.status === "skipped"
            ? "-"
            : "✗";
    console.log(`  ${mark}  ${s.detail}`);
  }
  if (r.dryRun) {
    console.log("");
    console.log("  →  kit ready --write");
  } else if (r.doctorOk) {
    console.log("");
    console.log("  →  open claude / codex here   ·   kit tui");
  }
  console.log("");
}

async function runUnifyCmd(rest: string[]): Promise<void> {
  if (rest.includes("--help") || rest.includes("-h")) {
    console.log(
      "Usage: kit unify [--dir <project>] [--write] [--link] [--force] [--top <n>] [--min-score <n>] [--all] [--json]",
    );
    console.log("");
    console.log("Your agents already have skills. They're a mess.");
    console.log("kit unify → one portable library every agent can use.");
    console.log("");
    console.log("  kit unify                 # show mess vs keepers (safe)");
    console.log("  kit unify --write         # adopt S/A keepers only");
    console.log("  kit unify --write --link  # adopt + project harness links");
    console.log("  kit unify --all           # include automation noise (not recommended)");
    return;
  }

  const write = rest.includes("--write");
  const link = rest.includes("--link");
  const force = rest.includes("--force");
  const includeNoise = rest.includes("--all");
  const asJson = rest.includes("--json");

  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit unify --dir <project>");
    }
  }

  const topFlag = rest.indexOf("--top");
  let top: number | undefined;
  if (topFlag >= 0) {
    const raw = rest[topFlag + 1];
    const n = Number(raw);
    if (!raw || Number.isNaN(n) || n < 1) fail("Usage: kit unify --top <positive number>");
    top = n;
  }

  const minFlag = rest.indexOf("--min-score");
  let minScore: number | undefined;
  if (minFlag >= 0) {
    const raw = rest[minFlag + 1];
    const n = Number(raw);
    if (!raw || Number.isNaN(n)) fail("Usage: kit unify --min-score <number>");
    minScore = n;
  }

  const result = await runUnify({
    write,
    link,
    force,
    includeNoise,
    ...(asJson
      ? {}
      : {
          onProgress: (msg: string) => {
            process.stderr.write(`  … ${msg}\n`);
          },
        }),
    ...(projectDir ? { projectDir } : {}),
    ...(top !== undefined ? { top } : {}),
    ...(minScore !== undefined ? { minScore } : {}),
  });
  if (!result.ok) fail(result.error);

  const report = result.value;
  if (asJson) {
    console.log(JSON.stringify(report, null, 2));
    return;
  }

  console.log("");
  console.log(report.dryRun ? "UNIFY  skill OS  (dry-run)" : "UNIFY  skill OS  (applied)");
  console.log("");
  console.log(`  Scanned   ${report.scanned} skill folders  (Claude · Codex · Grok · Kit)`);
  console.log(`  Unique    ${report.unique}`);
  console.log(
    `  Noise     ${report.noiseCount} filtered${report.includeNoise ? " (showing anyway: --all)" : ""}`,
  );
  console.log(
    `  Keepers   ${report.keeperCount}  (grade S/A · real structure or multi-agent)`,
  );
  console.log(
    `  Library   ${report.alreadyInLibrary} already in ~/.kit · ${report.adoptReady} ready to adopt`,
  );
  if (!report.dryRun) {
    console.log(`  Adopted   ${report.adopted}  ·  linked ${report.linked}`);
  }
  console.log("");

  const show = report.keepers.slice(0, 15);
  if (show.length === 0) {
    console.log("  No keepers found yet. Install a pack or loosen filters with --min-score 55.");
  } else {
    console.log("  Top keepers");
    for (const c of show) {
      const agents = [
        ...new Set(
          c.sources
            .map((s) => s.harness)
            .filter((h) => h === "claude-code" || h === "codex" || h === "grok-build"),
        ),
      ]
        .map((h) => (h === "claude-code" ? "claude" : h === "grok-build" ? "grok" : h))
        .join("+");
      const mark = c.inLibrary ? "·" : "+";
      const desc =
        c.description.length > 52
          ? `${c.description.slice(0, 49)}…`
          : c.description;
      console.log(
        `  ${mark} ${c.grade}  ${String(c.score).padStart(3)}  ${c.name.padEnd(22)}  ${(agents || "kit").padEnd(14)}  ${desc}`,
      );
    }
    if (report.keepers.length > 15) {
      console.log(`  … ${report.keepers.length - 15} more keepers`);
    }
  }

  if (report.noiseSample.length > 0 && !report.includeNoise) {
    console.log("");
    console.log("  Noise sample (filtered)");
    for (const n of report.noiseSample.slice(0, 3)) {
      console.log(
        `  × ${n.name.padEnd(28)}  ${n.noiseReasons[0] ?? "noise"}`,
      );
    }
  }

  if (report.notes.length > 0) {
    console.log("");
    for (const n of report.notes) console.log(`  · ${n}`);
  }

  if (report.dryRun) {
    console.log("");
    console.log(
      `  Safe default: adopt up to ${Math.min(top ?? 25, report.adoptReady)} keepers — not ${report.noiseCount} noise skills.`,
    );
    console.log("");
    console.log("  Why this matters");
    console.log("    Vibe coders install skills for every tool. They rot in one agent.");
    console.log("    Unify makes a single library Claude, Codex, and Grok can all share.");
    console.log("");
    console.log("  Next");
    console.log("    kit unify --write           # one portable library");
    console.log("    kit unify --write --link    # + wire into this project");
    console.log("    kit ready --write           # pack + link this repo in one shot");
  } else if (report.adoptedNames.length > 0) {
    console.log("");
    console.log(
      `  Adopted (${report.adoptedNames.length}): ${report.adoptedNames.slice(0, 12).join(", ")}${report.adoptedNames.length > 12 ? "…" : ""}`,
    );
    console.log("");
    console.log("  Next: kit link --to all --write   ·   kit ready --write   ·   kit list");
  }
  console.log("");
}

function printChecks(checks: CheckResult[], indent = "  "): void {
  for (const check of checks) {
    const mark =
      check.level === "pass"
        ? "✓"
        : check.level === "fail"
          ? "✗"
          : check.level === "warn"
            ? "!"
            : "·";
    console.log(`${indent}${mark} ${check.message}`);
    if (check.detail) {
      console.log(`${indent}  ${check.detail}`);
    }
  }
}

async function runTest(rest: string[]): Promise<void> {
  if (rest.includes("--help") || rest.includes("-h") || rest.length === 0) {
    console.log("Usage:");
    console.log("  kit test --all-packs");
    console.log("  kit test <pack-name>");
    console.log("  kit test <skill-dir>");
    console.log("  kit test pack <pack-name>");
    console.log("  kit test skill <skill-dir>");
    if (rest.length === 0) process.exit(1);
    return;
  }

  if (rest.includes("--all-packs") || rest[0] === "packs") {
    const result = await testAllPacks();
    if (result.ok) {
      console.log(
        `OK  ${result.value.passed} pack(s) passed, ${result.value.failed} failed`,
      );
      for (const pack of result.value.packs) {
        console.log(
          `  ✓ ${pack.packName}@${pack.version} (${pack.skillReports.length} skills)`,
        );
      }
      return;
    }
    if (result.report) {
      console.error(result.error);
      for (const pack of result.report.packs) {
        const mark = pack.ok ? "✓" : "✗";
        console.error(
          `  ${mark} ${pack.packName ?? pack.target}${pack.version ? `@${pack.version}` : ""}`,
        );
        if (!pack.ok) {
          printChecks(pack.checks, "    ");
          for (const skill of pack.skillReports.filter((s) => !s.ok)) {
            console.error(
              `    ✗ ${skill.skillName ?? skill.target}`,
            );
            printChecks(skill.checks, "      ");
            if (skill.issues.length > 0) {
              console.error(`      ${formatIssues(skill.issues)}`);
            }
          }
        }
      }
      process.exit(1);
    }
    fail(result.error);
  }

  if (rest[0] === "pack") {
    const name = rest[1];
    if (!name) fail("Usage: kit test pack <pack-name>");
    await testOnePack(name);
    return;
  }

  if (rest[0] === "skill") {
    const dir = rest[1];
    if (!dir) fail("Usage: kit test skill <skill-dir>");
    await testOneSkill(dir);
    return;
  }

  const target = rest[0];
  if (!target) fail("Usage: kit test <skill-dir|pack-name|--all-packs>");

  // Prefer pack name when it validates as a pack; else skill dir.
  const asPack = await testPack(target);
  if (asPack.ok || (asPack.report && asPack.report.packName)) {
    if (asPack.ok) {
      printPackTestOk(asPack.value);
      return;
    }
    printPackTestFail(asPack.error, asPack.report);
    process.exit(1);
  }

  await testOneSkill(target);
}

async function testOnePack(name: string): Promise<void> {
  const result = await testPack(name);
  if (result.ok) {
    printPackTestOk(result.value);
    return;
  }
  printPackTestFail(result.error, result.report);
  process.exit(1);
}

async function testOneSkill(dir: string): Promise<void> {
  const result = await testSkill(dir);
  if (result.ok) {
    console.log(
      `OK  ${result.value.skillName}@${result.value.version}`,
    );
    printChecks(result.value.checks);
    return;
  }
  console.error(result.error);
  if (result.report) {
    printChecks(result.report.checks);
    if (result.report.issues.length > 0) {
      console.error(formatIssues(result.report.issues));
    }
  }
  process.exit(1);
}

function printPackTestOk(report: {
  packName?: string;
  version?: string;
  skillReports: { skillName?: string; version?: string }[];
  checks: CheckResult[];
}): void {
  console.log(`OK  pack ${report.packName}@${report.version}`);
  printChecks(report.checks);
  for (const skill of report.skillReports) {
    console.log(`  · ${skill.skillName}@${skill.version}`);
  }
}

function printPackTestFail(
  error: string,
  report:
    | {
        packName?: string;
        version?: string;
        checks: CheckResult[];
        skillReports: {
          ok: boolean;
          skillName?: string;
          target: string;
          checks: CheckResult[];
          issues: { field: string; message: string }[];
        }[];
      }
    | undefined,
): void {
  console.error(error);
  if (!report) return;
  printChecks(report.checks);
  for (const skill of report.skillReports.filter((s) => !s.ok)) {
    console.error(`  ✗ ${skill.skillName ?? skill.target}`);
    printChecks(skill.checks, "    ");
    if (skill.issues.length > 0) {
      console.error(`    ${formatIssues(skill.issues)}`);
    }
  }
}

async function runDoctorCmd(rest: string[]): Promise<void> {
  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit doctor [--dir <project>]");
    }
  }

  const report = await runDoctor({
    ...(projectDir ? { projectDir } : {}),
  });

  console.log(`kit doctor  v${report.version}`);
  console.log(`  home:    ${report.kitHome}`);
  console.log(`  project: ${report.projectDir}`);
  console.log(
    `  summary: ${report.summary.passed} pass · ${report.summary.warnings} warn · ${report.summary.failed} fail`,
  );
  console.log("");
  printChecks(report.checks, "");

  if (!report.ok) {
    console.log("");
    console.log("Doctor found problems. Fix the ✗ items above.");
    process.exit(1);
  }

  if (report.summary.warnings > 0) {
    console.log("");
    console.log("Doctor OK with warnings.");
    return;
  }

  console.log("");
  console.log("Doctor OK.");
}

async function runLogin(rest: string[]): Promise<void> {
  if (rest.includes("--help") || rest.includes("-h")) {
    console.log("Usage: kit login");
    console.log("");
    console.log("Signs in with GitHub device flow via the Kit registry.");
    console.log(`Registry: ${getRegistryUrl()}`);
    console.log("Override with KIT_REGISTRY_URL.");
    return;
  }

  console.log("Kit login (GitHub device flow)");
  console.log(`Registry: ${getRegistryUrl()}`);
  console.log("");

  const result = await loginWithDeviceFlow({
    onPrompt: (progress) => {
      console.log(progress.message);
      console.log("");
      console.log(`  1. Open ${progress.verificationUri}`);
      console.log(`  2. Enter code: ${progress.userCode}`);
      console.log("  3. Approve Kit-skills");
      console.log("");
      console.log("Waiting for approval…");
    },
  });

  if (!result.ok) fail(result.error);

  console.log("");
  console.log(`Logged in as @${result.value.user.login}`);
  if (result.value.user.name) {
    console.log(`  name: ${result.value.user.name}`);
  }
  console.log(`  session: ~/.kit/auth.json`);
  console.log("");
  console.log("Next: kit whoami · kit doctor · kit tui");
}

async function runLogout(): Promise<void> {
  const result = await logout();
  if (!result.ok) fail(result.error);
  console.log("Logged out. Session cleared.");
}

async function runWhoami(): Promise<void> {
  const result = await getLoggedInUser();
  if (!result.ok) fail(result.error);
  const s = result.value;
  console.log(`@${s.user.login}`);
  if (s.user.name) console.log(`  name:     ${s.user.name}`);
  console.log(`  id:       ${s.user.id}`);
  console.log(`  profile:  ${s.user.html_url}`);
  console.log(`  loggedIn: ${s.loggedInAt}`);
  console.log(`  registry: ${s.registryUrl}`);
  console.log(`  scope:    ${s.scope || "(none)"}`);
}

async function runExplore(rest: string[]): Promise<void> {
  const sub = rest[0];
  if (!sub || sub === "--help" || sub === "-h") {
    console.log("Usage:");
    console.log("  kit explore packs [--tag <tag>] [--type <projectType>]");
    console.log("  kit explore show <pack>");
    console.log("  kit explore skills [--agent <id>]");
    console.log("  kit explore search <query>");
    console.log("");
    console.log(`Registry: ${getRegistryUrl()}`);
    if (!sub) process.exit(1);
    return;
  }

  if (sub === "packs" || sub === "list") {
    const tagFlag = rest.indexOf("--tag");
    const typeFlag = rest.indexOf("--type");
    let tag: string | undefined;
    let projectType: string | undefined;
    if (tagFlag >= 0) {
      tag = rest[tagFlag + 1];
      if (!tag || tag.startsWith("-")) fail("Usage: kit explore packs --tag <tag>");
    }
    if (typeFlag >= 0) {
      projectType = rest[typeFlag + 1];
      if (!projectType || projectType.startsWith("-")) {
        fail("Usage: kit explore packs --type <projectType>");
      }
    }
    const result = await exploreListPacks({
      ...(tag ? { tag } : {}),
      ...(projectType ? { projectType } : {}),
    });
    if (!result.ok) fail(result.error);
    console.log(`Registry packs (${result.value.count}) — ${getRegistryUrl()}`);
    for (const pack of result.value.packs) {
      console.log(
        `${pack.name}@${pack.version}  (${pack.skillCount} skills)  ${pack.title}`,
      );
      console.log(`  ${pack.description}`);
    }
    return;
  }

  if (sub === "show") {
    const name = rest[1];
    if (!name) fail("Usage: kit explore show <pack>");
    const result = await exploreShowPack(name);
    if (!result.ok) fail(result.error);
    const { pack, skills } = result.value;
    console.log(`${pack.name}@${pack.version}`);
    console.log(pack.title);
    console.log(pack.description);
    console.log(`publisher: ${pack.publisher}`);
    console.log(`tags: ${pack.tags.join(", ") || "(none)"}`);
    console.log("");
    console.log("Skills:");
    for (const skill of skills) {
      console.log(`  - ${skill.name}@${skill.version}`);
      console.log(`    ${skill.description}`);
    }
    console.log("");
    console.log(`Install locally: kit pack install ${pack.name}`);
    return;
  }

  if (sub === "skills") {
    const agentFlag = rest.indexOf("--agent");
    let agent: string | undefined;
    if (agentFlag >= 0) {
      agent = rest[agentFlag + 1];
      if (!agent || agent.startsWith("-")) {
        fail("Usage: kit explore skills --agent <id>");
      }
    }
    const result = await exploreListSkills({
      ...(agent ? { agent } : {}),
    });
    if (!result.ok) fail(result.error);
    console.log(`Registry skills (${result.value.count})`);
    for (const skill of result.value.skills) {
      console.log(`${skill.name}@${skill.version}`);
      console.log(`  ${skill.description}`);
      console.log(`  agents: ${skill.compatibility.join(", ")}`);
    }
    return;
  }

  if (sub === "search") {
    const query = rest.slice(1).join(" ").trim();
    if (!query) fail("Usage: kit explore search <query>");
    const result = await exploreSearch(query);
    if (!result.ok) fail(result.error);
    console.log(`Search "${result.value.q}" — ${result.value.count} hit(s)`);
    if (result.value.packs.length > 0) {
      console.log("");
      console.log("Packs:");
      for (const pack of result.value.packs) {
        console.log(`  ${pack.name}@${pack.version} — ${pack.title}`);
      }
    }
    if (result.value.skills.length > 0) {
      console.log("");
      console.log("Skills:");
      for (const skill of result.value.skills) {
        console.log(`  ${skill.name}@${skill.version} — ${skill.description}`);
      }
    }
    return;
  }

  fail(`kit explore: unknown command: ${sub}`);
}

async function runRecommend(rest: string[]): Promise<void> {
  const dirFlag = rest.indexOf("--dir");
  let projectDir: string | undefined;
  if (dirFlag >= 0) {
    projectDir = rest[dirFlag + 1];
    if (!projectDir || projectDir.startsWith("-")) {
      fail("Usage: kit recommend [--dir <project>]");
    }
  }

  const result = await recommendToolkits({
    ...(projectDir ? { projectDir } : {}),
  });
  if (!result.ok) fail(result.error);

  const report = result.value;
  console.log(report.summary);
  console.log(`Project: ${report.projectDir}`);
  if (report.signals.length > 0) {
    console.log(`Signals: ${report.signals.join(", ")}`);
  } else {
    console.log("Signals: (none — defaults apply)");
  }
  console.log("");
  console.log("Packs");
  for (const rec of report.recommendations) {
    const star = rec.packName === report.topPick ? "★ " : "  ";
    console.log(
      `${star}${rec.packName.padEnd(12)} score ${rec.score}  ${rec.title}`,
    );
    for (const reason of rec.reasons.slice(0, 2)) {
      console.log(`     · ${reason}`);
    }
  }
  if (report.skillRecommendations.length > 0) {
    console.log("");
    console.log("Skills this project likely wants");
    for (const s of report.skillRecommendations) {
      const via = s.fromPack ? ` (via ${s.fromPack})` : "";
      console.log(`  · ${s.skillName}${via}`);
      if (s.reasons[0]) console.log(`      ${s.reasons[0]}`);
    }
  }
  if (report.topPick) {
    console.log("");
    console.log(`Top pick: ${report.topPick}`);
    console.log(`Install:  kit pack install ${report.topPick}`);
    console.log(
      `Apply:    kit pack apply ${report.topPick} --dir ${report.projectDir}`,
    );
    console.log(`Point TUI: kit recommend --dir ${report.projectDir}`);
  }
}

main().catch((error: unknown) => {
  const detail = error instanceof Error ? error.message : String(error);
  console.error(`kit: unexpected error: ${detail}`);
  process.exit(2);
});
