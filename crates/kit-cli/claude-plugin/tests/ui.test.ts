import { expect, mock, test as nativeTest } from 'claude-code/testing'
import { readyTest as test } from './ready.ts'

const band = (columns = 100) => ({
  plugin: 'kit', component: 'AbovePrompt', surface: 'terminal', requestId: 'band',
  viewport: { columns, rows: 40, isFullscreen: true },
  props: { hasSurvey: false, isWorking: false, maxRows: 12, bodyColumns: columns,
    scroll: { offset: 0, bodyRows: 12 }, view: {} },
} as const)
const pane = {
  plugin: 'kit', component: 'Pane', surface: 'terminal', requestId: 'kit-specialists',
  viewport: { columns: 120, rows: 40, isFullscreen: true },
  props: { title: 'Kit', isFocused: true, bodyColumns: 30, placement: 'dock',
    scroll: { offset: 0, bodyRows: 35 }, view: {} },
} as const

test('bucket selection opens real catalog without filling or invoking anything', async ($, on, ready) => {
  on('ui.render', () => ({ type: 'Box', props: {}, children: [] }))
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('agent.list', () => ({ value: [] }))
  await ready()
  const ui = await $.ui.mount(band())
  await ui.press({ key: 'bucket-Backend' })
  const details = await $.ui.mount(pane)
  expect(await details.find({ type: 'Text', text: 'API builder' })).toBeDefined()
  expect(await details.find({ type: 'Text', text: 'API design' })).toBeDefined()
  expect(await details.find({ type: 'Text', text: 'No Kit attempts observed.' })).toBeDefined()
  expect(await details.find({ key: 'prepare-draft' })).toBeDefined()
  // Deliberately no prompt.fill, prompt.submit, model or agent.spawn stubs.
})

test('explicit Prepare draft preserves ordinary typing and requires native Enter', async ($, on, ready) => {
  let text = 'Fix my form.  Keep this exact text.'
  on('prompt.read', () => ({ value: { text, cursor: text.length } }))
  on('prompt.fill', ($, e) => { text = e.mode === 'append' ? text + e.text : e.text; return { isFilled: true } })
  await ready()
  const ui = await $.ui.mount(pane)
  await ui.press({ key: 'prepare-draft' })
  expect(text).toBe('Fix my form.  Keep this exact text.\n@agent-kit:frontend-ui-builder\n')
  expect(await ui.find({ type: 'Text', text: /press Enter/ })).toBeDefined()
})

test('narrow band stays usable, has no typing hotkeys and yields to surveys', async ($, on, ready) => {
  on('ui.render', () => ({ type: 'Text', props: {}, children: ['Native survey'] }))
  await ready()
  for (const columns of [20, 40, 80, 120]) {
    const ui = await $.ui.mount(band(columns))
    for (const name of ['Frontend', 'Backend', 'Security', 'Product']) {
      const button = await ui.find({ key: `bucket-${name}` })
      expect(button).toBeDefined()
      expect(button!.props.hotkey).toBeUndefined()
    }
    expect(await ui.find({ type: 'Raster' })).toBeUndefined()
    await ui.unmount()
  }
  const ui = await $.ui.mount({ ...band(), props: { ...band().props, hasSurvey: true } })
  expect(await ui.find({ type: 'Text', text: 'Native survey' })).toBeDefined()
})

test('fox uses fixed native raster only when expanded and enough room exists', async ($, on, ready) => {
  await ready()
  const ui = await $.ui.mount(pane)
  const fox = await ui.find({ key: 'kit-fox' })
  expect(fox!.props.columns).toBe(22)
  expect(fox!.props.rows).toBe(8)
  await ui.press({ key: 'toggle-fox' })
  expect(await ui.find({ key: 'kit-fox' })).toBeUndefined()
  await ui.unmount()
  const small = await $.ui.mount({ ...pane, props: { ...pane.props, bodyColumns: 20 } })
  expect(await small.find({ key: 'kit-fox' })).toBeUndefined()
})


test('catalog selection never hides another bucket’s observed work', async ($, on, ready) => {
  on('ui.render', () => ({ type: 'Box', props: {}, children: [] }))
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('agent.list', () => ({ value: [
    { id: 'native-1', type: 'kit:frontend-ui-builder', description: 'Build settings', status: 'completed', parentId: 'owner-7' },
    { id: 'native-2', type: 'kit:backend-api-builder', description: 'API settings', status: 'alien-state' },
  ] }))
  await ready()
  const strip = await $.ui.mount(band())
  await strip.press({ key: 'bucket-Security' })
  const ui = await $.ui.mount(pane)
  expect(await ui.find({ type: 'Text', text: 'Security reviewer' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'kit-task-1 · Needs review' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'kit-task-2 · Unknown (alien-state)' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'Owner: owner-7' })).toBeDefined()
  expect(await ui.find({ key: 'kit-fox' })).toBeUndefined()
})

test('status read failures display unknown instead of stale success', async ($, on, ready) => {
  on('agent.list', () => ({ deny: 'unavailable' }))
  await ready()
  const ui = await $.ui.mount(pane)
  await ui.press({ key: 'refresh-status' })
  expect(await ui.find({ type: 'Text', text: /Native status is unavailable/ })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'Needs review' })).toBeUndefined()
})


