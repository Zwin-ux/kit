import { expect, mock, test } from 'claude-code/testing'

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

test('bucket selection opens real catalog without filling or invoking anything', async ($, on) => {
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('agent.list', () => ({ value: [] }))
  const ui = await $.ui.mount(band())
  await ui.press({ key: 'bucket-Backend' })
  const details = await $.ui.mount(pane)
  expect(await details.find({ type: 'Text', text: 'API builder' })).toBeDefined()
  expect(await details.find({ type: 'Text', text: 'API design' })).toBeDefined()
  expect(await details.find({ type: 'Text', text: 'No Kit attempts observed.' })).toBeDefined()
  expect(await details.find({ key: 'prepare-draft' })).toBeDefined()
  // Deliberately no prompt.fill, prompt.submit, model or agent.spawn stubs.
})

test('explicit Prepare draft preserves ordinary typing and requires native Enter', async ($, on) => {
  let text = 'Fix my form.  Keep this exact text.'
  on('prompt.read', () => ({ value: { text, cursor: text.length } }))
  on('prompt.fill', ($, e) => { text = e.text; return { isFilled: true } })
  const ui = await $.ui.mount(pane)
  await ui.press({ key: 'prepare-draft' })
  expect(text).toBe('@agent-kit:frontend-ui-builder\nFix my form.  Keep this exact text.')
  expect(await ui.find({ type: 'Text', text: /press Enter/ })).toBeDefined()
})

test('narrow band stays usable, has no typing hotkeys and yields to surveys', async ($, on) => {
  on('ui.render', () => ({ type: 'Text', props: {}, children: ['Native survey'] }))
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

test('fox uses fixed native raster only when expanded and enough room exists', async ($) => {
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


test('catalog selection never hides another bucket’s observed work', async ($, on) => {
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('agent.list', () => ({ value: [
    { id: 'native-1', type: 'kit:frontend-ui-builder', description: 'Build settings', status: 'completed', parentId: 'owner-7' },
    { id: 'native-2', type: 'kit:backend-api-builder', description: 'API settings', status: 'alien-state' },
  ] }))
  const strip = await $.ui.mount(band())
  await strip.press({ key: 'bucket-Security' })
  const ui = await $.ui.mount(pane)
  expect(await ui.find({ type: 'Text', text: 'Security reviewer' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'kit-task-1 · Needs review' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'kit-task-2 · Unknown (alien-state)' })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'Owner: owner-7' })).toBeDefined()
  expect(await ui.find({ key: 'kit-fox' })).toBeUndefined()
})

test('status read failures display unknown instead of stale success', async ($, on) => {
  on('agent.list', () => { throw new Error('unavailable') })
  const ui = await $.ui.mount(pane)
  await ui.press({ key: 'refresh-status' })
  expect(await ui.find({ type: 'Text', text: /Native status is unavailable/ })).toBeDefined()
  expect(await ui.find({ type: 'Text', text: 'Needs review' })).toBeUndefined()
})


test('native clock moves only the fox and Pause motion holds the idle frame', async ($, on) => {
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

test('NO_COLOR reduced motion environment keeps the fixed fox idle', async ($, on) => {
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

test('a result review refuses to overwrite ordinary draft text', async ($, on) => {
  on('turn.complete', () => ({ text: '' }))
  on('agent.list', () => ({ value: [{ id: 'a', type: 'kit:frontend-ui-builder', description: 'Task', status: 'completed' }] }))
  on('prompt.read', () => ({ value: { text: 'My unsent text', cursor: 14 } }))
  await $.turn.complete({ agentId: 'a', turnId: 't', answer: 'Artifact', isAborted: false, durationMs: 1, reason: 'answer' })
  const ui = await $.ui.mount(pane)
  await ui.press({ key: 'review-a@t' })
  expect(await ui.find({ type: 'Text', text: /Your draft contains text/ })).toBeDefined()
  // No fill stub: the existing prompt must never be replaced.
})
