#!/usr/bin/env python3
"""Compile the API-only author template without adding it to the real pack."""
from pathlib import Path
import shutil
import subprocess
import tempfile

from native_modpack import ROOT


def main():
    with tempfile.TemporaryDirectory(prefix="native-author-") as tmp:
        root = Path(tmp)
        shutil.copytree(ROOT / "native-mods/api", root / "api")
        crate = root / "mods/example"
        shutil.copytree(ROOT / "docs/native-modding/templates/basic", crate)
        (root / "Cargo.toml").write_text('[workspace]\nmembers = ["api", "mods/example"]\nresolver = "2"\n')
        subprocess.run(["cargo", "generate-lockfile", "--manifest-path", str(root / "Cargo.toml"), "--offline"], check=True)
        subprocess.run(["cargo", "test", "--manifest-path", str(root / "Cargo.toml"), "--workspace", "--locked", "--offline"], check=True)
        subprocess.run(["cargo", "clippy", "--manifest-path", str(root / "Cargo.toml"), "--workspace", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"], check=True)
    print("author-template PASS")


if __name__ == "__main__":
    main()
