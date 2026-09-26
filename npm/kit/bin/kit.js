#!/usr/bin/env node
// Kit npm launcher. Finds the native binary for this platform and runs it.
// The binary ships in one optional dependency per platform; npm installs
// only the one whose os/cpu/libc match. Keep this file dependency-free.
"use strict";

const { spawn } = require("node:child_process");
const fs = require("node:fs");

const PLATFORMS = require("../platforms.json");

const VERSION = require("../package.json").version;

function isMusl() {
  if (process.platform !== "linux") return false;
  try {
    // Skip the network section: it can be slow on some hosts.
    process.report.excludeNetwork = true;
    const report = process.report && process.report.getReport();
    return !(report && report.header && report.header.glibcVersionRuntime);
  } catch {
    return false;
  }
}

function hostKey() {
  const libc = isMusl() ? " (musl)" : "";
  return `${process.platform}-${process.arch}${libc}`;
}

function findPlatform() {
  const musl = isMusl();
  return PLATFORMS.find(
    (p) =>
      p.os === process.platform &&
      p.cpu === process.arch &&
      (!p.libc || (p.libc === "glibc" && !musl)),
  );
}

function fail(lines) {
  process.stderr.write(`kit: ${lines.join("\nkit: ")}\n`);
  process.exit(1);
}

function resolveBinary() {
  const override = process.env.KIT_BINARY_PATH;
  if (override) {
    if (!fs.existsSync(override)) fail([`KIT_BINARY_PATH does not exist: ${override}`]);
    return override;
  }

  const platform = findPlatform();
  if (!platform) {
    fail([
      `no prebuilt binary for ${hostKey()}.`,
      `Supported: ${PLATFORMS.map((p) => `${p.os}-${p.cpu}${p.libc ? ` (${p.libc})` : ""}`).join(", ")}.`,
      "Build from source: cargo install --git https://github.com/Zwin-ux/kit kit-cli",
    ]);
  }

  try {
    return require.resolve(`${platform.package}/bin/${platform.bin}`);
  } catch {
    fail([
      `${platform.package} is not installed.`,
      "npm skips it when optional dependencies are off (--omit=optional, --no-optional).",
      // Pin the version: during the alpha, `latest` is still the 0.1 Node app.
      `Reinstall: npm install -g @mzwin/kit@${VERSION}`,
    ]);
  }
}

function ensureExecutable(file) {
  if (process.platform === "win32") return;
  try {
    fs.accessSync(file, fs.constants.X_OK);
  } catch {
    try {
      fs.chmodSync(file, 0o755);
    } catch {
      // spawn reports the real error below.
    }
  }
}

const binary = resolveBinary();
ensureExecutable(binary);

const child = spawn(binary, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: false,
  // `kit doctor` reports the install channel from this.
  env: { ...process.env, KIT_LAUNCHER: "npm" },
});

// The terminal sends Ctrl+C to the whole process group, so the child already
// gets it. The launcher must stay alive until the child exits, or the shell
// prompt returns while kit is still cleaning up.
for (const signal of ["SIGINT", "SIGQUIT", "SIGBREAK"]) {
  try {
    process.on(signal, () => {});
  } catch {
    // Signal not supported on this platform.
  }
}
for (const signal of ["SIGTERM", "SIGHUP"]) {
  try {
    process.on(signal, () => child.kill(signal));
  } catch {
    // Signal not supported on this platform.
  }
}

child.on("error", (err) => fail([`could not start ${binary}: ${err.message}`]));
child.on("exit", (code, signal) => {
  if (signal) {
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});
