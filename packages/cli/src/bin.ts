#!/usr/bin/env node
import {
  applyPack,
  formatIssues,
  installPack,
  installSkill,
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

  if (command === "pack") {
    await runPack(args.slice(1));
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
  console.log("  kit pack list");
  console.log("  kit pack show <pack>");
  console.log("  kit pack validate <pack>");
  console.log("  kit pack install <pack> [--force]");
  console.log("  kit pack apply <pack> [--dir <project>]");
  console.log("");
  console.log("Library: ~/.kit (or KIT_HOME)");
  console.log("Packs:   packs/ in the Kit repo (or KIT_PACKS)");
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

  console.log(`Installed ${result.value.name}@${result.value.version}`);
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
    console.log("Try: kit pack install essentials");
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

async function runPack(rest: string[]): Promise<void> {
  const sub = rest[0];
  if (!sub || sub === "--help" || sub === "-h") {
    printPackHelp();
    process.exit(0);
  }

  if (sub === "list" || sub === "ls") {
    const result = await listPacks();
    if (!result.ok) {
      console.error(result.error);
      process.exit(1);
    }
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
    if (!name) {
      console.error("Usage: kit pack show <pack>");
      process.exit(1);
    }
    const result = await loadPack(name);
    if (!result.ok) {
      console.error(result.error);
      process.exit(1);
    }
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
    if (!name) {
      console.error("Usage: kit pack validate <pack>");
      process.exit(1);
    }
    const result = await validatePack(name);
    if (!result.ok) {
      console.error(result.error);
      process.exit(1);
    }
    console.log(
      `OK  pack ${result.value.pack.name}@${result.value.pack.version}`,
    );
    console.log(`    ${result.value.skills.length} skills valid`);
    return;
  }

  if (sub === "install") {
    const packName = rest.slice(1).find((arg) => !arg.startsWith("-"));
    if (!packName) {
      console.error("Usage: kit pack install <pack> [--force]");
      process.exit(1);
    }
    // Packs replace library skills by default so the set is complete.
    const installResult = await installPack(packName, { force: true });
    if (!installResult.ok) {
      console.error(installResult.error);
      process.exit(1);
    }
    console.log(
      `Installed pack ${installResult.value.pack.name}@${installResult.value.pack.version}`,
    );
    for (const skill of installResult.value.installed) {
      console.log(`  + ${skill.name}@${skill.version}`);
    }
    if (installResult.value.skipped.length > 0) {
      console.log(`  skipped: ${installResult.value.skipped.join(", ")}`);
    }
    return;
  }

  if (sub === "apply") {
    const dirFlag = rest.indexOf("--dir");
    let projectDir: string | undefined;
    if (dirFlag >= 0) {
      projectDir = rest[dirFlag + 1];
      if (!projectDir || projectDir.startsWith("-")) {
        console.error("Usage: kit pack apply <pack> [--dir <project>]");
        process.exit(1);
      }
    }
    const packName = rest
      .slice(1)
      .filter((arg, i, arr) => {
        if (arg.startsWith("-")) return false;
        if (i > 0 && arr[i - 1] === "--dir") return false;
        return true;
      })[0];
    if (!packName) {
      console.error("Usage: kit pack apply <pack> [--dir <project>]");
      process.exit(1);
    }
    const result = await applyPack(packName, {
      ...(projectDir ? { projectDir } : {}),
      force: true,
    });
    if (!result.ok) {
      console.error(result.error);
      process.exit(1);
    }
    console.log(
      `Applied pack ${result.value.pack.name}@${result.value.pack.version}`,
    );
    console.log(`  project: ${result.value.projectDir}`);
    console.log(`  skills:  ${result.value.projectSkillsDir}`);
    console.log(`  record:  ${result.value.appliedPath}`);
    for (const skill of result.value.installed) {
      console.log(`  + ${skill.name}@${skill.version}`);
    }
    return;
  }

  console.error(`kit pack: unknown command: ${sub}`);
  printPackHelp();
  process.exit(1);
}

function printPackHelp(): void {
  console.log("Usage:");
  console.log("  kit pack list");
  console.log("  kit pack show <pack>");
  console.log("  kit pack validate <pack>");
  console.log("  kit pack install <pack> [--force]");
  console.log("  kit pack apply <pack> [--dir <project>]");
}

main().catch((error: unknown) => {
  const detail = error instanceof Error ? error.message : String(error);
  console.error(`kit: ${detail}`);
  process.exit(1);
});
