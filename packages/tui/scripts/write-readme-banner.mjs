/**
 * Legacy pixel-text hero assets (wordmark / banner / loop strip).
 *
 * Prefer the full marketing set (paper background + GIFs + pack tiles):
 *   python packages/tui/scripts/generate-readme-assets.py
 *
 * Black-on-transparent looks fine on light GitHub but vanishes on dark mode.
 * This script still writes transparent PNGs for TUI experiments — use the
 * Python generator for anything linked from README.md.
 *
 * Run: node packages/tui/scripts/write-readme-banner.mjs
 */
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.resolve(__dirname, "../../../docs/assets");

// 5×7 pixel font (uppercase + space + a few symbols)
const FONT = {
  " ": [".....", ".....", ".....", ".....", ".....", ".....", "....."],
  K: ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
  I: ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "#####"],
  T: ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
  P: ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
  O: [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
  R: ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
  A: [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
  B: ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."],
  L: ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
  E: ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
  N: ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"],
  S: [".####", "#....", "#....", ".###.", "....#", "....#", "####."],
  G: [".####", "#....", "#....", "#.###", "#...#", "#...#", ".###."],
  C: [".####", "#....", "#....", "#....", "#....", "#....", ".####"],
  H: ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
  L: ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
  M: ["#...#", "##.##", "#.#.#", "#...#", "#...#", "#...#", "#...#"],
  U: ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
  V: ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
  W: ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"],
  Y: ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
  D: ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."],
  F: ["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
  ".": [".....", ".....", ".....", ".....", ".....", ".##..", ".##.."],
  "-": [".....", ".....", ".....", "#####", ".....", ".....", "....."],
  "→": [".....", "..#..", "...#.", "#####", "...#.", "..#..", "....."],
};

function measure(text, scale, gap) {
  let w = 0;
  for (const ch of text.toUpperCase()) {
    const g = FONT[ch] ?? FONT[" "];
    w += g[0].length * scale + gap;
  }
  return w - gap;
}

function paintText(grid, text, ox, oy, scale, gap) {
  let x = ox;
  for (const ch of text.toUpperCase()) {
    const glyph = FONT[ch] ?? FONT[" "];
    const gw = glyph[0].length;
    const gh = glyph.length;
    for (let gy = 0; gy < gh; gy++) {
      for (let gx = 0; gx < gw; gx++) {
        if (glyph[gy][gx] !== "#") continue;
        for (let sy = 0; sy < scale; sy++) {
          for (let sx = 0; sx < scale; sx++) {
            const px = x + gx * scale + sx;
            const py = oy + gy * scale + sy;
            if (py >= 0 && py < grid.length && px >= 0 && px < grid[0].length) {
              grid[py][px] = true;
            }
          }
        }
      }
    }
    x += gw * scale + gap;
  }
}

function toPng(grid) {
  const h = grid.length;
  const w = grid[0].length;
  const png = new PNG({ width: w, height: h });
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const i = (y * w + x) << 2;
      const on = grid[y][x] === true;
      png.data[i] = 0;
      png.data[i + 1] = 0;
      png.data[i + 2] = 0;
      png.data[i + 3] = on ? 255 : 0;
    }
  }
  return PNG.sync.write(png);
}

function blank(w, h) {
  return Array.from({ length: h }, () => Array(w).fill(false));
}

await mkdir(outDir, { recursive: true });

// --- Wordmark: KIT ---
{
  const scale = 12;
  const gap = 10;
  const padX = 40;
  const padY = 36;
  const text = "KIT";
  const tw = measure(text, scale, gap);
  const th = 7 * scale;
  const w = tw + padX * 2;
  const h = th + padY * 2;
  const grid = blank(w, h);
  paintText(grid, text, padX, padY, scale, gap);
  const file = path.join(outDir, "kit-wordmark.png");
  await writeFile(file, toPng(grid));
  console.log("wrote", file, `${w}x${h}`);
}

// --- Banner: KIT + tagline (GitHub social-friendly width) ---
{
  const scaleTitle = 10;
  const scaleSub = 3;
  const gapTitle = 8;
  const gapSub = 3;
  const title = "KIT";
  const sub = "PORTABLE AGENT SKILLS";
  const padX = 48;
  const padTop = 40;
  const gapLines = 28;
  const padBot = 40;

  const tw = measure(title, scaleTitle, gapTitle);
  const sw = measure(sub, scaleSub, gapSub);
  const contentW = Math.max(tw, sw);
  const w = contentW + padX * 2;
  const th = 7 * scaleTitle;
  const sh = 7 * scaleSub;
  const h = padTop + th + gapLines + sh + padBot;

  const grid = blank(w, h);
  const titleX = Math.floor((w - tw) / 2);
  const subX = Math.floor((w - sw) / 2);
  paintText(grid, title, titleX, padTop, scaleTitle, gapTitle);
  paintText(grid, sub, subX, padTop + th + gapLines, scaleSub, gapSub);

  const file = path.join(outDir, "readme-banner.png");
  await writeFile(file, toPng(grid));
  console.log("wrote", file, `${w}x${h}`);
}

// --- Loop caption strip ---
{
  const scale = 2;
  const gap = 2;
  const line = "POINT  -  RECOMMEND  -  INSTALL  -  LINK";
  const padX = 24;
  const padY = 16;
  const tw = measure(line, scale, gap);
  const th = 7 * scale;
  const w = tw + padX * 2;
  const h = th + padY * 2;
  const grid = blank(w, h);
  paintText(grid, line, padX, padY, scale, gap);
  const file = path.join(outDir, "readme-loop.png");
  await writeFile(file, toPng(grid));
  console.log("wrote", file, `${w}x${h}`);
}

console.log("done →", outDir);
