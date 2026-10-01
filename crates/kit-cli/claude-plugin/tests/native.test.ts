import { expect, test } from 'claude-code/testing'
import type { CommandRunInput } from 'claude-code'

const command = (args: string): CommandRunInput => ({
  command: 'kit', args, origin: { kind: 'composer' },
  presentation: { isFullscreen: true, columns: 120 },
})

test('catalog is a no-model command with four real buckets', async ($) => {
  const reply = await $.command.run(command(''))
  for (const bucket of ['Frontend', 'Backend', 'Security', 'Product']) expect(reply.text).toContain(bucket)
  expect(reply.text).toContain('Selected for next task')
})

test('use agent only fills a visible draft and preserves its task', async ($, on) => {
  let draft = 'Implement the agreed endpoint.'
  on('prompt.read', () => ({ value: { text: draft, cursor: draft.length } }))
  on('prompt.fill', ($, e) => { draft = e.text; return { isFilled: true } })
  const reply = await $.command.run(command('use backend'))
  expect(draft).toBe('@agent-kit:backend-api-builder\nImplement the agreed endpoint.')
  expect(reply.text).toContain('press Enter')
  // No agent.spawn, model, tools, prompt.submit, permissions or network stub exists.
})

test('a refused fill reports that no job started', async ($, on) => {
  on('prompt.read', () => ({ value: { text: '', cursor: 0 } }))
  on('prompt.fill', () => ({ isFilled: false }))
  expect((await $.command.run(command('use frontend'))).text).toContain('Nothing started')
})

test('jobs report actual session state and ignore unrelated agent definitions', async ($, on) => {
  on('agent.list', () => ({ value: [
    { id: 'native-1', type: 'kit:frontend-ui-builder', description: 'Task one', status: 'completed' },
    { id: 'native-2', type: 'kit:frontend-ui-builder', description: 'Task two', status: 'running' },
    { id: 'outside', type: 'Explore', description: 'Unrelated', status: 'running' },
  ] }))
  const reply = await $.command.run(command('jobs'))
  expect(reply.text).toContain('This Claude session only')
  expect(reply.text).toContain('native-1')
  expect(reply.text).toContain('Needs review')
  expect(reply.text).toContain('native-2')
  expect(reply.text).not.toContain('outside')
})

test('result review preserves a fixed result and does not send it', async ($, on) => {
  let draft = ''
  on('turn.complete', () => ({ text: '' }))
  on('agent.list', () => ({ value: [{ id: 'a', type: 'kit:frontend-ui-builder', description: 'Task', status: 'completed' }] }))
  on('prompt.read', () => ({ value: { text: draft, cursor: draft.length } }))
  on('prompt.fill', ($, e) => { draft = e.text; return { isFilled: true } })
  await $.turn.complete({ agentId: 'a', turnId: 't', answer: 'Artifact: change.patch; tests unverified.', isAborted: false, durationMs: 1, reason: 'answer' })
  await $.command.run(command('review a@t'))
  expect(draft).toContain('@agent-kit:security-reviewer')
  expect(draft).toContain('fixed result a@t')
  expect(draft).toContain('Artifact: change.patch; tests unverified.')
})
