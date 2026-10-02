const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const { test } = require('node:test');
const html = fs.readFileSync('index.html', 'utf8');
test('Kit provides truthful connection commands for both providers on each desktop platform', () => {
  const scripts = [...html.matchAll(/<script(?:\s[^>]*)?>([\s\S]*?)<\/script>/gi)].map(m => m[1]).join('\n');
  const elements = new Map();
  const node = id => { if (!elements.has(id)) elements.set(id, {value: id === 'platform' ? 'macos' : 'claude', textContent:'', addEventListener(){}, setAttribute(){}, querySelectorAll(){return []}}); return elements.get(id); };
  const context = { document: { getElementById: node, querySelector: node, querySelectorAll(){return []}, addEventListener(){} }, navigator:{clipboard:{async writeText(){}}}, console, setTimeout(){}, clearTimeout(){}, window:{} };
  vm.createContext(context);
  vm.runInContext(scripts, context);
  assert.equal(typeof context.getCommands, 'function');
  for (const platform of ['macos', 'linux', 'windows']) {
    for (const provider of ['claude', 'codex']) {
      const commands = context.getCommands(platform, provider);
      assert.equal(commands.connect, `kit connect ${provider}`);
      assert.equal(commands.launch, 'kit');
      assert.match(commands.run, /^kit run /);
      assert.ok(commands.run.includes(`--agent ${provider}`));
      assert.ok(commands.run.includes('add a test'));
    }
  }
  assert.throws(() => context.getCommands('unknown', 'claude'));
  assert.throws(() => context.getCommands('macos', 'unknown'));
});
test('page is branded, keyboard usable, and avoids collecting credentials or promising unlimited usage', () => {
  assert.match(html, /Frontend/); assert.match(html, /Backend/); assert.match(html, /Security/); assert.match(html, /Product/);
  assert.match(html, /<label[^>]*for=["']platform["']/i);
  assert.match(html, /<label[^>]*for=["']provider["']/i);
  assert.match(html, /id=["']platform["']/i); assert.match(html, /id=["']provider["']/i);
  assert.match(html, /id=["']connect-command["']/i); assert.match(html, /id=["']launch-command["']/i); assert.match(html, /id=["']run-command["']/i);
  assert.match(html, /aria-live=["']polite["']/i);
  assert.match(html, /usage limits/i);
  assert.match(html, /local build|built locally/i);
  assert.doesNotMatch(html, /<input[^>]*type=["'](?:password|email)["']/i);
  assert.doesNotMatch(html, /https?:\/\/[^"'\s]+\.(?:js|css)(?:["'\s])/i);
});
