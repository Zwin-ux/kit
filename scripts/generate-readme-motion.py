#!/usr/bin/env python3
"""
README motion assets: demo GIFs + packs strip + kit-success.
Pixel paper cards — exact font, no AI text.
"""
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

REPO = Path(__file__).resolve().parents[1]
OUT = REPO / "docs" / "assets"
PIXEL = REPO / "assets" / "pixel"
PACKS = PIXEL / "packs"

BG = (247, 244, 237, 255)
INK = (18, 18, 18, 255)
MUTED = (90, 86, 78, 255)
ACCENT = (196, 92, 42, 255)
RULE = (210, 204, 192, 255)
GREEN = (46, 125, 50, 255)
RED = (160, 50, 40, 255)

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
    ":": [".....", ".##..", ".##..", ".....", ".##..", ".##..", "....."],
    "/": ["....#", "...#.", "...#.", "..#..", ".#...", ".#...", "#...."],
    "~": [".....", ".....", ".#..#", "#.##.", ".....", ".....", "....."],
    "+": [".....", "..#..", "..#..", "#####", "..#..", "..#..", "....."],
    "×": [".....", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "....."],
    "·": [".....", ".....", ".....", ".##..", ".##..", ".....", "....."],
    "$": [".###.", "#.#..", "#.#..", "####.", ".#.#.", ".#.#.", "###.."],
    "✓": [".....", "....#", "...#.", "#.#..", ".#...", ".....", "....."],
    "→": [".....", "..#..", "...#.", "#####", "...#.", "..#..", "....."],
    "★": ["..#..", "..#..", "#####", ".###.", "#.#.#", ".....", "....."],
    "(": ["..##.", ".#...", "#....", "#....", "#....", ".#...", "..##."],
    ")": [".##..", "...#.", "....#", "....#", "....#", "...#.", ".##.."],
    "'": [".##..", ".##..", ".#...", ".....", ".....", ".....", "....."],
    ",": [".....", ".....", ".....", ".....", ".##..", ".##..", ".#..."],
    "%": ["##..#", "##.#.", "..#..", ".#...", ".#.##", "#..##", "....."],
    ">": [".....", ".#...", "..#..", "...#.", "..#..", ".#...", "....."],
    "|": ["..#..", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."],
}


def measure(text: str, scale: int, gap: int) -> int:
    w = 0
    for ch in text.upper():
        g = FONT.get(ch, FONT[" "])
        w += len(g[0]) * scale + gap
    return max(0, w - gap)


def paint(
    img: Image.Image,
    text: str,
    x: int,
    y: int,
    scale: int,
    gap: int,
    color=INK,
) -> None:
    cx = x
    for ch in text.upper():
        glyph = FONT.get(ch, FONT[" "])
        for gy, row in enumerate(glyph):
            for gx, cell in enumerate(row):
                if cell == "#":
                    for dy in range(scale):
                        for dx in range(scale):
                            px, py = cx + gx * scale + dx, y + gy * scale + dy
                            if 0 <= px < img.width and 0 <= py < img.height:
                                img.putpixel((px, py), color)
        cx += len(glyph[0]) * scale + gap


def card(w: int, h: int) -> Image.Image:
    img = Image.new("RGBA", (w, h), BG)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, w - 1, h - 1], outline=RULE, width=2)
    return img


def paste_mascot(img: Image.Image, frame_path: Path, x: int, y: int, size: int = 96) -> None:
    if not frame_path.exists():
        return
    fox = Image.open(frame_path).convert("RGBA")
    # scale nearest
    fox = fox.resize((size, size), Image.NEAREST)
    # composite on paper (transparent → paper)
    base = Image.new("RGBA", fox.size, BG)
    base.alpha_composite(fox)
    img.paste(base, (x, y), base)


def demo_unify_frames() -> list[Image.Image]:
    """Counters climb: noise filtered, keepers rise."""
    frames: list[Image.Image] = []
    stages = [
        (0, 0, 0, "SCAN"),
        (220, 40, 0, "FILTER"),
        (640, 180, 2, "SCORE"),
        (998, 809, 4, "KEEP"),
        (998, 809, 4, "KEEP"),
    ]
    for scanned, noise, keepers, phase in stages:
        img = card(720, 280)
        paint(img, "kit unify", 24, 18, 3, 2, ACCENT)
        paint(img, phase, 24, 52, 2, 1, MUTED)
        paint(img, f"scanned  {scanned}", 24, 100, 3, 2, INK)
        paint(img, f"noise    {noise}", 24, 140, 3, 2, MUTED)
        paint(img, f"keepers  {keepers}", 24, 180, 3, 2, GREEN if keepers else INK)
        # progress bar
        d = ImageDraw.Draw(img)
        bar_x, bar_y, bar_w, bar_h = 24, 230, 400, 14
        d.rectangle([bar_x, bar_y, bar_x + bar_w, bar_y + bar_h], outline=INK, width=1)
        fill = int(bar_w * min(1.0, scanned / 998 if scanned else 0.05))
        if fill > 2:
            d.rectangle(
                [bar_x + 1, bar_y + 1, bar_x + fill, bar_y + bar_h - 1],
                fill=ACCENT,
            )
        # mascot rail
        fi = min(6, max(1, 1 + stages.index((scanned, noise, keepers, phase)) % 6))
        paste_mascot(img, PIXEL / f"kit-frame-{fi}.png", 560, 80, 100)
        frames.append(img)
    return frames


