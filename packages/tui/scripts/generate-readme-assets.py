#!/usr/bin/env python3
"""
Generate GitHub-safe README marketing assets for Kit.

All assets use a warm paper background so black silhouettes stay readable
on both GitHub light and dark themes (black-on-transparent fails on dark UI).
"""
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageSequence

REPO = Path(__file__).resolve().parents[3]
DOCS = REPO / "docs" / "assets"
PIXEL = REPO / "assets" / "pixel"
PACKS_OUT = DOCS / "packs"

# Warm paper — works on light + dark GitHub
BG = (247, 244, 237, 255)  # #F7F4ED
INK = (18, 18, 18, 255)
MUTED = (90, 86, 78, 255)
ACCENT = (196, 92, 42, 255)  # fox-orange accent
RULE = (210, 204, 192, 255)

FONT: dict[str, list[str]] = {
    " ": [".....", ".....", ".....", ".....", ".....", ".....", "....."],
    "A": [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
    "B": ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."],
    "C": [".####", "#....", "#....", "#....", "#....", "#....", ".####"],
    "D": ["####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####."],
    "E": ["#####", "#....", "#....", "####.", "#....", "#....", "#####"],
    "F": ["#####", "#....", "#....", "####.", "#....", "#....", "#...."],
    "G": [".####", "#....", "#....", "#.###", "#...#", "#...#", ".###."],
    "H": ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"],
    "I": ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "#####"],
    "J": ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."],
    "K": ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"],
    "L": ["#....", "#....", "#....", "#....", "#....", "#....", "#####"],
    "M": ["#...#", "##.##", "#.#.#", "#...#", "#...#", "#...#", "#...#"],
    "N": ["#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#"],
    "O": [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    "P": ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."],
    "Q": [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"],
    "R": ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"],
    "S": [".####", "#....", "#....", ".###.", "....#", "....#", "####."],
    "T": ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
    "U": ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."],
    "V": ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."],
    "W": ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "##.##", "#...#"],
    "X": ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"],
    "Y": ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."],
    "Z": ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"],
    "0": [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."],
    "1": ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."],
    "2": [".###.", "#...#", "....#", "..##.", ".#...", "#....", "#####"],
    "3": [".###.", "#...#", "....#", "..##.", "....#", "#...#", ".###."],
    "4": ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."],
    "5": ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."],
    "6": [".###.", "#....", "####.", "#...#", "#...#", "#...#", ".###."],
    "7": ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."],
    "8": [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."],
    "9": [".###.", "#...#", "#...#", ".####", "....#", "....#", ".###."],
    ".": [".....", ".....", ".....", ".....", ".....", ".##..", ".##.."],
    "-": [".....", ".....", ".....", "#####", ".....", ".....", "....."],
    "·": [".....", ".....", ".....", ".##..", ".##..", ".....", "....."],
    "/": ["....#", "...#.", "...#.", "..#..", ".#...", ".#...", "#...."],
    ">": ["#....", ".#...", "..#..", "...#.", "..#..", ".#...", "#...."],
    "*": [".#.#.", "..#..", "#####", "..#..", ".#.#.", ".....", "....."],
    "$": [".###.", "#.#..", "#.#..", "####.", ".#.#.", ".#.#.", "###.."],
    "✓": [".....", "....#", "...#.", "#.#..", ".#...", ".....", "....."],
    "→": [".....", "..#..", "...#.", "#####", "...#.", "..#..", "....."],
}


def measure(text: str, scale: int, gap: int) -> int:
    w = 0
    for ch in text.upper():
        g = FONT.get(ch, FONT[" "])
        w += len(g[0]) * scale + gap
    return max(0, w - gap)


def paint_text(
    img: Image.Image,
    text: str,
    ox: int,
    oy: int,
    scale: int,
    gap: int,
    color: tuple[int, int, int, int] = INK,
) -> None:
    x = ox
    for ch in text.upper():
        glyph = FONT.get(ch, FONT[" "])
        gh, gw = len(glyph), len(glyph[0])
        for gy in range(gh):
            for gx in range(gw):
                if glyph[gy][gx] != "#":
                    continue
                for sy in range(scale):
                    for sx in range(scale):
                        px, py = x + gx * scale + sx, oy + gy * scale + sy
                        if 0 <= px < img.width and 0 <= py < img.height:
                            img.putpixel((px, py), color)
        x += gw * scale + gap


def solid(w: int, h: int, color=BG) -> Image.Image:
    return Image.new("RGBA", (w, h), color)


def load_ink_rgba(path: Path) -> Image.Image:
    """Load PNG/GIF frame; pure black ink, transparent elsewhere. Crops to content."""
    im = Image.open(path).convert("RGBA")
    # Build alpha mask of dark opaque pixels
    mask = Image.new("L", im.size, 0)
    sp, mp = im.load(), mask.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = sp[x, y]
            if a > 20 and (r + g + b) < 240:
                mp[x, y] = 255
    bbox = mask.getbbox()
    if bbox:
        im = im.crop(bbox)
        mask = mask.crop(bbox)
    out = Image.new("RGBA", im.size, (0, 0, 0, 0))
    op = out.load()
    mp = mask.load()
    for y in range(out.height):
        for x in range(out.width):
            if mp[x, y] > 0:
                op[x, y] = INK
    return out


def fit_height(im: Image.Image, target_h: int) -> Image.Image:
    """Nearest-neighbor scale so height == target_h (up or down)."""
    if im.height == target_h:
        return im
    factor = target_h / im.height
    nw = max(1, int(round(im.width * factor)))
    nh = target_h
    return im.resize((nw, nh), Image.Resampling.NEAREST)


def fit_width(im: Image.Image, target_w: int) -> Image.Image:
    if im.width == target_w:
        return im
    factor = target_w / im.width
    nw = target_w
    nh = max(1, int(round(im.height * factor)))
    return im.resize((nw, nh), Image.Resampling.NEAREST)


def paste_center(base: Image.Image, overlay: Image.Image, cy: int) -> None:
    x = (base.width - overlay.width) // 2
    y = cy - overlay.height // 2
    base.alpha_composite(overlay, (max(0, x), max(0, y)))


def quantize_gif_frame(rgba: Image.Image) -> Image.Image:
    """
    Stable 4-color palette: paper, ink, muted, accent.
    Avoids adaptive remapping that turns black into red/cyan.
    """
    # Flatten onto paper first
    flat = solid(rgba.width, rgba.height)
    flat.alpha_composite(rgba)
    # Build fixed palette image
    palette_img = Image.new("P", (1, 1))
    # 256-entry palette: index 0=paper, 1=ink, 2=muted, 3=accent, rest paper
    pal = []
    colors = [
        BG[:3],
        INK[:3],
        MUTED[:3],
        ACCENT[:3],
        RULE[:3],
    ]
    for c in colors:
        pal.extend(c)
    while len(pal) < 256 * 3:
        pal.extend(BG[:3])
    palette_img.putpalette(pal)

    # Map each pixel to nearest palette index
    src = flat.convert("RGB")
    out = Image.new("P", src.size)
    out.putpalette(pal)
    sp, op = src.load(), out.load()
    targets = [BG[:3], INK[:3], MUTED[:3], ACCENT[:3], RULE[:3]]

    def nearest(rgb):
        best, bd = 0, 1e18
        for i, t in enumerate(targets):
            d = (rgb[0] - t[0]) ** 2 + (rgb[1] - t[1]) ** 2 + (rgb[2] - t[2]) ** 2
            if d < bd:
                best, bd = i, d
        return best

    for y in range(src.height):
        for x in range(src.width):
            op[x, y] = nearest(sp[x, y])
    return out


def write_wordmark() -> Path:
    scale, gap, pad_x, pad_y = 14, 12, 48, 40
    text = "KIT"
    tw, th = measure(text, scale, gap), 7 * scale
    img = solid(tw + pad_x * 2, th + pad_y * 2)
    paint_text(img, text, pad_x, pad_y, scale, gap)
    path = DOCS / "kit-wordmark.png"
    img.save(path, "PNG")
    print("wrote", path, img.size)
    return path


def write_banner(fox: Image.Image) -> Path:
    """
    Wide hero: left copy + right fox (fixed layout, nothing covers text).
    960×320 paper card with orange rule.
    """
    W, H = 960, 320
    img = solid(W, H)
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, W, 6], fill=ACCENT)
    draw.rectangle([0, H - 6, W, H], fill=RULE)

    # Left column text
    title = "KIT"
    t_scale, t_gap = 16, 12
    paint_text(img, title, 64, 72, t_scale, t_gap)

    sub = "PORTABLE AGENT SKILLS"
    paint_text(img, sub, 64, 72 + 7 * t_scale + 24, 3, 3, MUTED)

    line = "POINT  ·  RECOMMEND  ·  INSTALL  ·  LINK"
    paint_text(img, line, 64, H - 72, 2, 2, MUTED)

    # Right: fox fitted to ~180px tall
    fox_s = fit_height(fox, 180)
    fx = W - fox_s.width - 72
    fy = (H - fox_s.height) // 2
    img.alpha_composite(fox_s, (fx, fy))

    path = DOCS / "readme-banner.png"
    img.save(path, "PNG")
    print("wrote", path, img.size)
    return path


def write_loop_strip() -> Path:
    steps = ["POINT", "RECOMMEND", "INSTALL", "LINK"]
    scale, gap = 3, 3
    step_ws = [measure(s, scale, gap) for s in steps]
    dot_w = measure("·", scale, gap)
    total = sum(step_ws) + dot_w * 3 + 28 * 3
    pad_x, pad_y = 40, 28
    W = total + pad_x * 2
    H = 7 * scale + pad_y * 2 + 16
    img = solid(W, H)
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, W, 4], fill=ACCENT)

    x = pad_x
    y = pad_y + 8
    for i, step in enumerate(steps):
        paint_text(img, step, x, y, scale, gap)
        x += step_ws[i]
        if i < len(steps) - 1:
            x += 10
            paint_text(img, "·", x, y, scale, gap, MUTED)
            x += dot_w + 10

    path = DOCS / "readme-loop.png"
    img.save(path, "PNG")
    print("wrote", path, img.size)
    return path


