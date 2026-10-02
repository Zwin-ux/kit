import { expect, test } from 'claude-code/testing'
import { bandLayout, bandRows } from '../hooks/band.ts'

test('short bands retain all four bucket slots without exceeding their row budget', () => {
  for (const width of [12, 20, 39, 40, 80, 120]) {
    for (const maxRows of [1, 2, 3, 4, 5, 12]) {
      const layout = bandLayout(width, maxRows)
      expect(layout.rows * (layout.rules ? 2 : 1) + layout.padding).toBeLessThanOrEqual(maxRows)
      expect(layout.rows === 1 || layout.rows === 2).toBe(true)
    }
  }
})

test('downstream row bounds preserve wrapping and refuse unknown layout assumptions', () => {
  expect(bandRows({ type: 'engine', ref: 1 }, 20)).toBe(0)
  expect(bandRows({ type: 'Text', children: ['Downstream band'] }, 20)).toBe(1)
  expect(bandRows({ type: 'Text', children: ['A'.repeat(41)] }, 20)).toBe(3)
  expect(bandRows({ type: 'Box', props: { flexDirection: 'column' }, children: [
    { type: 'Text', children: ['First'] }, { type: 'Text', children: ['Second'] },
  ] }, 20)).toBe(2)
  expect(bandRows({ type: 'Text', children: ['aaaaaaaaaaa bbbbbbbbbb cccccccccc'] }, 20)).toBe(3)
  expect(bandRows({ type: 'Box', props: { padding: 2 }, children: [] }, 20)).toBeUndefined()
  expect(bandRows({ type: 'Text', children: ['A\tB'] }, 20)).toBeUndefined()
})
