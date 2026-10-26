import assert from 'node:assert/strict'
import test from 'node:test'
import { sprites } from '../hooks/art.js'
import { roles, parseRoles, contextFor } from '../hooks/roles.js'

let instance = 0
async function host({ placed = true, saved = false, noColor, fail = [] } = {}) {
  const hooks = []
  const calls = []
  const store = new Map(saved ? [['intro-seen', true]] : [])
  let prompt = ''
  let filled = true
  let racedText = ''
  const tree = type => props => ({ type, ...props })
  const $ = {
    env: { get: async () => noColor },
    store: {
      get: async key => { if (fail.includes('read')) throw Error('denied'); return store.get(key) },
      set: async (key, value) => { if (fail.includes('save')) throw Error('denied'); store.set(key, value) },
    },
    command: { register: async value => { if (fail.includes('register')) throw Error('taken'); calls.push(['command', value]) } },
    prompt: {
      read: async () => ({ text: prompt, cursor: prompt.length }),
      fill: async value => { calls.push(['fill', value]); if (filled) prompt = (value.mode === 'append' ? prompt + racedText : '') + value.text; return { isFilled: filled } },
    },
    ui: {
      open: async value => { if (fail.includes('open')) throw Error('denied'); calls.push(['open', value]); return { isPlaced: placed } },
      panes: async () => calls.some(c => c[0] === 'open') ? [{ id: 'kit-red-picker', isPlaced: placed }] : [],
      close: async value => { if (fail.includes('close')) throw Error('denied'); calls.push(['close', value]) },
      status: value => calls.push(['status', value]),
      invalidate: value => calls.push(['invalidate', value]),
      resolve: () => Object.fromEntries(['Box', 'Text', 'Button', 'Raster'].map(type => [type, tree(type)])),
    },
  }
  const { register } = await import('../hooks/register.js?test=' + instance++)
  register((name, filter, fn) => hooks.push({ name, filter: typeof filter === 'function' ? {} : filter, fn: fn ?? filter }))
  async function fire(name, event, next = async e => e) {
    const hook = hooks.find(h => h.name === name && Object.entries(h.filter).every(([key, value]) => Array.isArray(value) ? value.includes(event[key]) : value === event[key]))
    return hook ? hook.fn($, event, next) : next(event)
  }
  return { fire, calls, store, setPrompt: text => { prompt = text }, rejectFill: () => { filled = false }, race: text => { racedText = text }, prompt: () => prompt }
}
function flatten(node) {
  if (!node || typeof node !== 'object') return []
  return [node, ...(node.children ?? []).flatMap(flatten)]
}
const pane = { component: 'Pane', requestId: 'kit-red-picker', surface: 'terminal', props: { bodyColumns: 64, placement: 'dock', scroll: { bodyRows: 26 } } }

test('activation requests the picker once; narrow and headless sessions keep a usable command', async () => {
  const h = await host()
  const event = { isInteractive: true, cwd: '/fixture' }
  assert.deepEqual(await h.fire('session.start', event), event)
  assert.equal(h.calls.filter(c => c[0] === 'open').length, 1)
  assert.equal(h.store.get('intro-seen'), true)
  await h.fire('session.start', event)
  assert.equal(h.calls.filter(c => c[0] === 'open').length, 1)
  const narrow = await host({ placed: false })
  await narrow.fire('session.start', event)
  assert.notEqual(narrow.store.get('intro-seen'), true)
  assert.deepEqual(await narrow.fire('command.run', { command: 'kit', args: '' }), {})
  const headless = await host()
  await headless.fire('session.start', { isInteractive: false })
  assert.equal(headless.calls.filter(c => c[0] === 'open').length, 0)
  assert.match((await headless.fire('command.run', { command: 'kit', args: '' })).text, /frontend/)
})

