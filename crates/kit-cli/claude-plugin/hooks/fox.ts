// Original parent-reviewed palette grid. Fixed 22×16 source → 22×8 terminal cells.
// Source: https://code.claude.com/docs/en/plugins/mods/interface#draw-a-grid-of-colored-cells
const idle = [
  '......................',
  '......................',
  '...R..............R...',
  '...RR............RR...',
  '...RDR..........RDR...',
  '...RDDRR......RRDDR...',
  '...RDDDRR....RRDDDR...',
  '..RRDDRRRRRRRRRRDDRR..',
  '..RRRRRRRRRRRRRRRRRR..',
  '.RRRRDDDRRRRRRDDDRRRR.',
  'RRRDDDWDDRRRRDDWDDDRRR',
  '..RCDDWDDDDDDDDWDDCR..',
  '...CCCCCCDDDDCCCCCC...',
  '....CCCCCCDDCCCCCC....',
  '......CCCCCCCCCC......',
  '......................',
] as const
export const foxFrames = {
  idle,
  leftEarTwitch: [
    '......................', '..R...................', '..RR..............R...',
    '..RDR............RR...', '..RDDRR.........RDR...',
    ...idle.slice(5),
  ],
  bothEarsPerked: [
    '...R..............R...', '...RR............RR...', '...RDR..........RDR...',
    '...RDDRR......RRDDR...', '...RDDDRR....RRDDDR...', '...RDDDDRR..RRDDDDR...',
    '...RDDDRRR..RRRDDDR...', ...idle.slice(7),
  ],
  blink: [...idle.slice(0, 10), 'RRRDDDDDDRRRRDDDDDDRRR', '..RCDRRRDDDDDDRRRDCR..', ...idle.slice(12)],
} as const
export type FoxFrame = keyof typeof foxFrames
export const defaultColor = 0x01000000
const palette: Record<string, number> = { '.': defaultColor, R: 0xff3b46, D: 0x171b23, C: 0xffe2b0, W: 0xf8f6f1 }
export function foxCells(frame: FoxFrame): [number, number, number][] {
  const rows = foxFrames[frame]
  const cells: [number, number, number][] = []
  for (let y = 0; y < 16; y += 2) for (let x = 0; x < 22; x++) {
    const upper = palette[rows[y]![x]!]!
    const lower = palette[rows[y + 1]![x]!]!
    cells.push(upper !== defaultColor ? [0x2580, upper, lower]
      : lower !== defaultColor ? [0x2584, lower, defaultColor]
      : [32, defaultColor, defaultColor])
  }
  return cells
}
export function foxRaster(frame: FoxFrame): string {
  const cells = foxCells(frame).flat()
  const bytes = new Uint8Array(cells.length * 4)
  const view = new DataView(bytes.buffer)
  cells.forEach((word, i) => view.setUint32(i * 4, word, true))
  return btoa(String.fromCharCode(...bytes))
}
export function reduceMotion(env: Record<string, string | undefined>): boolean {
  return env.KIT_MOTION === 'off' || env.NO_COLOR !== undefined || env.REDUCE_MOTION === '1'
}
export function foxFrameAt(elapsed: number, reduced: boolean): FoxFrame {
  if (reduced) return 'idle'
  const t = elapsed % 19000
  if (t >= 4000 && t < 4110) return 'leftEarTwitch'
  if (t >= 9000 && t < 9100) return 'blink'
  if (t >= 14000 && t < 14220) return 'bothEarsPerked'
  return 'idle'
}
