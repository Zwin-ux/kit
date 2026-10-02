import assert from 'node:assert/strict';
import { mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { packagePlugin, pluginSource } from './sync-claude-plugin.mjs';

const expectedAgents = ['backend-api-builder', 'frontend-ui-builder', 'product-spec-writer', 'security-reviewer'];
const claude = process.env.KIT_TEST_CLAUDE ?? (process.platform === 'win32' ? 'claude.exe' : 'claude');

async function withBundle(body) {
  const temp = await mkdtemp(path.join(os.tmpdir(), 'kit-plugin-test-'));
  try { await body(await packagePlugin(path.join(temp, 'plugin')), temp); }
  finally { await rm(temp, { recursive: true, force: true }); }
}

test('packaged agent declarations, IDs and preloaded skills match the catalog', async () => {
  await withBundle(async bundle => {
    const manifest = JSON.parse(await readFile(path.join(bundle, '.claude-plugin/plugin.json'), 'utf8'));
    const catalog = await readFile(path.join(bundle, 'hooks/catalog.ts'), 'utf8');
    assert.equal(manifest.version, '2.0.0');
    assert.deepEqual(manifest.agents.map(file => path.basename(file, '.md')).sort(), expectedAgents);
    for (const file of manifest.agents) {
      const contents = await readFile(path.join(bundle, file), 'utf8');
      assert.equal(contents, await readFile(path.join(pluginSource, file), 'utf8'));
      const name = contents.match(/^name: (.+)$/m)?.[1].trim();
      assert.equal(name, path.basename(file, '.md'));
      assert.ok(catalog.includes(`id: 'kit:${name}'`));
      const preload = [...contents.matchAll(/^  - kit:(.+)$/gm)].map(match => match[1].trim());
      assert.equal(preload.length, 2);
      for (const skill of preload) {
        const text = await readFile(path.join(bundle, 'skills', skill, 'SKILL.md'), 'utf8');
        assert.match(text, /user-invocable: false/);
        assert.doesNotMatch(text, /disable-model-invocation: true/);
      }
      if (name !== 'security-reviewer') assert.match(contents, /isolation: worktree/);
    }
    const theme = JSON.parse(await readFile(path.join(bundle, 'themes/kit-red.json'), 'utf8'));
    assert.equal(theme.name, 'Kit Red');
    assert.equal(theme.overrides.claude, '#FF3B46');
    assert.equal((await readdir(path.join(bundle, 'skills'))).length, 8);
    assert.deepEqual(await readdir(path.join(bundle, '.claude-plugin')), ['plugin.json']);
    assert.equal(JSON.parse(await readFile(path.join(bundle, 'provenance.json'), 'utf8')).upstreamInstalled, false);
    await writeFile(path.join(bundle, 'user-note.txt'), 'preserve me');
    await assert.rejects(packagePlugin(bundle), /exists/);
    assert.equal(await readFile(path.join(bundle, 'user-note.txt'), 'utf8'), 'preserve me');
  });
});

test('installed Claude loads all four packaged agents without a model call', async t => {
  const version = spawnSync(claude, ['--version'], { encoding: 'utf8', timeout: 10000 });
  if (version.error?.code === 'ENOENT' && !process.env.KIT_TEST_CLAUDE) {
    t.skip('Claude Code is not installed; no runtime discovery claim');
    return;
  }
  assert.equal(version.status, 0, version.error?.message ?? version.stderr);
  await withBundle(async (bundle, temp) => {
    // Disposable configuration; user credentials, settings and MCP are not loaded.
    const env = { ...process.env, CLAUDE_CONFIG_DIR: path.join(temp, 'config'),
      CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC: '1', ANTHROPIC_API_KEY: '',
      ANTHROPIC_AUTH_TOKEN: '', CLAUDE_CODE_OAUTH_TOKEN: '',
      CLAUDE_CODE_USE_BEDROCK: '0', CLAUDE_CODE_USE_VERTEX: '0', CLAUDE_CODE_USE_FOUNDRY: '0' };
    const run = args => {
      const result = spawnSync(claude, args, { cwd: temp, env, encoding: 'utf8', timeout: 30000 });
      assert.equal(result.status, 0, result.error?.message ?? result.stderr);
      return result.stdout;
    };
    const validation = JSON.parse(run(['plugin', 'validate', bundle, '--strict', '--json']));
    assert.equal(validation.success, true);
    const logPath = path.join(temp, 'discovery.log');
    const reply = run(['--setting-sources', '', '--strict-mcp-config', '--no-chrome',
      '--plugin-dir', bundle, '--debug-file', logPath, '--no-session-persistence', '-p', '/kit catalog']);
    const log = await readFile(logPath, 'utf8');
    // 2.1.287 plugin details incorrectly displays Agents (0), even for flat files.
    // Startup records each file loaded by the real agent loader.
    for (const name of expectedAgents) {
      assert.ok(log.split('\n').some(line => line.includes('Loaded agent from plugin kit custom file:') && line.trim().endsWith(`${name}.md`)), name);
      assert.ok(reply.includes(`kit:${name}`), name);
    }
    assert.match(log, /Total plugin agents loaded: 4/);
    assert.match(log, /Loaded 8 skills from plugin kit default directory/);
    assert.match(reply, /Selected for next task/);
    t.diagnostic(`${version.stdout.trim()}: four agent files and eight skills loaded; local command executed.`);
  });
});
