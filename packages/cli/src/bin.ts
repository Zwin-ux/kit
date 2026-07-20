#!/usr/bin/env node
import { KIT_PACKAGE_VERSION } from "@kit-skills/shared";

const args = process.argv.slice(2);

if (args.includes("--help") || args.includes("-h") || args.length === 0) {
  console.log(`kit ${KIT_PACKAGE_VERSION}`);
  console.log("");
  console.log("Terminal-native Agent Skills platform.");
  console.log("");
  console.log("Usage:");
  console.log("  kit --version");
  console.log("  kit --help");
  console.log("");
  console.log("More commands will land in Phase 1.");
  process.exit(0);
}

if (args.includes("--version") || args.includes("-v")) {
  console.log(KIT_PACKAGE_VERSION);
  process.exit(0);
}

console.error(`kit: unknown command: ${args.join(" ")}`);
console.error("Run `kit --help` for usage.");
process.exit(1);
