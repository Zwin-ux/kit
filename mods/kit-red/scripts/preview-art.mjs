// Development-only: render the exact packed Raster assets, not a native Claude screenshot.
import assert from 'node:assert/strict'
import { sprites } from '../hooks/art.js'
import { roles } from '../hooks/roles.js'

const titles = ['Kit fox', ...roles.map(role => role.title)]
const cards = Object.entries(sprites).map(([name, sprite], index) => {
  const cells = Buffer.from(sprite.cells, 'base64')
  assert.equal(cells.length, sprite.columns * sprite.rows * 12)
  let pixels = ''
  for (let row = 0; row < sprite.rows; row++) {
    for (let column = 0; column < sprite.columns; column++) {
      const offset = (row * sprite.columns + column) * 12
      const character = cells.readUInt32LE(offset)
      const foreground = cells.readUInt32LE(offset + 4)
      const background = cells.readUInt32LE(offset + 8)
      assert([0x20, 0x2580, 0x2584, 0x2588].includes(character), name + ': unsupported character')
      for (let half = 0; half < 2; half++) {
        const ink = character === 0x2588 || character === (half ? 0x2584 : 0x2580)
        const color = ink ? foreground : background
        if (color === 0x01000000) continue // Host-default background remains transparent.
        assert(color <= 0xffffff, name + ': invalid color')
        pixels += `<rect x="${column}" y="${row * 2 + half}" width="1" height="1" fill="#${color.toString(16).padStart(6, '0')}"/>`
      }
    }
  }
  const svg = `<svg role="img" aria-label="${titles[index]} packed code asset" viewBox="0 0 ${sprite.columns} ${sprite.rows * 2}" xmlns="http://www.w3.org/2000/svg">${pixels}</svg>`
  return `<section><h2>${titles[index]}</h2><p>${sprite.columns} columns × ${sprite.rows} rows</p><div class="samples"><div class="native" style="--w:${sprite.columns * 8}px;--h:${sprite.rows * 16}px">${svg}<small>Sample 8 × 16 px terminal cells</small></div><div class="detail" style="--w:${sprite.columns * 16}px;--h:${sprite.rows * 32}px">${svg}<small>2× inspection</small></div><div class="light" style="--w:${sprite.columns * 8}px;--h:${sprite.rows * 16}px">${svg}<small>Light-background study</small></div></div></section>`
}).join('\n')

process.stdout.write(`<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Kit code assets</title>
<style>body{margin:0;background:#101214;color:#f4ead7;font:16px/1.6 system-ui}main{max-width:960px;margin:auto;padding:32px 20px}h1{font-size:26px}h2{font-size:20px}p,small{color:#b4b4b4}a{color:#ee8a83}section{padding:24px 0;border-top:1px solid #353535}.samples{display:flex;gap:48px;align-items:start;flex-wrap:wrap}.samples svg{display:block;width:var(--w);height:var(--h);shape-rendering:crispEdges}.light svg{background:#f8f7f3}.samples small{display:block;margin-top:12px}a:focus-visible{outline:2px solid #ee8a83;outline-offset:4px}</style>
<main><h1>Kit assets from the code</h1><p>These drawings decode <code>mods/kit-red/hooks/art.js</code> directly. They are asset previews, not native Claude screenshots. Terminal cell dimensions vary. All four roles retain full text labels.</p><p><a href="review.html">Approved concept direction</a> · <a href="MOD-BUILD.md">Build brief and proof limits</a></p>${cards}</main></html>\n`)
