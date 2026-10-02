#!/usr/bin/env python3
"""Prepare a reproducible private alpha with the existing Node exporter; never call Claude."""
import argparse
from datetime import datetime, timezone
import gzip
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "crates/kit-cli/claude-plugin"


def package(destination):
    out = Path(destination).absolute()
    if any(path.is_symlink() for path in (out, *out.parents)):
        raise ValueError("Choose an output path without symbolic links")
    out = out.resolve()
    if out.is_relative_to(SOURCE):
        raise ValueError("Output must be outside the plugin source")
    manifest = json.loads((SOURCE / ".claude-plugin/plugin.json").read_text())
    version = manifest["version"]
    if not re.fullmatch(r"\d+\.\d+\.\d+-alpha\.\d+", version):
        raise ValueError("Only private alpha versions can use this packager")
    guide = (ROOT / "docs/dev/NATIVE-ALPHA.md").read_bytes()
    if not guide.startswith(f"# Kit private alpha — {version}\n".encode()):
        raise ValueError("Alpha guide and plugin version disagree")
    archive = out.parent / f"kit-claude-{version}.tar.gz"
    checksum = archive.with_name(archive.name + ".sha256")
    for path in (out, archive, checksum):
        if path.exists() or path.is_symlink():
            raise FileExistsError(f"{path} exists; choose a fresh output location")
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    epoch = int(subprocess.check_output(["git", "show", "-s", "--format=%ct", commit], cwd=ROOT, text=True))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.mkdir()
    created = []
    try:
        subprocess.run(["node", str(ROOT / "scripts/sync-claude-plugin.mjs"), "--out", str(out / "kit")], cwd=ROOT, check=True, capture_output=True, text=True)
        files = sorted((out / "kit").rglob("*"))
        if any(file.is_symlink() or not (file.is_dir() or file.is_file()) for file in files):
            raise ValueError("Payload contains a link or non-regular entry")
        payload = [file for file in files if file.is_file()]
        for file in payload:
            if file.read_bytes() != (SOURCE / file.relative_to(out / "kit")).read_bytes():
                raise ValueError(f"Source changed during export: {file.relative_to(out)}")
        exported = json.loads((out / "kit/.claude-plugin/plugin.json").read_text())
        provenance = json.loads((out / "kit/provenance.json").read_text())
        if exported != manifest or provenance["version"] != version or guide != (ROOT / "docs/dev/NATIVE-ALPHA.md").read_bytes():
            raise ValueError("Metadata or guide changed during export")
        dirty = bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT))
        build = {
            "name": "kit-native-private-alpha", "version": version,
            "baseCommit": commit, "sourceDirty": dirty,
            # Commit time fixes tar/gzip metadata; it is not a build/acceptance timestamp.
            "sourceCommitAtUtc": datetime.fromtimestamp(epoch, timezone.utc).isoformat(),
            "reproducibleArchiveEpoch": epoch,
            "distribution": "local only; not published", "nativeAcceptance": "pending",
            "payloadFiles": len(payload),
            "skillResourceFiles": sum(file.is_relative_to(out / "kit/skills") for file in payload),
            "roleCount": len(exported["agents"]), "skillCount": len(provenance["skills"]),
        }
        (out / "START-HERE.md").write_bytes(guide)
        (out / "BUILD.json").write_text(json.dumps(build, indent=2) + "\n")
        members = sorted(file for file in out.rglob("*") if file.is_file())
        (out / "FILES.sha256").write_text("".join(
            f"{hashlib.sha256(file.read_bytes()).hexdigest()}  {file.relative_to(out).as_posix()}\n"
            for file in members))
        members = sorted(file for file in out.rglob("*") if file.is_file())
        with archive.open("xb") as raw:
            created.append(archive)
            with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=epoch) as compressed:
                with tarfile.open(fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT) as bundle:
                    for file in members:
                        data = file.read_bytes()
                        info = tarfile.TarInfo(file.relative_to(out).as_posix())
                        info.size, info.mtime, info.mode = len(data), epoch, 0o644
                        bundle.addfile(info, io.BytesIO(data))
        with checksum.open("x") as sums:
            created.append(checksum)
            sums.write(f"{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n")
        return archive
    except BaseException:
        for path in created:
            path.unlink()
        shutil.rmtree(out)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, type=Path, help="fresh extracted-candidate directory; archive/checksum go beside it")
    print(package(parser.parse_args().out))
