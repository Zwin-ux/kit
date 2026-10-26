import { expect, test } from 'claude-code/testing'

test('choosing a kit adds guidance without replacing the user prompt or other context', async ($, on) => {
  on('session.start', () => ({ cwd: '/fixture' }))
  on('command.register', () => ({ value: undefined }))
  on('env.get', () => ({ value: undefined }))
  on('ui.status', () => ({ value: undefined }))
  on('store.get', () => ({ value: true }))
  on('store.set', () => ({ value: undefined }))
  on('prompt.submit', ($, e) => ({ text: e.text, context: e.context }))
  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/fixture' })
  await $.command.run({ command: 'kit', args: 'frontend-design' })
  const answer = await $.prompt.submit({ text: 'Fix the button' })
  expect(answer.text).toBe('Fix the button')
  expect(answer.context?.join('\n')).toContain('Frontend Design')
  expect(answer.context?.join('\n')).toContain('keyboard')
})

test('native role buttons change next-prompt guidance on terminal and Desktop', async ($, on) => {
  on('session.start', () => ({ cwd: '/fixture' }))
  on('command.register', () => ({ value: undefined }))
  on('env.get', () => ({ value: undefined }))
  on('ui.status', () => ({ value: undefined }))
  on('store.get', () => ({ value: true }))
  on('prompt.submit', ($, e) => ({ text: e.text, context: e.context }))
  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/fixture' })
  for (const surface of ['terminal', 'desktop'] as const) {
    await $.command.run({ command: 'kit', args: 'off' })
    const ui = await $.ui.mount({
      plugin: 'kit-red', component: 'Pane', requestId: 'kit-red-picker', surface,
      viewport: { columns: 160, rows: 40 },
      props: { title: 'Kit', isFocused: true, bodyColumns: 64, placement: 'dock', scroll: { offset: 0, bodyRows: 30 }, view: {} },
    })
    await ui.press({ key: 'frontend-design' })
    expect(await ui.find({ key: 'frontend-design' })).toMatchObject({ props: { label: '[x] Frontend Design' } })
    const answer = await $.prompt.submit({ text: 'Fix the button' })
    expect(answer.context?.join('\n')).toContain('Frontend Design')
    await ui.press({ key: 'off' })
    expect(await ui.find({ type: 'Text', text: 'Kit guidance is off.' })).toBeDefined()
    expect((await $.prompt.submit({ text: 'Continue' })).context?.join('\n')).toContain('now off')
    await ui.unmount()
  }
})

test('first arrival opens once and an example fills an editable draft without submitting', async ($, on) => {
  const saved = new Map<string, unknown>()
  let opened = 0
  let submitted = 0
  let draft = ''
  on('session.start', () => ({ cwd: '/fixture' }))
  on('command.register', () => ({ value: undefined }))
  on('env.get', () => ({ value: undefined }))
  on('ui.status', () => ({ value: undefined }))
  on('store.get', ($, e) => ({ value: saved.get(e.key) }))
  on('store.set', ($, e) => { saved.set(e.key, e.value); return { value: undefined } })
  on('ui.open', () => { opened++; return { value: undefined } })
  on('ui.panes', () => ({ value: [{ id: 'kit-red-picker', title: 'Kit', isShown: true, isFocused: true, isPlaced: true }] }))
  on('ui.close', () => ({ value: undefined }))
  on('prompt.read', () => ({ value: { text: draft, cursor: draft.length } }))
  on('prompt.fill', ($, e) => { draft += e.text; return { isFilled: true } })
  on('prompt.submit', () => { submitted++; return { text: 'Submitted' } })
  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/fixture' })
  await $.session.start({ surface: 'terminal', isInteractive: true, cwd: '/fixture' })
  expect(opened).toBe(1)
  expect(saved.get('intro-seen')).toBe(true)
  const ui = await $.ui.mount({
    plugin: 'kit-red', component: 'Pane', requestId: 'kit-red-picker', surface: 'terminal',
    viewport: { columns: 80, rows: 24 },
    props: { title: 'Kit', isFocused: true, bodyColumns: 60, placement: 'inline', scroll: { offset: 0, bodyRows: 20 }, view: {} },
  })
  await ui.press({ key: 'backend-engineer' })
  await ui.press({ key: 'example' })
  expect(draft).toContain('existing endpoint')
  expect(submitted).toBe(0)
  await ui.unmount()
})