def demo_ready_frames() -> list[Image.Image]:
    steps = [
        "recommend pack",
        "install skills",
        "apply project",
        "link agents",
        "doctor ok",
    ]
    frames: list[Image.Image] = []
    for n in range(len(steps) + 1):
        img = card(720, 300)
        paint(img, "kit ready", 24, 18, 3, 2, ACCENT)
        paint(img, "one shot setup", 24, 52, 2, 1, MUTED)
        y = 90
        for i, step in enumerate(steps):
            if i < n:
                paint(img, f"✓  {step}", 24, y, 2, 2, GREEN)
            elif i == n:
                paint(img, f">  {step}", 24, y, 2, 2, ACCENT)
            else:
                paint(img, f"·  {step}", 24, y, 2, 2, MUTED)
            y += 28
        paste_mascot(
            img,
            PIXEL / f"kit-frame-{min(6, n + 1)}.png",
            560,
            90,
            100,
        )
        frames.append(img)
    # hold final
    frames.append(frames[-1].copy())
    return frames


def demo_link_frames() -> list[Image.Image]:
    agents = ["claude-code", "codex", "grok-build"]
    frames: list[Image.Image] = []
    for n in range(len(agents) + 1):
        img = card(720, 260)
        paint(img, "kit link", 24, 18, 3, 2, ACCENT)
        paint(img, "library → agents", 24, 52, 2, 1, MUTED)
        paint(img, "from  ~/.kit/skills", 24, 90, 2, 2, INK)
        y = 130
        for i, a in enumerate(agents):
            mark = "✓" if i < n else "·"
            color = GREEN if i < n else MUTED
            paint(img, f"{mark}  {a}", 40, y, 2, 2, color)
            y += 28
        # fan lines
        d = ImageDraw.Draw(img)
        if n > 0:
            d.line([(200, 105), (220, 140)], fill=ACCENT, width=2)
        paste_mascot(img, PIXEL / f"kit-frame-{min(6, n + 1)}.png", 560, 70, 100)
        frames.append(img)
    frames.append(frames[-1].copy())
    return frames


def kit_success_frames() -> list[Image.Image]:
    """Bobbing fox + check — README close beat."""
    frames: list[Image.Image] = []
    bobs = [0, -4, 0, 4]
    for i, bob in enumerate(bobs):
        img = card(360, 280)
        fi = (i % 6) + 1
        paste_mascot(img, PIXEL / f"kit-frame-{fi}.png", 120, 60 + bob, 120)
        paint(img, "✓ ready", 110, 210, 3, 2, GREEN)
        frames.append(img)
    return frames


def packs_strip() -> Image.Image:
    names = [
        "essentials",
        "web-app",
        "library",
        "cli-tool",
        "api-service",
        "full-stack",
        "data-ml",
    ]
    cell = 88
    pad = 16
    label_h = 28
    w = pad * 2 + len(names) * cell + (len(names) - 1) * 12
    h = pad * 2 + cell + label_h + 20
    img = card(w, h)
    paint(img, "starter packs", pad, 10, 2, 1, MUTED)
    x = pad
    y = 40
    for name in names:
        icon_path = PACKS / f"{name}.png"
        if icon_path.exists():
            ic = Image.open(icon_path).convert("RGBA")
            ic = ic.resize((cell - 16, cell - 16), Image.NEAREST)
            base = Image.new("RGBA", ic.size, BG)
            base.alpha_composite(ic)
            img.paste(base, (x + 8, y + 4), base)
        else:
            d = ImageDraw.Draw(img)
            d.rectangle([x + 8, y + 4, x + cell - 8, y + cell - 12], outline=INK, width=2)
        # short label
        short = name.replace("-", " ")[:10]
        paint(img, short, x + 4, y + cell - 4, 1, 1, MUTED)
        x += cell + 12
    return img


def save_gif(frames: list[Image.Image], path: Path, duration: int = 420) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    # ensure palette-friendly
    converted = [f.convert("P", palette=Image.ADAPTIVE, colors=64) for f in frames]
    converted[0].save(
        path,
        save_all=True,
        append_images=converted[1:],
        duration=duration,
        loop=0,
        optimize=True,
    )
    print(f"wrote {path.relative_to(REPO)} ({len(frames)} frames)")


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    save_gif(demo_unify_frames(), OUT / "demo-unify.gif", duration=500)
    save_gif(demo_ready_frames(), OUT / "demo-ready.gif", duration=450)
    save_gif(demo_link_frames(), OUT / "demo-link.gif", duration=480)
    save_gif(kit_success_frames(), OUT / "kit-success.gif", duration=200)
    strip = packs_strip()
    strip_path = OUT / "packs-strip.png"
    strip.save(strip_path)
    print(f"wrote {strip_path.relative_to(REPO)}")

    # also export success frames as pixel masters for TUI load (optional PNG)
    # derive simple success bob from idle masters if present
    for i in range(1, 5):
        src = PIXEL / f"kit-frame-{((i - 1) % 6) + 1}.png"
        if src.exists():
            dest = PIXEL / f"kit-success-{i}.png"
            dest.write_bytes(src.read_bytes())
            print(f"wrote {dest.relative_to(REPO)}")
        scan_src = PIXEL / f"kit-frame-{i}.png"
        if scan_src.exists():
            dest = PIXEL / f"kit-scan-{i}.png"
            dest.write_bytes(scan_src.read_bytes())
            print(f"wrote {dest.relative_to(REPO)}")


if __name__ == "__main__":
    main()
