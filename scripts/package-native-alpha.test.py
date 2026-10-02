#!/usr/bin/env python3
"""One offline archive regression; no Claude calls or retained release edits."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest


class NativeAlphaPackage(unittest.TestCase):
    def test_roundtrip_determinism_and_preservation(self):
        sys.dont_write_bytecode = True
        spec = importlib.util.spec_from_file_location("alpha", Path(__file__).with_name("package-native-alpha.py"))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory(prefix="kit-alpha-test-") as temp:
            root = Path(temp).resolve()
            archive = module.package(root / "first" / "candidate")
            repeated = module.package(root / "second" / "candidate")
            self.assertEqual(archive.read_bytes(), repeated.read_bytes())
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            self.assertEqual(archive.with_name(archive.name + ".sha256").read_text(), f"{digest}  {archive.name}\n")
            extract = root / "extract"
            extract.mkdir()
            with tarfile.open(archive) as bundle:
                self.assertTrue(all(member.isfile() and not Path(member.name).is_absolute() and ".." not in Path(member.name).parts for member in bundle.getmembers()))
                bundle.extractall(extract)
            lines = (extract / "FILES.sha256").read_text().splitlines()
            self.assertEqual({line.split("  ", 1)[1] for line in lines}, {
                p.relative_to(extract).as_posix() for p in extract.rglob("*")
                if p.is_file() and p.name != "FILES.sha256"
            })
            for line in lines:
                expected, name = line.split("  ", 1)
                self.assertEqual(hashlib.sha256((extract / name).read_bytes()).hexdigest(), expected)
            build = json.loads((extract / "BUILD.json").read_text())
            manifest = json.loads((extract / "kit/.claude-plugin/plugin.json").read_text())
            self.assertEqual(build["version"], manifest["version"])
            self.assertRegex(build["baseCommit"], r"^[a-f0-9]{40}$")
            self.assertIsInstance(build["sourceDirty"], bool)
            self.assertEqual((extract / "START-HERE.md").read_bytes(), (module.ROOT / "docs/dev/NATIVE-ALPHA.md").read_bytes())
            source = module.ROOT / "crates/kit-cli/claude-plugin"
            payload = [p for p in (extract / "kit").rglob("*") if p.is_file()]
            self.assertEqual(len(payload), build["payloadFiles"])
            for file in payload:
                self.assertEqual(file.read_bytes(), (source / file.relative_to(extract / "kit")).read_bytes())
            self.assertFalse((extract / "kit/tests").exists())
            self.assertEqual(sorted(p.name for p in (extract / "kit/.claude-plugin").iterdir()), ["plugin.json"])
            before = archive.read_bytes()
            with self.assertRaises(FileExistsError):
                module.package(root / "first" / "candidate")
            with self.assertRaises(FileExistsError):
                module.package(root / "first" / "other-candidate")
            self.assertEqual(archive.read_bytes(), before)
            self.assertFalse((root / "first/other-candidate").exists())
            linked = root / "linked"
            linked.symlink_to(root / "first", target_is_directory=True)
            with self.assertRaises(ValueError):
                module.package(linked / "candidate")


if __name__ == "__main__":
    unittest.main()
