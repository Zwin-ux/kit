#!/usr/bin/env node
/** Package the same native plugin the Rust CLI embeds. No installs or startup. */
import { cp, lstat, mkdir, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
export const pluginSource = path.join(root, 'crates', 'kit-cli', 'claude-plugin');
export const bundleEntries = ['.claude-plugin/plugin.json', 'hooks', 'agents', 'skills', 'README.md', 'LICENSE', 'provenance.json'];

export async function packagePlugin(destination = path.join(root, 'dist', 'claude-plugin')) {
  const out = path.resolve(destination);
  for (let dir = out; ; dir = path.dirname(dir)) {
    const entry = await lstat(dir).catch(error => {
      if (error.code !== 'ENOENT') throw error;
    });
    if (entry?.isSymbolicLink()) throw new Error(`Refusing linked output path: ${dir}`);
    if (dir === path.dirname(dir)) break;
  }
  await mkdir(path.dirname(out), { recursive: true });
  // A fresh directory preserves both hand-edited and older generated bundles.
  await mkdir(out).catch(error => {
    if (error.code === 'EEXIST') throw new Error(`${out} exists; choose a fresh --out directory.`);
    throw error;
  });
  try {
    for (const entry of bundleEntries) {
      await mkdir(path.dirname(path.join(out, entry)), { recursive: true });
      await cp(path.join(pluginSource, entry), path.join(out, entry), { recursive: true });
    }
  } catch (error) {
    // Only the directory this invocation just created is removed.
    await rm(out, { recursive: true, force: true });
    throw error;
  }
  return out;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  if (args.length && (args.length !== 2 || args[0] !== '--out')) {
    throw new Error('Usage: node scripts/sync-claude-plugin.mjs [--out <fresh-directory>]');
  }
  console.log('wrote', await packagePlugin(args[1]));
}
