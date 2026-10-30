#!/usr/bin/env python3
"""Fail closed on a missing/mismatched engine submodule or source recipe pin."""
import json
from pathlib import Path
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
URL = "https://github.com/natimys/horizon-native-engine.git"


def output(*args, root=ROOT):
    return subprocess.check_output(args, cwd=root, text=True).strip()


def main():
    pin = json.loads((ROOT / "native-modpack.lock").read_text())["horizon"]
    entry = output("git", "ls-files", "--stage", "--", "horizon").split()
    if entry != ["160000", pin, "0", "horizon"]:
        raise SystemExit("engine gitlink differs from native-modpack.lock (or is missing)")
    path = output("git", "config", "-f", ".gitmodules", "--get", "submodule.horizon.path")
    url = output("git", "config", "-f", ".gitmodules", "--get", "submodule.horizon.url")
    if path != "horizon" or url != URL:
        raise SystemExit("unexpected public engine submodule path/URL")
    if output("git", "rev-parse", "HEAD", root=ROOT / "horizon") != pin:
        raise SystemExit("engine checkout differs from pinned gitlink; initialize/update submodules")
    output("git", "cat-file", "-e", pin + "^{commit}", root=ROOT / "horizon")
    rust = tomllib.loads((ROOT / "native-modpack.toml").read_text())["rust"]
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    if toolchain != rust:
        raise SystemExit("rust-toolchain.toml differs from native recipe")
    print("Engine submodule URL, gitlink, checkout, source lock and toolchain: PASS")


if __name__ == "__main__":
    main()