def write_mascot(fox: Image.Image) -> Path:
    f = fit_height(fox, 200)
    pad = 28
    img = solid(f.width + pad * 2, f.height + pad * 2)
    img.alpha_composite(f, (pad, pad))
    path = DOCS / "kit-mascot.png"
    img.save(path, "PNG")
    print("wrote", path, img.size)
    return path


def write_pack_cards() -> None:
    PACKS_OUT.mkdir(parents=True, exist_ok=True)
    src_dir = PIXEL / "packs"
    for src in sorted(src_dir.glob("*.png")):
        raw = load_ink_rgba(src)
        raw = fit_height(raw, 40)
        tile = solid(72, 72)
        draw = ImageDraw.Draw(tile)
        draw.rounded_rectangle([1, 1, 70, 70], radius=12, outline=RULE, width=2)
        ox = (72 - raw.width) // 2
        oy = (72 - raw.height) // 2
        tile.alpha_composite(raw, (ox, oy))
        out = PACKS_OUT / src.name
        tile.save(out, "PNG")
        print("wrote", out)


def write_flow_gif(fox: Image.Image) -> Path:
    """Four-beat product loop for the README."""
    W, H = 720, 300
    frames: list[Image.Image] = []
    beats = [
        ("01  POINT", "AIM KIT AT A PROJECT FOLDER"),
        ("02  RECOMMEND", "PICK THE RIGHT STARTER PACK"),
        ("03  INSTALL", "LAND SKILLS IN THE LIBRARY"),
        ("04  LINK", "WIRE CLAUDE · GROK · CODEX"),
    ]
    fox_s = fit_height(fox, 72)

    for idx, (title, sub) in enumerate(beats):
        for pulse in range(10):  # ~1.2s per beat
            img = solid(W, H)
            draw = ImageDraw.Draw(img)
            draw.rectangle([0, 0, W, 5], fill=ACCENT)
            draw.rounded_rectangle([20, 20, W - 20, H - 20], radius=8, outline=RULE, width=2)

            t_scale = 4
            tw = measure(title, t_scale, 3)
            paint_text(img, title, (W - tw) // 2, 48, t_scale, 3)

            s_scale = 2
            sw = measure(sub, s_scale, 2)
            paint_text(
                img, sub, (W - sw) // 2, 48 + 7 * t_scale + 16, s_scale, 2, MUTED
            )

            bob = 0 if pulse < 5 else 2
            fx = (W - fox_s.width) // 2
            fy = H - 100 - bob
            img.alpha_composite(fox_s, (fx, fy))

            n = len(beats)
            total_w = n * 12 + (n - 1) * 12
            dx = (W - total_w) // 2
            for i in range(n):
                cx = dx + i * 24 + 6
                cy = H - 40
                fill = ACCENT if i == idx else RULE
                draw.ellipse([cx - 5, cy - 5, cx + 5, cy + 5], fill=fill)

            frames.append(quantize_gif_frame(img))

    path = DOCS / "kit-flow.gif"
    frames[0].save(
        path,
        save_all=True,
        append_images=frames[1:],
        duration=120,
        loop=0,
        optimize=False,
        disposal=2,
    )
    print("wrote", path, f"{len(frames)} frames")
    return path


def rewrite_idle_gif() -> Path:
    """Marketing kit-idle on paper, fixed palette, ~240px."""
    src = PIXEL / "kit-idle.gif"
    frames_out: list[Image.Image] = []
    with Image.open(src) as im:
        for frame in ImageSequence.Iterator(im):
            ink = load_ink_rgba_from_image(frame.convert("RGBA"))
            fox = fit_height(ink, 160)
            pad = 36
            canvas = solid(fox.width + pad * 2, fox.height + pad * 2)
            canvas.alpha_composite(fox, (pad, pad))
            # square frame
            side = max(canvas.width, canvas.height, 240)
            sq = solid(side, side)
            sq.alpha_composite(
                canvas,
                ((side - canvas.width) // 2, (side - canvas.height) // 2),
            )
            frames_out.append(quantize_gif_frame(sq))

    path = DOCS / "kit-idle.gif"
    frames_out[0].save(
        path,
        save_all=True,
        append_images=frames_out[1:],
        duration=180,
        loop=0,
        optimize=False,
        disposal=2,
    )
    print("wrote", path, f"{len(frames_out)} frames")
    return path


def load_ink_rgba_from_image(im: Image.Image) -> Image.Image:
    mask = Image.new("L", im.size, 0)
    sp, mp = im.load(), mask.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = sp[x, y]
            if a > 20 and (r + g + b) < 240:
                mp[x, y] = 255
    bbox = mask.getbbox()
    if bbox:
        im = im.crop(bbox)
        mask = mask.crop(bbox)
    out = Image.new("RGBA", im.size, (0, 0, 0, 0))
    op, mp = out.load(), mask.load()
    for y in range(out.height):
        for x in range(out.width):
            if mp[x, y] > 0:
                op[x, y] = INK
    return out


def write_social_preview(fox: Image.Image) -> Path:
    W, H = 1280, 640
    img = solid(W, H)
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, W, 10], fill=ACCENT)
    draw.rectangle([0, H - 10, W, H], fill=RULE)

    # Left text block
    title = "KIT"
    t_scale, t_gap = 22, 16
    paint_text(img, title, 80, 160, t_scale, t_gap)

    sub = "ONE LIBRARY. MANY AGENTS."
    paint_text(img, sub, 80, 160 + 7 * t_scale + 32, 4, 4, MUTED)

    line = "POINT  ·  RECOMMEND  ·  INSTALL  ·  LINK"
    paint_text(img, line, 80, H - 140, 3, 3, MUTED)

    # Right fox
    f = fit_height(fox, 320)
    fx = W - f.width - 100
    fy = (H - f.height) // 2
    img.alpha_composite(f, (fx, fy))

    path = DOCS / "social-preview.png"
    img.save(path, "PNG")
    print("wrote", path, img.size)
    return path


