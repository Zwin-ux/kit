import { expect, test } from 'claude-code/testing'
import type { CommandRunInput } from 'claude-code'

const command = (args: string): CommandRunInput => ({
  command: 'kit', args, origin: { kind: 'composer' }, presentation: { isFullscreen: true, columns: 120 },
})
const pane = {
  plugin: 'kit', component: 'Pane', surface: 'terminal', requestId: 'kit-specialists',
  props: { title: 'Kit', isFocused: true, bodyColumns: 30, placement: 'dock',
    scroll: { offset: 0, bodyRows: 35 }, view: {} },
} as const
function deferred() {
  let resolve!: () => void
  const promise = new Promise<void>(done => { resolve = done })
  return { promise, resolve }
}

// Official Mods reference maps /branch to resume; the 2.1.287 reason type has no branch value.
for (const [label, reason] of [['clear', 'clear'], ['resume', 'resume'], ['branch', 'resume']] as const) {
  test(`pending Prepare draft cannot fill after ${label}`, async ($, on) => {
    const started = deferred()
    const release = deferred()
    let fills = 0
    on('prompt.read', async () => { started.resolve(); await release.promise; return { value: { text: 'Discarded task', cursor: 14 } } })
    on('prompt.fill', () => { fills++; return { isFilled: true } })
    on('session.end', () => ({ sessionId: 'old' }))
    const ui = await $.ui.mount(pane)
    const pending = ui.press({ key: 'prepare-draft' })
    await started.promise
    await $.session.end({ reason, sessionId: 'old', resume: { id: 'old' } })
    release.resolve()
    await pending
    expect(fills).toBe(0)
  })
}

for (const action of ['use backend', 'review a@t', 'handoff a@t product']) {
  test(`pending /kit ${action} cannot restore an old-session draft`, async ($, on) => {
    const started = deferred()
    const release = deferred()
    let fills = 0
    on('turn.complete', () => ({ text: '' }))
    on('agent.list', () => ({ value: [{ id: 'a', type: 'kit:frontend-ui-builder', description: 'Task', status: 'completed' }] }))
    on('prompt.read', async () => { started.resolve(); await release.promise; return { value: { text: '', cursor: 0 } } })
    on('prompt.fill', () => { fills++; return { isFilled: true } })
    on('session.end', () => ({ sessionId: 'old' }))
    await $.turn.complete({ agentId: 'a', turnId: 't', answer: 'Old artifact', isAborted: false, durationMs: 1, reason: 'answer' })
    const pending = $.command.run(command(action))
    await started.promise
    await $.session.end({ reason: 'clear', sessionId: 'old', resume: { id: 'old' } })
    release.resolve()
    expect((await pending).text).toContain('Session changed')
    expect(fills).toBe(0)
    expect((await $.command.run(command('catalog'))).text).toContain('Selected for next task: kit:frontend-ui-builder')
  })
}

test('double-click locks draft preparation before prompt.read', async ($, on) => {
  const started = deferred()
  const release = deferred()
  let reads = 0
  let fills = 0
  on('prompt.read', async () => { reads++; started.resolve(); await release.promise; return { value: { text: 'Task', cursor: 4 } } })
  on('prompt.fill', () => { fills++; return { isFilled: true } })
  const ui = await $.ui.mount(pane)
  const first = ui.press({ key: 'prepare-draft' })
  await started.promise
  await ui.press({ key: 'prepare-draft' })
  // Count while the first read is held: the host itself can read the prompt after fill.
  expect(reads).toBe(1)
  expect(fills).toBe(0)
  release.resolve()
  await first
  expect(fills).toBe(1)
})

test('an old transaction finishing cannot unlock a new session transaction', async ($, on) => {
  const entered = [deferred(), deferred()]
  const released = [deferred(), deferred()]
  let reads = 0
  const fills: string[] = []
  on('prompt.read', async () => {
    const index = reads++
    entered[index]?.resolve()
    if (released[index]) await released[index]!.promise
    return { value: { text: index === 0 ? 'Old task' : 'New task', cursor: 8 } }
  })
  on('prompt.fill', ($, e) => { fills.push(e.text); return { isFilled: true } })
  on('session.end', () => ({ sessionId: 'old' }))
  const old = $.command.run(command('use backend'))
  await entered[0]!.promise
  await $.session.end({ reason: 'resume', sessionId: 'old', resume: { id: 'old' } })
  const current = $.command.run(command('use product'))
  await entered[1]!.promise
  released[0]!.resolve()
  await old
  const competing = await $.command.run(command('use security'))
  expect(competing.text).toContain('already in progress')
  expect(reads).toBe(2)
  released[1]!.resolve()
  await current
  expect(fills).toEqual(['@agent-kit:product-spec-writer\nNew task'])
})
