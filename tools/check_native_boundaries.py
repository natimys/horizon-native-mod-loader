#!/usr/bin/env python3
"""Audit complete Cargo dependency paths; diagnostic only, not a native sandbox."""
import argparse
import json
from pathlib import Path
import re
import subprocess

from native_modpack import ROOT, PackError


def audit(metadata, root=ROOT):
    packages = {p["id"]: p for p in metadata["packages"]}
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    native = (root / "native-mods").resolve()
    engine = (root / "horizon").resolve()
    mods = {i for i, p in packages.items()
            if Path(p["manifest_path"]).resolve().is_relative_to(native / "mods")}
    protected = {i for i, p in packages.items() if p["name"] in
                 {"horizon-native-api", "horizon-native-runtime", "horizon-native-ui"}}
    for start in sorted(mods | protected):
        start_name = packages[start]["name"]
        todo = [(start, [start_name])]
        seen = set()
        while todo:
            current, chain = todo.pop()
            if current in seen:
                continue
            seen.add(current)
            for edge in nodes[current]["deps"]:
                target = edge["pkg"]
                p = packages[target]
                path = Path(p["manifest_path"]).resolve()
                name = p["name"]
                allowed_native = {"horizon-native-api"}
                if start_name == "horizon-native-ui":
                    allowed_native.add("horizon-native-runtime")
                is_mod_library = target in mods and start in mods
                allowed_paths = [native / "api"]
                if start_name == "horizon-native-ui":
                    allowed_paths.append(native / "runtime")
                if start in mods:
                    allowed_paths.append(native / "mods")
                private_native = path.is_relative_to(native) and not any(path.is_relative_to(p) for p in allowed_paths)
                forbidden = (private_native or path.is_relative_to(engine)
                             or (name.startswith("horizon-") and name not in allowed_native and not is_mod_library)
                             or (start_name == "horizon-native-api"))
                if forbidden:
                    raise PackError("prohibited dependency: " + " -> ".join([*chain, name]))
                todo.append((target, [*chain, name]))
    # Full resolved graph must be locked. Direct third-party declarations must also
    # be exact; local path libraries are content-hashed by the source receipt.
    for p in packages.values():
        if not Path(p["manifest_path"]).resolve().is_relative_to(native):
            continue
        for dep in p["dependencies"]:
            if dep.get("path"):
                if not Path(dep["path"]).resolve().is_relative_to(native):
                    raise PackError(f"{p['name']}: local dependency escapes native workspace")
            elif dep.get("source", "").startswith("registry+"):
                if not re.fullmatch(r"=[0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?", dep["req"]):
                    raise PackError(f"{p['name']}: pin {dep['name']} with an exact =x.y.z version")
            else:
                raise PackError(f"{p['name']}: vendor {dep['name']} as reviewed local source; git dependencies unsupported")
    return len(mods)


def metadata(root=ROOT):
    return json.loads(subprocess.check_output([
        "cargo", "metadata", "--manifest-path", str(root / "native-mods/Cargo.toml"),
        "--all-features", "--format-version", "1", "--locked", "--offline"], cwd=root, text=True))


def feature_off(root=ROOT):
    tree = subprocess.check_output([
        "cargo", "tree", "--manifest-path", str(root / "horizon/Cargo.toml"),
        "-p", "horizon-client", "--edges", "normal", "--no-default-features",
        "--locked", "--offline", "--prefix", "none"], cwd=root, text=True)
    if any(line.startswith("horizon-native-") for line in tree.splitlines()):
        raise PackError("native dependency in feature-off Horizon graph")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--feature-off", action="store_true", help="also inspect the Horizon no-native production graph")
    args = parser.parse_args()
    try:
        count = audit(metadata())
        if args.feature_off:
            feature_off()
    except (PackError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"native-boundaries: {error}\n")
    print(f"native-boundaries: PASS ({count} independent mod crates)")


if __name__ == "__main__":
    main()
