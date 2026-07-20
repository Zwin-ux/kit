#!/usr/bin/env node
import {
  formatIssues,
  installSkill,
  listSkills,
  loadSkill,
  removeSkill,
} from "@kit-skills/core";
import { KIT_PACKAGE_VERSION } from "@kit-skills/shared";

const args = process.argv.slice(2);
const command = args[0];

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

  if (command === "tui" || command === "ui" || command === "start") {
    const { startTui } = await import("@kit-skills/tui");
    startTui();
    return;
  }

  console.error(`kit: unknown command: ${command}`);
  console.error("Run `kit --help` for usage.");
  process.exit(1);
}

function printHelp(): void {
  console.log(`kit ${KIT_PACKAGE_VERSION}`);
  console.log("");
  console.log("Terminal-native Agent Skills platform.");
  console.log("");
  console.log("Usage:");
  console.log("  kit --version");
  console.log("  kit --help");
  console.log("  kit tui");
  console.log("  kit validate <skill-dir>");
  console.log("  kit install <skill-dir> [--force]");
  console.log("  kit list");
  console.log("  kit remove <skill-name>");
  console.log("");
  console.log("Library data is stored under ~/.kit (or KIT_HOME).");
  console.log("Mascot frames load from assets/pixel/kit-frame-1.png … 4.png");
}

async function runValidate(rest: string[]): Promise<void> {
  const skillDir = rest[0];
  if (!skillDir) {
    console.error("Usage: kit validate <skill-dir>");
    process.exit(1);
  }

  const result = await loadSkill(skillDir);
  if (!result.ok) {
    console.error("Validation failed:");
    console.error(formatIssues(result.issues));
    process.exit(1);
  }

  console.log(`OK  ${result.skill.name}@${result.skill.version}`);
  console.log(`    ${result.skill.description}`);
  console.log(`    agents: ${result.skill.compatibility.join(", ")}`);
}

async function runInstall(rest: string[]): Promise<void> {
  const force = rest.includes("--force");
  const skillDir = rest.find((arg) => !arg.startsWith("-"));

  if (!skillDir) {
    console.error("Usage: kit install <skill-dir> [--force]");
    process.exit(1);
  }

  const result = await installSkill(skillDir, { force });
  if (!result.ok) {
    console.error(result.error);
    process.exit(1);
  }

  console.log(
    `Installed ${result.value.name}@${result.value.version}`,
  );
  console.log(`  path: ${result.value.installPath}`);
}

async function runList(): Promise<void> {
  const result = await listSkills();
  if (!result.ok) {
    console.error(result.error);
    process.exit(1);
  }

  if (result.value.length === 0) {
    console.log("No skills installed.");
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
  if (!name) {
    console.error("Usage: kit remove <skill-name>");
    process.exit(1);
  }

  const result = await removeSkill(name);
  if (!result.ok) {
    console.error(result.error);
    process.exit(1);
  }

  console.log(`Removed ${result.value.name}`);
}

main().catch((error: unknown) => {
  const detail = error instanceof Error ? error.message : String(error);
  console.error(`kit: ${detail}`);
  process.exit(1);
});
