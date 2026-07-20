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
  linkSkills,
  listFirstRunPackOptions,
  listPacks,
  listSkills,
  loadPack,
  loadSkill,
  loginWithDeviceFlow,
  logout,
  removeSkill,
  runDoctor,
  testAllPacks,
  testPack,
  testSkill,
  validatePack,
  type CheckResult,
  type HarnessId,
  type LinkMode,
  type PathScope,
} from "@kit-skills/core";
import { KIT_PACKAGE_VERSION } from "@kit-skills/shared";

const args = process.argv.slice(2);
const command = args[0];

/** Exit 1 = user/validation; 2 = unexpected (handled in main catch). */
function fail(message: string, code = 1): never {
  console.error(message);
  process.exit(code);
}

async function main(): Promise<void> {
  if (
    command === undefined ||
    command === "--help" ||
    command === "-h" ||
    command === "help"
  ) {
    printHelp();
    process.exit(0);
  }

  if (command === "--version" || command === "-v" || command === "version") {
    console.log(KIT_PACKAGE_VERSION);
    process.exit(0);
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

  if (command === "tui" || command === "ui" || command === "start") {
    const { startTui } = await import("@kit-skills/tui");
    startTui();
    return;
  }

  fail(`kit: unknown command: ${command}\nRun \`kit --help\` for usage.`);
}

function printHelp(): void {
  console.log(`kit ${KIT_PACKAGE_VERSION}`);
  console.log("");
  console.log("Terminal-native Agent Skills platform.");
  console.log("");
  console.log("Usage:");
  console.log("  kit --version");
  console.log("  kit --help");
  console.log("  kit init [--pack essentials|web-app|library] [--apply] [--dir <path>]");
  console.log("  kit tui");
  console.log("  kit validate <skill-dir>");
  console.log("  kit install <skill-dir> [--force]");
  console.log("  kit list");
  console.log("  kit remove <skill-name>");
  console.log("  kit pack list");
  console.log("  kit pack show <pack>");
  console.log("  kit pack validate <pack>");
  console.log("  kit pack install <pack>");
  console.log("  kit pack apply <pack> [--dir <project>]");
  console.log("  kit paths [--dir <project>] [--skill <name>]");
  console.log("  kit link [--to <harness|all>] [--scope personal|project] [--write] [--force] [--mode symlink|copy] [--dir <project>]");
  console.log("  kit test [skill-dir|pack-name|--all-packs]");
  console.log("  kit doctor [--dir <project>]");
  console.log("  kit login");
  console.log("  kit whoami");
  console.log("  kit logout");
  console.log("  kit explore packs");
  console.log("  kit explore search <query>");
  console.log("  kit explore show <pack>");
  console.log("  kit explore skills [--agent <id>]");
  console.log("");
  console.log("Library:  ~/.kit (or KIT_HOME)");
  console.log("Packs:    packs/ in the Kit repo (or KIT_PACKS)");
  console.log("Registry: KIT_REGISTRY_URL or production Railway URL");
  console.log("Windows:  .\\kit.cmd whoami   (from repo root after pnpm build)");
  console.log("Tip:      kit login → kit explore packs → kit pack install essentials");
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
      fail("Usage: kit init [--pack essentials|web-app|library]");
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

main().catch((error: unknown) => {
  const detail = error instanceof Error ? error.message : String(error);
  console.error(`kit: unexpected error: ${detail}`);
  process.exit(2);
});
