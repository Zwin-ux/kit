#!/usr/bin/env node
import {
  applyPack,
  completeFirstRun,
  formatIssues,
  getFirstRunStatus,
  installPack,
  installSkill,
  isFirstRunPackName,
  listFirstRunPackOptions,
  listPacks,
  listSkills,
  loadPack,
  loadSkill,
  removeSkill,
  validatePack,
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
  console.log("");
  console.log("Library: ~/.kit (or KIT_HOME)");
  console.log("Packs:   packs/ in the Kit repo (or KIT_PACKS)");
  console.log("Tip:     kit init  → first-run starter pack");
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
  console.log("  Point your agent at .kit/skills (or keep using the global library).");
  console.log("  kit list");
  console.log("  kit tui");
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

main().catch((error: unknown) => {
  const detail = error instanceof Error ? error.message : String(error);
  console.error(`kit: unexpected error: ${detail}`);
  process.exit(2);
});
