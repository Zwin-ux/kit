#!/usr/bin/env python3
"""
Iconic product ads for GitHub / npm README.
Pixel terminal cards on paper — show, don't tell.
"""
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

REPO = Path(__file__).resolve().parents[1]
OUT = REPO / "docs" / "assets"

BG = (247, 244, 237, 255)
INK = (18, 18, 18, 255)
MUTED = (90, 86, 78, 255)
ACCENT = (196, 92, 42, 255)
RULE = (210, 204, 192, 255)
GREEN = (46, 125, 50, 255)

# Reuse 5x7 font from banner generator (subset + digits)
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
                if cell != "#":
                    continue
                for sy in range(scale):
                    for sx in range(scale):
                        px, py = cx + gx * scale + sx, y + gy * scale + sy
                        if 0 <= px < img.width and 0 <= py < img.height:
                            img.putpixel((px, py), color)
        cx += len(glyph[0]) * scale + gap


def card(w: int, h: int) -> Image.Image:
    img = Image.new("RGBA", (w, h), BG)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, w - 1, 5], fill=ACCENT)
    d.rounded_rectangle([12, 16, w - 13, h - 13], radius=10, outline=RULE, width=2)
    return img


def save(img: Image.Image, name: str) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / name
    img.save(path, "PNG")
    print("wrote", path, img.size)


def ad_install() -> None:
    W, H = 920, 220
    img = card(W, H)
    paint(img, "NPM I -G @MZWIN/KIT", 48, 48, 3, 2, MUTED)
    paint(img, "$  KIT", 48, 100, 6, 4, INK)
    paint(img, "ONE LIBRARY. MANY AGENTS.", 48, 160, 2, 2, ACCENT)
    save(img, "ad-install.png")


def ad_unify() -> None:
    W, H = 920, 420
    img = card(W, H)
    paint(img, "KIT UNIFY", 48, 40, 4, 3, INK)
    paint(img, "SKILL OS", 320, 48, 3, 2, ACCENT)

    lines = [
        (MUTED, "SCANNED   998  FOLDERS"),
        (MUTED, "NOISE     809  FILTERED"),
        (GREEN, "KEEPERS     4  GRADE A+"),
        (INK, ""),
        (INK, "+  CAREFUL          CLAUDE+CODEX"),
        (INK, "+  FREEZE           CLAUDE+CODEX"),
        (MUTED, "X  ABLY-AUTOMATION  NOISE"),
        (MUTED, "X  * 800 MORE DUMPS"),
    ]
    y = 110
    for color, line in lines:
        if line:
            paint(img, line, 48, y, 2, 2, color)
        y += 32
    save(img, "ad-unify.png")


def ad_ready() -> None:
    W, H = 920, 360
    img = card(W, H)
    paint(img, "KIT READY --WRITE", 48, 40, 4, 3, INK)
    y = 110
    steps = [
        (GREEN, "OK  PACK INSTALL     ESSENTIALS"),
        (GREEN, "OK  PACK APPLY       ./MY-APP"),
        (GREEN, "OK  LINK             CLAUDE CODEX GROK"),
        (GREEN, "OK  DOCTOR           GREEN"),
    ]
    for color, line in steps:
        paint(img, line, 48, y, 2, 2, color)
        y += 40
    paint(img, "THIS REPO IS AGENT-READY.", 48, 300, 2, 2, ACCENT)
    save(img, "ad-ready.png")


def ad_link() -> None:
    W, H = 920, 280
    img = card(W, H)
    paint(img, "KIT LINK --TO ALL --WRITE", 48, 40, 3, 2, INK)
    paint(img, "~/.KIT/SKILLS", 48, 110, 3, 2, MUTED)
    paint(img, "        |", 48, 150, 2, 2, RULE)
    paint(img, "   +----+----+", 48, 175, 2, 2, RULE)
    paint(img, "   V    V    V", 48, 200, 2, 2, MUTED)
    paint(img, "CLAUDE CODEX GROK", 48, 230, 3, 3, ACCENT)
    save(img, "ad-link.png")


def ad_strip() -> None:
    """Command strip for README mid-section."""
    W, H = 920, 120
    img = card(W, H)
    paint(img, "KIT   ·   KIT READY --WRITE   ·   KIT UNIFY --WRITE", 36, 52, 2, 2, INK)
    save(img, "ad-commands.png")


def main() -> None:
    ad_install()
    ad_unify()
    ad_ready()
    ad_link()
    ad_strip()
    print("done →", OUT)


if __name__ == "__main__":
    main()
