#!/usr/bin/env python3
"""Build the Zwin-ux/kits index repo from a Kit release commit.

    python3 scripts/kits-index-generate.py <kit checkout> <release ref> <out dir>

Needs Python 3.11+ (tomllib) and git.

Reads every bundled kit in crates/kit-cli/kits/ at <release ref> (so a kit
added by a merged PR, like ios-apple-design, is picked up with no edit
here), and writes <out dir>/kits-index/ plus <out dir>/kits-index.bundle:
commit 1 holds kits/, README.md and LICENSE; commit 2 adds index.toml
pinning every kit to commit 1. Push the bundle's history, never the loose
folder, or every rev dangles. Deterministic: same input, same shas.
"""
import os, shutil, subprocess, sys, tempfile, tomllib
from pathlib import Path

# Search tags per kit. A bundled kit missing here is listed with no tags
# and a warning, never dropped.
TAGS = {
    "essentials": ["starter", "workflow", "testing", "review"],
    "frontend-design": ["frontend", "design", "ui", "accessibility", "web"],
    "fullstack-design": ["fullstack", "frontend", "backend", "design", "api"],
    "backend-engineer": ["backend", "api", "database", "security", "postgres"],
    "llm-engineer": ["llm", "ai", "evals", "prompts", "mcp"],
    "ios-apple-design": ["ios", "apple", "swift", "swiftui", "design", "macos"],
}
HERE = Path(__file__).resolve().parent
ENV = {**os.environ, "GIT_AUTHOR_NAME": "Kit", "GIT_AUTHOR_EMAIL": "noreply@github.com",
       "GIT_COMMITTER_NAME": "Kit", "GIT_COMMITTER_EMAIL": "noreply@github.com"}


def git(cwd, *args, date=None, out=False):
    env = dict(ENV)
    if date:
        env["GIT_AUTHOR_DATE"] = env["GIT_COMMITTER_DATE"] = date
    r = subprocess.run(["git", "-c", "commit.gpgsign=false", *args], cwd=cwd, env=env,
                       check=True, capture_output=True)
    return r.stdout.decode().strip() if out else None


def q(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def main(src, ref, out):
    src, out = Path(src), Path(out)
    sha = git(src, "rev-parse", ref, out=True)
    date = git(src, "show", "-s", "--format=%cI", sha, out=True)
    with tempfile.TemporaryDirectory() as tmp:
        work = Path(tmp) / "kits"
        work.mkdir()
        tar = subprocess.run(["git", "archive", sha, "crates/kit-cli/kits", "LICENSE"],
                             cwd=src, check=True, capture_output=True).stdout
        subprocess.run(["tar", "-x", "-C", tmp], input=tar, check=True)
        shutil.move(Path(tmp) / "crates/kit-cli/kits", work / "kits")
        shutil.move(Path(tmp) / "LICENSE", work / "LICENSE")
        kits = []
        for d in sorted((work / "kits").iterdir()):
            meta = tomllib.loads((d / "KIT.toml").read_text())["kit"]
            if meta["name"] != d.name:
                sys.exit(f"{d.name}: KIT.toml names {meta['name']}")
            if d.name not in TAGS:
                print(f"warning: no tags for {d.name}; add it to TAGS", file=sys.stderr)
            kits.append(meta)
        titles = ", ".join(k["title"] for k in kits)
        readme = (HERE / "kits-index-README.template.md").read_text()
        (work / "README.md").write_text(readme.replace("{KITS}", titles))
        git(work, "init", "-q", "-b", "main")
        git(work, "add", "kits", "README.md", "LICENSE")
        git(work, "commit", "-q", "-m", f"The kits that ship with Kit, from {sha[:7]}", date=date)
        pin = git(work, "rev-parse", "HEAD", out=True)
        lines = ["# The kit index. See README.md for how a kit gets listed.", "schema = 1"]
        for k in kits:
            tags = ", ".join(q(t) for t in TAGS.get(k["name"], []))
            lines += ["", "[[kit]]", f"name    = {q(k['name'])}", f"title   = {q(k['title'])}",
                      f"summary = {q(k['description'])}", 'source  = "github:Zwin-ux/kits"',
                      f"path    = {q('kits/' + k['name'])}", f"rev     = {q(pin)}",
                      'level   = "official"', f"tags    = [{tags}]"]
        (work / "index.toml").write_text("\n".join(lines) + "\n")
        git(work, "add", "index.toml")
        git(work, "commit", "-q", "-m", f"index.toml: list {len(kits)} kits", date=date)
        dest = out / "kits-index"
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(work, dest, ignore=shutil.ignore_patterns(".git"))
        git(work, "bundle", "create", "-q", str((out / "kits-index.bundle").resolve()), "main")
        print(f"{len(kits)} kits from {sha[:7]}: {', '.join(k['name'] for k in kits)}")
        print(f"kits pinned at {pin}; bundle {out / 'kits-index.bundle'}")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    main(*sys.argv[1:])
