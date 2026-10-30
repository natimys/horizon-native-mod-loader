import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import native_pack_release as release
from native_modpack import PackError


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        self.write("native-mods/api/src/lib.rs", "original")
        self.write("native-modpack.lock", "{}")
        self.write("tools/check_native_boundaries.py", "# architectural guard")
        self.write("native-modpack.toml", "schema = 1")
        self.write("game-resources/private.txt", "PRIVATE")
        self.write("native-mods/target/secret", "BUILD OUTPUT")
        subprocess.run(["git", "add", "native-mods/api", "native-modpack.lock", "native-modpack.toml", "tools/check_native_boundaries.py"], cwd=self.root, check=True)
        subprocess.run(["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "fixture"], cwd=self.root, check=True)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def test_package_determinism_allowlist_checksums_and_no_overwrite(self):
        with patch.object(release, "verify", return_value={"horizon": "abc", "rustc": "r", "cargo": "c", "host": "test"}):
            a, b = self.root / "a.zip", self.root / "b.zip"
            self.assertEqual(release.package(a, self.root), release.package(b, self.root))
            with self.assertRaises(FileExistsError):
                release.package(a, self.root)
        with zipfile.ZipFile(a) as archive:
            self.assertIn("tools/check_native_boundaries.py", archive.namelist())
            self.assertFalse(any("private" in n or "target" in n for n in archive.namelist()))
            for line in archive.read("SHA256SUMS").decode().splitlines():
                expected, name = line.split("  ", 1)
                self.assertEqual(expected, release.digest(archive.read(name)))
            self.assertEqual(json.loads(archive.read("BUILD.json"))["kind"], "native-source-recipe")

    def test_dirty_release_refused(self):
        self.write("native-mods/api/src/lib.rs", "changed")
        with patch.object(release, "verify", return_value={}):
            with self.assertRaisesRegex(PackError, "commit reviewed"):
                release.package(self.root / "a.zip", self.root)

    def test_lock_drift_and_no_implicit_relock(self):
        expected = {"schema": 1, "sha256": {"a": "old"}, "horizon": "one"}
        self.write("native-modpack.lock", json.dumps(expected))
        with patch.object(release, "snapshot", return_value={"sha256": {"a": "new"}, "horizon": "two"}):
            with self.assertRaisesRegex(PackError, "stale source lock"):
                release.verify(self.root)
        self.assertEqual(json.loads((self.root / "native-modpack.lock").read_text()), expected)

    def test_malformed_lock_fails_with_actionable_error(self):
        for value in [[], {"schema": 99, "sha256": {}}, {"schema": 1, "sha256": []}]:
            self.write("native-modpack.lock", json.dumps(value))
            with self.assertRaisesRegex(PackError, "malformed source lock"):
                release.verify(self.root)

    def test_symlinks_and_prohibited_tracked_files(self):
        (self.root / "link").symlink_to(self.root / "game-resources", target_is_directory=True)
        with self.assertRaisesRegex(PackError, "symlink"):
            release.safe_read(self.root, "link/private.txt")
        self.write("native-mods/forbidden.jar", "private bytes")
        subprocess.run(["git", "add", "native-mods/forbidden.jar"], cwd=self.root, check=True)
        with self.assertRaisesRegex(PackError, "prohibited"):
            release.package_files(self.root)

    def test_untracked_native_source_refused(self):
        self.write("native-mods/ignored.rs", "untracked source")
        with self.assertRaisesRegex(PackError, "untracked native source"):
            release.source_files(self.root)

    def test_build_rejects_configuration_before_cargo(self):
        with patch.object(release, "verify", return_value={}), patch.object(release, "recipe", return_value={"mods": []}):
            for variant, features in [("native", "unknown"), ("vanilla", "gameplay-example")]:
                with self.assertRaises(PackError):
                    release.build(variant, features, False, self.root)


if __name__ == "__main__":
    unittest.main()
