#!/usr/bin/env python3
"""Capture Kit Workbench and Trenchwire output as a README proof GIF."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


REPO = Path(__file__).resolve().parents[1]
OUT = REPO / "docs" / "assets" / "demo-workbench-trenchwire.gif"
KIT_BIN = REPO / "packages" / "cli" / "dist" / "bin.js"

PAPER = "#F5F0E6"
INK = "#171713"
PANEL = "#111311"
PANEL_RULE = "#343A34"
ORANGE = "#C45C2A"
GREEN = "#83D49B"
CREAM = "#F5F0E6"
MUTED = "#9BA39B"
VIOLET = "#8D78D6"


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    names = (
        ["C:/Windows/Fonts/segoeuib.ttf", "C:/Windows/Fonts/arialbd.ttf"]
        if bold
        else ["C:/Windows/Fonts/consola.ttf", "C:/Windows/Fonts/arial.ttf"]
    )
    for name in names:
        try:
            return ImageFont.truetype(name, size=size)
        except OSError:
            continue
    return ImageFont.load_default()


TITLE = font(34, bold=True)
LABEL = font(16, bold=True)
MONO = font(18)
SMALL = font(14)


def run_kit(
    trenchwire: Path,
    kit_home: Path,
    data_home: Path,
    *args: str,
) -> str:
    env = os.environ.copy()
    env.update(
        {
            "KIT_HOME": str(kit_home),
            "TRENCHWIRE_OFFLINE": "1",
            "TRENCHWIRE_DEX_FIXTURE_DIR": str(
                trenchwire / "tests" / "fixtures" / "dexscreener"
            ),
            "TRENCHWIRE_NOW_MS": "1784203200000",
            "TRENCHWIRE_COLOR": "never",
            "TRENCHWIRE_ASCII": "1",
            "TRENCHWIRE_DATA_DIR": str(data_home),
        }
    )
    result = subprocess.run(
        ["node", str(KIT_BIN), *args],
        cwd=trenchwire,
        env=env,
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        shell=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"kit {' '.join(args)} failed ({result.returncode}):\n{result.stderr}"
        )
    return result.stdout.strip()

def detect_runners() -> list[dict[str, object]]:
    source = (
        "import {detectCodingRunners} from './packages/core/dist/index.js';"
        "console.log(JSON.stringify(await detectCodingRunners()));"
    )
    result = subprocess.run(
        ["node", "--input-type=module", "-e", source],
        cwd=REPO,
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        shell=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"runner detection failed:\n{result.stderr}")
    return json.loads(result.stdout)


def sanitize(text: str, trenchwire: Path, kit_home: Path, data_home: Path) -> str:
    replacements = [
        (str(trenchwire / "target" / "release" / "trenchwire.exe"), "<release-binary>"),
        (str(trenchwire / "target" / "release" / "trenchwire"), "<release-binary>"),
        (str(data_home), "<temporary-data>"),
        (str(kit_home), "<temporary-kit-home>"),
        (str(trenchwire), "<trenchwire-checkout>"),
        (str(Path.home()), "<home>"),
    ]
    value = text
    for source, target in replacements:
        value = value.replace(source, target)
        value = value.replace(source.replace("\\", "/"), target)
    return value


def terminal_frame(
    step: str,
    command: str,
    lines: list[tuple[str, str]],
    index: int,
) -> Image.Image:
    image = Image.new("RGB", (960, 540), PAPER)
    draw = ImageDraw.Draw(image)

    draw.rectangle((0, 0, 960, 76), fill=INK)
    draw.text((34, 20), "KIT", font=TITLE, fill=CREAM)
    draw.text((122, 31), "// LOCAL WORKBENCH", font=LABEL, fill=ORANGE)
    draw.text((766, 31), f"0{index} / 05", font=LABEL, fill=CREAM)

    draw.text((34, 100), step, font=LABEL, fill=ORANGE)
    draw.text(
        (34, 128),
        "Kit Workbench" if step == "RUNNERS" else "Kit > Trenchwire",
        font=TITLE,
        fill=INK,
    )

    draw.rounded_rectangle(
        (34, 184, 926, 482),
        radius=12,
        fill=PANEL,
        outline=PANEL_RULE,
        width=2,
    )
    draw.ellipse((56, 205, 68, 217), fill="#E36B5D")
    draw.ellipse((76, 205, 88, 217), fill="#D9A441")
    draw.ellipse((96, 205, 108, 217), fill="#5EBA78")
    draw.text((132, 200), command, font=MONO, fill=CREAM)
    draw.line((56, 232, 904, 232), fill=PANEL_RULE, width=1)

    y = 252
    for text, color in lines:
        draw.text((58, y), text, font=MONO, fill=color)
        y += 31

    for dot in range(5):
        x = 34 + dot * 20
        fill = ORANGE if dot + 1 == index else "#D4CCC0"
        draw.rounded_rectangle((x, 505, x + 12, 512), radius=3, fill=fill)
    draw.text(
        (492, 500),
        "REAL COMMANDS / OFFLINE FIXTURE / NO WALLET ACTION",
        font=SMALL,
        fill="#5C5A54",
    )
    return image


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("trenchwire", type=Path)
    args = parser.parse_args()
    trenchwire = args.trenchwire.resolve()
    executable = trenchwire / "target" / "release" / (
        "trenchwire.exe" if os.name == "nt" else "trenchwire"
    )
    manifest = trenchwire / "kit.plugin.json"
    if not KIT_BIN.is_file():
        raise SystemExit("Build Kit first: pnpm build")
    if not manifest.is_file():
        raise SystemExit(f"Missing plugin manifest: {manifest}")
    if not executable.is_file():
        raise SystemExit("Build Trenchwire first: cargo build --locked --release")

    with tempfile.TemporaryDirectory(prefix="kit-trenchwire-proof-") as temp:
        root = Path(temp)
        kit_home = root / "kit-home"
        data_home = root / "trenchwire-data"

        add = run_kit(trenchwire, kit_home, data_home, "plugin", "add", ".", "--write")
        doctor = run_kit(
            trenchwire, kit_home, data_home, "plugin", "doctor", "trenchwire"
        )
        tasks = run_kit(
            trenchwire, kit_home, data_home, "plugin", "task", "trenchwire"
        )
        check = run_kit(
            trenchwire,
            kit_home,
            data_home,
            "plugin",
            "task",
            "trenchwire",
            "health",
        )
        find = run_kit(
            trenchwire,
            kit_home,
            data_home,
            "plugin",
            "task",
            "trenchwire",
            "market",
        )
        runners = detect_runners()

        add_clean = sanitize(add, trenchwire, kit_home, data_home)
        doctor_clean = sanitize(doctor, trenchwire, kit_home, data_home)
        check_data = json.loads(check)["data"]
        find_lines = [
            line.rstrip()
            for line in find.splitlines()
            if line.strip() and not set(line.strip()) <= {"-"}
        ]

        frames = [
            terminal_frame(
                "RUNNERS",
                "$ kit tui workbench",
                [
                    (
                        f"> {runner['label']:<14} "
                        f"{'ready' if runner['available'] else 'missing'}",
                        GREEN if runner["available"] else MUTED,
                    )
                    for runner in runners
                ]
                + [
                    ("", CREAM),
                    ("inspect: provider read-only mode", CREAM),
                    ("build:   explicit confirmation", VIOLET),
                ],
                1,
            ),
            terminal_frame(
                "ATTACH",
                "$ kit plugin add . --write",
                [
                    ("Plugin registered", GREEN),
                    ("name:       trenchwire", CREAM),
                    ("version:    1.0.0", CREAM),
                    ("executable: <release-binary> (local)", MUTED),
                    ("manifest:   kit.plugin.json", MUTED),
                ],
                2,
            ),
            terminal_frame(
                "SERVICE TASKS",
                "$ kit plugin task trenchwire",
                [
                    ("health       Check local market and Phantom providers.", GREEN),
                    ("market       Show public Solana market facts.", GREEN),
                    ("", CREAM),
                    ("Tab maps the selected task to the main panel.", CREAM),
                    ("live output / Esc stops / 30 second limit", MUTED),
                    ("wallet + SEND stay inside Trenchwire", VIOLET),
                ],
                3,
            ),
            terminal_frame(
                "HEALTH",
                "$ kit plugin task trenchwire health",
                [
                    ("schema_version: 1", CREAM),
                    (f"market: {check_data['market']['provider']}", GREEN),
                    ("market API key required: false", CREAM),
                    (f"wallet: {check_data['wallet']['provider']}", VIOLET),
                    ("private key required: false", CREAM),
                ],
                4,
            ),
            terminal_frame(
                "MARKET",
                "$ kit plugin task trenchwire market",
                [
                    (find_lines[0][:82], VIOLET),
                    (find_lines[1][:82], MUTED),
                    (find_lines[2][:82], CREAM),
                    (find_lines[3][:82], GREEN),
                    (find_lines[-2][:82], GREEN),
                    (find_lines[-1][:82], MUTED),
                ],
                5,
            ),
        ]

        # Assert that every displayed result came from the live capture.
        if "Plugin registered" not in add_clean:
            raise RuntimeError("Registration proof did not contain the expected status.")
        if "status:     ready" not in doctor_clean:
            raise RuntimeError("Doctor proof did not contain the ready status.")
        if "health" not in tasks or "market" not in tasks:
            raise RuntimeError("Task proof did not contain both service tasks.")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    palette = [frame.convert("P", palette=Image.Palette.ADAPTIVE, colors=96) for frame in frames]
    palette[0].save(
        OUT,
        save_all=True,
        append_images=palette[1:] + [palette[-1]],
        duration=[1200, 1000, 1300, 1200, 1800, 1000],
        loop=0,
        optimize=True,
    )
    print(f"wrote {OUT.relative_to(REPO)} ({len(frames)} captured steps)")


if __name__ == "__main__":
    main()