def write_terminal_card() -> Path:
    """Static 'what you get' terminal-style card for README."""
    W, H = 720, 280
    img = solid(W, H)
    draw = ImageDraw.Draw(img)
    draw.rectangle([0, 0, W, 5], fill=ACCENT)
    draw.rounded_rectangle([16, 16, W - 16, H - 16], radius=8, outline=RULE, width=2)

    lines = [
        ("$ ", "kit recommend --dir .", MUTED, INK),
        ("  ", "→ web-app  score 25", None, ACCENT),
        ("", "", None, INK),
        ("$ ", "kit pack apply web-app --dir .", MUTED, INK),
        ("  ", "✓ skills applied", None, ACCENT),
        ("$ ", "kit link --to claude-code --write", MUTED, INK),
        ("  ", "✓ harness linked · doctor green", None, ACCENT),
    ]
    y = 40
    for prefix, body, pcol, bcol in lines:
        x = 40
        if prefix:
            paint_text(img, prefix, x, y, 2, 2, pcol or MUTED)
            x += measure(prefix, 2, 2) + 12
        if body:
            paint_text(img, body, x, y, 2, 2, bcol)
        y += 28

    path = DOCS / "readme-terminal.png"
    img.save(path, "PNG")
    print("wrote", path, img.size)
    return path


def main() -> None:
    DOCS.mkdir(parents=True, exist_ok=True)
    PACKS_OUT.mkdir(parents=True, exist_ok=True)

    frame1 = PIXEL / "kit-frame-1.png"
    if not frame1.exists():
        raise SystemExit(f"missing {frame1}")
    fox = load_ink_rgba(frame1)
    print("fox master", fox.size)

    write_wordmark()
    write_banner(fox)
    write_loop_strip()
    write_mascot(fox)
    write_pack_cards()
    write_flow_gif(fox)
    rewrite_idle_gif()
    write_social_preview(fox)
    write_terminal_card()
    print("done →", DOCS)


if __name__ == "__main__":
    main()