test('denied preferences and command conflicts do not break normal Claude startup', async () => {
  const event = { isInteractive: true, cwd: '/fixture' }
  const unreadable = await host({ fail: ['read'] })
  assert.deepEqual(await unreadable.fire('session.start', event), event)
  assert.equal(unreadable.calls.filter(c => c[0] === 'open').length, 1)
  const denied = await host({ fail: ['open'] })
  assert.deepEqual(await denied.fire('session.start', event), event)
  assert(flatten(await denied.fire('ui.render', pane)).some(n => n.children?.some(v => typeof v === 'string' && v.includes('could not open'))))
  const conflict = await host({ fail: ['register'] })
  assert.deepEqual(await conflict.fire('session.start', event), event)
  const downstream = { text: 'Other command' }
  assert.equal(await conflict.fire('command.run', { command: 'kit', args: '' }, async () => downstream), downstream)
  assert(!flatten(await conflict.fire('ui.render', { component: 'AbovePrompt' })).some(n => n.children?.includes('Kit · /kit')))
  const failures = ['save']
  const retry = await host({ fail: failures })
  await retry.fire('session.start', event)
  assert(flatten(await retry.fire('ui.render', pane)).some(n => n.children?.includes('First-launch preference could not be saved. /kit remains available.')))
  failures.length = 0
  await retry.fire('command.run', { command: 'kit', args: '' })
  assert.equal(retry.store.get('intro-seen'), true)
  assert(!flatten(await retry.fire('ui.render', pane)).some(n => n.children?.includes('First-launch preference could not be saved. /kit remains available.')))
})

test('a filled draft survives a rejected pane close with an accurate message', async () => {
  const h = await host({ fail: ['close'] })
  await h.fire('session.start', { isInteractive: true })
  await h.fire('command.run', { command: 'kit', args: 'backend' })
  const example = flatten(await h.fire('ui.render', pane)).find(n => n.key === 'example')
  await example.onPress()
  assert.match(h.prompt(), /existing endpoint/)
  const nodes = flatten(await h.fire('ui.render', pane))
  assert(nodes.some(n => n.children?.includes('Added to your prompt. Close Kit to review it. Nothing was sent.')))
})

test('choices compose with the user prompt, survive compaction, and clear explicitly', async () => {
  assert.equal(parseRoles(''), null)
  assert.equal(parseRoles('frontend,unknown'), null)
  assert.deepEqual(parseRoles('frontend-design frontend,backend-engineer'), ['frontend-design', 'backend-engineer'])
  assert.deepEqual(parseRoles('frontend fullstack backend llm'), roles.map(r => r.id))
  assert.match(contextFor([]), /now off/)
  assert(contextFor(roles.map(r => r.id)).length < 3000)
  const h = await host({ saved: true })
  await h.fire('session.start', { isInteractive: true })
  const original = { text: 'Fix this', context: ['Other mod'], attachments: [{ id: 'image' }], origin: 'composer' }
  assert.match((await h.fire('prompt.submit', original)).context[1], /now off/)
  await h.fire('command.run', { command: 'kit', args: 'frontend, backend, frontend' })
  let result = await h.fire('prompt.submit', original)
  assert.equal(result.text, original.text)
  assert.equal(result.attachments, original.attachments)
  assert.equal(result.context[0], 'Other mod')
  assert.match(result.context[1], /Frontend Design/)
  assert.match(result.context[1], /Backend Engineer/)
  const fullstack = await h.fire('command.run', { command: 'kit', args: 'fullstack-design' })
  assert.match(fullstack.context[0], /keyboard/)
  assert.match(fullstack.context[0], /server inputs/)
  await h.fire('command.run', { command: 'kit', args: 'frontend, backend' })
  await h.fire('classic.SessionStart', { source: 'compact' })
  assert.deepEqual(await h.fire('prompt.submit', original), result)
  const invalid = await h.fire('command.run', { command: 'kit', args: 'frontend, bogus' })
  assert.match(invalid.text, /Choose/)
  assert.deepEqual(await h.fire('prompt.submit', original), result)
  for (const source of ['clear', 'resume', 'fork']) {
    await h.fire('command.run', { command: 'kit', args: 'backend-engineer' })
    await h.fire('classic.SessionStart', { source })
    assert.match((await h.fire('prompt.submit', original)).context[1], /now off/)
  }
  await h.fire('command.run', { command: 'kit', args: 'llm' })
  await h.fire('command.run', { command: 'kit', args: 'off' })
  assert.match((await h.fire('prompt.submit', original)).context[1], /now off/)
})

