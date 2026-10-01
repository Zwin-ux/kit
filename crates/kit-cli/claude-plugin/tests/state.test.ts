import { expect, test } from 'claude-code/testing'
import { agents, invocation } from '../hooks/catalog.ts'
import { initialState, reconcile, recordResult, resultStatus, statusLabel } from '../hooks/state.ts'

test('repeated selection preserves the task and does not duplicate the invocation', () => {
  const draft = 'Build the settings page.\nKeep my exact task.  '
  const first = invocation(agents[0]!, draft)
  expect(invocation(agents[1]!, first)).toBe(`@agent-kit:backend-api-builder\n${draft}`)
})

test('two attempts of one specialist keep distinct native identities', () => {
  const jobs = ['a', 'b'].map(id => ({ id, type: agents[0]!.id, description: 'Independent task', status: 'running' }))
  const state = reconcile(initialState(), jobs)
  expect(state.jobs.map(j => j.id)).toEqual(['a', 'b'])
  expect(reconcile(state, [jobs[1]!]).jobs.find(j => j.id === 'a')!.status).toBe('not-attached')
})

test('completion requires review and unknown native status remains unknown', () => {
  expect(statusLabel('completed')).toBe('Needs review')
  expect(statusLabel('new-host-state')).toBe('Unknown (new-host-state)')
  expect(resultStatus('error')).toBe('Failed')
  expect(resultStatus('refusal')).toBe('Refused')
})

test('a captured result is stable when a repeated event tries to replace it', () => {
  const result = { agentId: 'a', agentType: agents[0]!.id, task: 'Task', turnId: 't1', answer: 'Fixed result', reason: 'answer' as const }
  const first = recordResult(initialState(), result)
  expect(recordResult(first, { ...result, answer: 'Changed result' }).results[0]!.answer).toBe('Fixed result')
  expect(recordResult(first, { ...result, turnId: 't2' }).results.length).toBe(2)
})
