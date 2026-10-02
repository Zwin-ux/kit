import type { RenderElement } from 'claude-code'

// AbovePrompt's core engine reference is empty. For other mods, only compose
// when this small subset gives a safe row bound. Opaque/custom layouts keep the
// entire band; /kit open remains available. Never clip someone else's controls.
export function bandRows(tree: RenderElement, columns: number): number | undefined {
  if (columns < 1) return undefined
  if (tree.type === 'engine') return 0
  if (tree.type === 'Text') {
    if (tree.children?.some(child => typeof child !== 'string')) return undefined
    const text = (tree.children ?? []).join('')
    if (!text) return 0
    // Tabs, escape sequences and control characters have host-dependent width.
    if (/[\x00-\x09\x0b-\x1f\x7f]/.test(text)) return undefined
    return text.split('\n').reduce((rows, line) => {
      // Two cells for every non-ASCII scalar is conservative for wide/combined glyphs.
      const width = (value: string) => [...value].reduce((n, char) => n + (char.charCodeAt(0) < 128 ? 1 : 2), 0)
      if (width(line) <= columns) return rows + 1
      // Word wrapping can use more rows than ceil(total width / columns).
      // Bound each word separately rather than assuming the host's wrap algorithm.
      return rows + (line.match(/\S+\s*|\s+/g) ?? ['']).reduce((n, word) => n + Math.max(1, Math.ceil(width(word) / columns)), 0)
    }, 0)
  }
  if (tree.type !== 'Box') return undefined
  const props = tree.props ?? {}
  if (Object.keys(props).some(key => key !== 'key' && key !== 'flexDirection')) return undefined
  if (props.flexDirection !== undefined && props.flexDirection !== 'column') return undefined
  // A Box defaults to a row, so only a zero/one-child Box is unambiguous without direction.
  if (props.flexDirection === undefined && (tree.children?.length ?? 0) > 1) return undefined
  let rows = 0
  for (const child of tree.children ?? []) {
    const height = bandRows(typeof child === 'string' ? { type: 'Text', children: [child] } : child, columns)
    if (height === undefined) return undefined
    rows += height
  }
  return rows
}

export function bandLayout(columns: number, availableRows: number) {
  const rows = availableRows < 1 ? 0 : columns < 40 && availableRows >= 2 ? 2 : 1
  const rules = availableRows >= rows * 2 && rows > 0
  const used = rows * (rules ? 2 : 1)
  const padding = availableRows > used && rows > 0 ? 1 : 0
  return { rows, rules, padding, height: used + padding }
}