nativeTest('native clock moves only the fox and Pause motion holds the idle frame', async ($, on) => {
  const clock = mock.clock(on)
  mock.env(on, {})
  on('session.start', () => ({ cwd: '/work' }))
  on('command.register', () => ({ value: { command: 'kit' } }))
  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/work' })
  const ui = await $.ui.mount(pane)
  const idle = (await ui.find({ key: 'kit-fox' }))!.props.cells
  await clock.advance(4000)
  expect((await ui.find({ key: 'kit-fox' }))!.props.cells).not.toBe(idle)
  await clock.advance(200)
  expect((await ui.find({ key: 'kit-fox' }))!.props.cells).toBe(idle)
  await ui.press({ key: 'toggle-motion' })
  await clock.advance(4800)
  expect((await ui.find({ key: 'kit-fox' }))!.props.cells).toBe(idle)
})

nativeTest('NO_COLOR reduced motion environment keeps the fixed fox idle', async ($, on) => {
  const clock = mock.clock(on)
  mock.env(on, { NO_COLOR: '' })
  on('session.start', () => ({ cwd: '/work' }))
  on('command.register', () => ({ value: { command: 'kit' } }))
  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/work' })
  const ui = await $.ui.mount(pane)
  const idle = (await ui.find({ key: 'kit-fox' }))!.props.cells
  await clock.advance(4000)
  expect((await ui.find({ key: 'kit-fox' }))!.props.cells).toBe(idle)
  expect(await ui.find({ key: 'toggle-motion' })).toBeDefined()
})

test('a result review refuses to overwrite ordinary draft text', async ($, on, ready) => {
  on('turn.complete', () => ({ text: '' }))
  on('agent.list', () => ({ value: [{ id: 'a', type: 'kit:frontend-ui-builder', description: 'Task', status: 'completed' }] }))
  on('prompt.read', () => ({ value: { text: 'My unsent text', cursor: 14 } }))
  await ready()
  await $.turn.complete({ agentId: 'a', turnId: 't', answer: 'Artifact', isAborted: false, durationMs: 1, reason: 'answer' })
  const ui = await $.ui.mount(pane)
  await ui.press({ key: 'review-a@t' })
  expect(await ui.find({ type: 'Text', text: /Your draft contains text/ })).toBeDefined()
  // No fill stub: the existing prompt must never be replaced.
})

test('downstream band retains complete role controls inside narrow budgets', async ($, on, ready) => {
  on('ui.render', () => ({ type: 'Text', props: {}, children: ['Downstream band'] }))
  await ready()
  for (const maxRows of [3, 4, 5, 12]) {
    const ui = await $.ui.mount({ ...band(20), props: { ...band(20).props, maxRows } })
    expect(await ui.find({ type: 'Text', text: 'Downstream band' })).toBeDefined()
    const root = await ui.find({ key: 'kit-composed-band' })
    expect(root).toBeDefined()
    expect(root!.props.height).toBeUndefined()
    const strip = await ui.find({ key: 'kit-bucket-strip' })
    expect(strip!.props.height).toBeLessThanOrEqual(maxRows - 1)
    for (const bucket of ['Frontend', 'Backend', 'Security', 'Product']) {
      expect(await ui.find({ key: `bucket-${bucket}` })).toBeDefined()
    }
    await ui.unmount()
  }
})

test('a full downstream band is preserved instead of clipped to make room for Kit', async ($, on, ready) => {
  on('ui.render', () => ({ type: 'Text', props: {}, children: ['One\nTwo\nThree'] }))
  await ready()
  const ui = await $.ui.mount({ ...band(20), props: { ...band(20).props, maxRows: 3 } })
  expect(await ui.find({ type: 'Text', text: 'One\nTwo\nThree' })).toBeDefined()
  expect(await ui.find({ key: 'bucket-Frontend' })).toBeUndefined()
})


test('a one-row narrow band uses one Kit opener with a complete label', async ($, on, ready) => {
  // The documented ref 0 draws the original native props.
  on('ui.render', () => ({ type: 'engine', ref: 0 }))
  await ready()
  for (const columns of [12, 19, 20, 39]) {
    const ui = await $.ui.mount({ ...band(columns), props: { ...band(columns).props, maxRows: 1 } })
    expect((await ui.find({ key: 'kit-open' }))!.props.label).toBe('Kit')
    expect((await ui.find({ key: 'kit-composed-band' }))!.props.height).toBeUndefined()
    expect((await ui.find({ key: 'kit-opener-row' }))!.props.height).toBe(1)
    expect(await ui.find({ key: 'bucket-Frontend' })).toBeUndefined()
    await ui.unmount()
  }
})

test('Desktop retains draft and status actions without Raster controls', async ($, on, ready) => {
  await ready()
  const ui = await $.ui.mount({ ...pane, surface: 'desktop' })
  expect(await ui.find({ key: 'prepare-draft' })).toBeDefined()
  expect(await ui.find({ key: 'refresh-status' })).toBeDefined()
  expect(await ui.find({ key: 'kit-fox' })).toBeUndefined()
  expect(await ui.find({ key: 'toggle-fox' })).toBeUndefined()
  expect(await ui.find({ key: 'toggle-motion' })).toBeUndefined()
})


test('native engine ref 0 composes with the full role strip in one row', async ($, on, ready) => {
  on('ui.render', () => ({ type: 'engine', ref: 0 }))
  await ready()
  const ui = await $.ui.mount({ ...band(100), props: { ...band(100).props, maxRows: 1 } })
  expect((await ui.find({ key: 'kit-composed-band' }))!.props.height).toBeUndefined()
  expect((await ui.find({ key: 'kit-bucket-strip' }))!.props.height).toBe(1)
  for (const bucket of ['Frontend', 'Backend', 'Security', 'Product']) {
    expect(await ui.find({ key: `bucket-${bucket}` })).toBeDefined()
  }
  await ui.unmount()
})
