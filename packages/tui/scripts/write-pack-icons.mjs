/**
 * Write pure black silhouette PNGs for each pack icon.
 * - 16×16 masters → assets/pixel/packs/
 * - 64×64 nearest-neighbor upscales → docs/assets/packs/ (README)
 *
 * Run: pnpm --filter @kit-skills/tui build && pnpm --filter @kit-skills/tui icons:write
 */
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import {
  OFFICIAL_PACK_ICONS,
  PACK_ICON_SIZE,
  getPackIconBitmap,
} from "../dist/mascot/packIcons.js";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "../../..");
const out16 = path.join(repoRoot, "assets/pixel/packs");
const out64 = path.join(repoRoot, "docs/assets/packs");

await mkdir(out16, { recursive: true });
await mkdir(out64, { recursive: true });

function bitmapToPng(pixels, size) {
  const png = new PNG({ width: size, height: size });
  for (let i = 0; i < pixels.length; i++) {
    const on = pixels[i] === true;
    const o = i * 4;
    png.data[o] = 0;
    png.data[o + 1] = 0;
    png.data[o + 2] = 0;
    png.data[o + 3] = on ? 255 : 0;
  }
  return PNG.sync.write(png);
}

/** Nearest-neighbor upscale for crisp GitHub display. */
function scaleNearest(pixels, srcSize, scale) {
  const dst = srcSize * scale;
  const out = new Array(dst * dst);
  for (let y = 0; y < dst; y++) {
    const sy = Math.floor(y / scale);
    for (let x = 0; x < dst; x++) {
      const sx = Math.floor(x / scale);
      out[y * dst + x] = pixels[sy * srcSize + sx] === true;
    }
  }
  return { pixels: out, size: dst };
}

for (const name of OFFICIAL_PACK_ICONS) {
  const pixels = getPackIconBitmap(name);
  const file16 = path.join(out16, `${name}.png`);
  await writeFile(file16, bitmapToPng(pixels, PACK_ICON_SIZE));
  console.log("wrote", file16);

  const up = scaleNearest(pixels, PACK_ICON_SIZE, 4);
  const file64 = path.join(out64, `${name}.png`);
  await writeFile(file64, bitmapToPng(up.pixels, up.size));
  console.log("wrote", file64);
}

console.log(
  "done",
  OFFICIAL_PACK_ICONS.length,
  "icons × {16,64} → assets/pixel/packs + docs/assets/packs",
);
