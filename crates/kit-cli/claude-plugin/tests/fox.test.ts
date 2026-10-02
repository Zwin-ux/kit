import { expect, test } from 'claude-code/testing'
import { foxFrames, foxCells, foxRaster, foxFrameAt, reduceMotion } from '../hooks/fox.ts'

test('every approved fox frame has the same canvas and face anchor', () => {
  for (const [name, rows] of Object.entries(foxFrames)) {
    expect(rows.length).toBe(16)
    for (const row of rows) expect(row.length).toBe(22)
    for (const y of [7, 8, 9, 12, 13, 14, 15]) expect(rows[y]).toBe(foxFrames.idle[y])
    expect(rows[13]!.slice(10, 12)).toBe('DD')
    for (let y = 0; y < 16; y++) for (let x = 0; x < 22; x++) {
      if (rows[y]![x] === foxFrames.idle[y]![x]) continue
      expect(name === 'blink' ? (y === 10 || y === 11) && ((x >= 5 && x <= 7) || (x >= 14 && x <= 16)) : y <= 6).toBe(true)
    }
  }
})

test('fox cells use upper/lower half blocks and default transparent background', () => {
  const cells = foxCells('idle')
  expect(cells.length).toBe(22 * 8)
  expect(cells[0]).toEqual([32, 0x01000000, 0x01000000])
  expect(cells[22 + 3]).toEqual([0x2580, 0xff3b46, 0xff3b46])
  // Last muzzle source row above a transparent source row.
  expect(cells[7 * 22 + 6]).toEqual([0x2580, 0xffe2b0, 0x01000000])
})

test('brief gestures return to idle and reduced motion never animates', () => {
  expect(foxFrameAt(0, false)).toBe('idle')
  expect(foxFrameAt(4000, false)).toBe('leftEarTwitch')
  expect(foxFrameAt(4110, false)).toBe('idle')
  expect(foxFrameAt(9000, false)).toBe('blink')
  expect(foxFrameAt(9100, false)).toBe('idle')
  expect(foxFrameAt(14000, false)).toBe('bothEarsPerked')
  expect(foxFrameAt(14220, false)).toBe('idle')
  for (const t of [0, 4000, 9000, 14000]) expect(foxFrameAt(t, true)).toBe('idle')
  expect(reduceMotion({ KIT_MOTION: 'off' })).toBe(true)
  expect(reduceMotion({ NO_COLOR: '' })).toBe(true)
  expect(reduceMotion({})).toBe(false)
})


test('Raster encodes every cell as little-endian codepoint foreground background', () => {
  for (const name of Object.keys(foxFrames) as (keyof typeof foxFrames)[]) {
    const binary = atob(foxRaster(name))
    const bytes = Uint8Array.from(binary, c => c.charCodeAt(0))
    const view = new DataView(bytes.buffer)
    expect(bytes.length).toBe(22 * 8 * 12)
    expect(Array.from({ length: bytes.length / 4 }, (_, i) => view.getUint32(i * 4, true))).toEqual(foxCells(name).flat())
  }
})
