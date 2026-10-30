#!/usr/bin/env python3
"""Roundtrip the actual source recipe using a temporary local engine checkout."""
from pathlib import Path
import hashlib
import json
import subprocess
import tempfile
import zipfile

from native_modpack import ROOT
from native_pack_release import package


def main():
    with tempfile.TemporaryDirectory(prefix="native-package-") as tmp:
        scratch = Path(tmp)
        a, b = scratch / "a.zip", scratch / "b.zip"
        assert package(a) == package(b), "source recipe differs across repeated packaging"
        root = scratch / "source"
        root.mkdir()
        with zipfile.ZipFile(a) as archive:
            for line in archive.read("SHA256SUMS").decode().splitlines():
                checksum, name = line.split("  ", 1)
                assert checksum == hashlib.sha256(archive.read(name)).hexdigest()
            archive.extractall(root)
        subprocess.run(["git", "init", "-q", str(root)], check=True)
        subprocess.run(["git", "add", "native-mods", "docs", "tools", "tests", ".github",
                        "native-modpack.toml", "native-modpack.lock"], cwd=root, check=True)
        subprocess.run(["git", "add", "LICENSE", "THIRD_PARTY_NOTICES.md"], cwd=root, check=True)
        subprocess.run(["git", "-c", "user.name=Recipe test", "-c", "user.email=test@example.invalid",
                        "commit", "-qm", "reviewed source recipe"], cwd=root, check=True)
        subprocess.run(["git", "clone", "--shared", "--no-checkout", str(ROOT / "horizon"), str(root / "horizon")], check=True)
        pin = json.loads((root / "native-modpack.lock").read_text())["horizon"]
        subprocess.run(["git", "checkout", "--detach", pin], cwd=root / "horizon", check=True)
        for command in [
            ["python3", "tools/native_pack_release.py", "verify"],
            ["python3", "tools/check_native_boundaries.py"],
            ["cargo", "test", "--manifest-path", "native-mods/Cargo.toml", "--workspace", "--all-features", "--locked", "--offline"],
        ]:
            subprocess.run(command, cwd=root, check=True)
    print("source-recipe-roundtrip PASS")


if __name__ == "__main__":
    main()