test('native drawings preserve other mods; drafts never overwrite or send a prompt', async () => {
  const h = await host()
  await h.fire('session.start', { isInteractive: true })
  const other = { type: 'Other' }
  const band = await h.fire('ui.render', { component: 'AbovePrompt' }, async () => other)
  assert.equal(band.children[0], other)
  assert.equal(await h.fire('ui.render', { component: 'AbovePrompt', props: { hasSurvey: true } }, async () => other), other)
  await h.fire('command.run', { command: 'kit', args: 'frontend,fullstack,backend,llm' })
  const smallBand = await h.fire('ui.render', { component: 'AbovePrompt', props: { bodyColumns: 25 } })
  assert.equal(smallBand.children[1].children[0], 'Kit · 4 roles · /kit')
  assert.equal(await h.fire('ui.render', { ...pane, requestId: 'other' }, async () => other), other)
  await h.fire('command.run', { command: 'kit', args: 'backend' })
  const nodes = flatten(await h.fire('ui.render', pane))
  assert(nodes.some(n => n.type === 'Raster'))
  assert(nodes.find(n => n.type === 'Raster').cells === sprites.backend.cells, 'Backend emblem shown')
  for (const role of roles) {
    await h.fire('command.run', { command: 'kit', args: role.id })
    const emblem = flatten(await h.fire('ui.render', pane)).find(n => n.type === 'Raster')
    assert(emblem.cells === sprites[role.alias].cells, role.title + ' emblem shown')
  }
  await h.fire('command.run', { command: 'kit', args: 'frontend,llm' })
  assert(flatten(await h.fire('ui.render', pane)).find(n => n.type === 'Raster').cells === sprites.fox.cells, 'Fox for multiple roles')
  await h.fire('command.run', { command: 'kit', args: 'backend' })
  h.setPrompt('My unsent text')
  await nodes.find(n => n.key === 'example').onPress()
  assert.equal(h.calls.filter(c => c[0] === 'fill').length, 0)
  h.setPrompt('')
  h.rejectFill()
  await nodes.find(n => n.key === 'example').onPress()
  assert.equal(h.calls.filter(c => c[0] === 'close').length, 0)
  assert(flatten(await h.fire('ui.render', pane)).some(n => n.children?.includes('Could not fill the prompt. Nothing was sent.')))
  const mono = await host({ noColor: '' })
  await mono.fire('session.start', { isInteractive: true })
  assert(!flatten(await mono.fire('ui.render', pane)).some(n => n.type === 'Raster'))
  assert(!flatten(await h.fire('ui.render', { ...pane, surface: 'desktop' })).some(n => n.type === 'Raster'))
  assert(!flatten(await h.fire('ui.render', { ...pane, props: { bodyColumns: 64, placement: 'dock', scroll: { bodyRows: 20 } } })).some(n => n.type === 'Raster'))
  assert(!flatten(await h.fire('ui.render', { ...pane, props: { ...pane.props, placement: 'inline' } })).some(n => n.type === 'Raster'))
  const race = await host()
  await race.fire('session.start', { isInteractive: true })
  await race.fire('command.run', { command: 'kit', args: 'frontend' })
  race.race('Newly typed text')
  const example = flatten(await race.fire('ui.render', pane)).find(n => n.key === 'example')
  await example.onPress()
  assert.match(race.prompt(), /^Newly typed text\nImprove/)
  assert.equal(race.calls.filter(c => c[0] === 'close').length, 1)
  for (const sprite of Object.values(sprites)) {
    const cells = Buffer.from(sprite.cells, 'base64')
    assert.equal(cells.length, sprite.columns * sprite.rows * 12)
  }
})
