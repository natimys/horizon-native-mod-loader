#!/usr/bin/env python3
"""Lock, verify, build and package reviewed native source recipes (no game assets)."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile

from native_modpack import ROOT, PackError, generate, recipe

LOCK = "native-modpack.lock"


def run(args, root=ROOT):
    return subprocess.check_output(args, cwd=root, text=True).strip()


def digest(value):
    return hashlib.sha256(value).hexdigest()


def tracked(root, *paths):
    return sorted(filter(None, run(["git", "ls-files", "-z", "--", *paths], root).split("\0")))


def source_files(root):
    files = tracked(root, "native-mods")
    # Untracked source can still be consumed by Rust modules/include_bytes/build.rs.
    extra = run(["git", "ls-files", "--others", "--exclude=native-mods/target/", "--", "native-mods"], root)
    if extra:
        raise PackError("untracked native source: review and explicitly stage/commit it first")
    if not files:
        raise PackError("missing tracked native source")
    return files


def safe_read(root, name):
    relative = Path(name)
    if relative.is_absolute() or ".." in relative.parts:
        raise PackError("unsafe package path")
    if any((root / Path(*relative.parts[:i])).is_symlink() for i in range(1, len(relative.parts) + 1)):
        raise PackError(f"symlink source refused: {name}")
    return (root / relative).read_bytes()


def snapshot(root=ROOT):
    generate(root / "native-modpack.toml", check=True, root=root)
    config = recipe(root / "native-modpack.toml", root)
    rust = run(["rustc", "--version"], root)
    cargo = run(["cargo", "--version"], root)
    if rust.split()[1] != config["rust"] or cargo.split()[1] != config["rust"]:
        raise PackError(f"toolchain mismatch: recipe requires rustc/Cargo {config['rust']}")
    engine = root / "horizon"
    if run(["git", "diff", "HEAD", "--name-only"], engine):
        raise PackError("tracked Horizon changes must be reviewed and committed before locking/building")
    files = ["native-modpack.toml", "horizon/Cargo.lock", *source_files(root),
             *tracked(root, "tools/native_modpack.py", "tools/native_pack_release.py", "tools/check_native_boundaries.py")]
    return {"schema": 1, "api": config["api"], "rustc": rust, "cargo": cargo,
            "host": next(line[6:] for line in run(["rustc", "-vV"], root).splitlines() if line.startswith("host: ")),
            "horizon": run(["git", "rev-parse", "HEAD"], engine),
            "sha256": {name: digest(safe_read(root, name)) for name in sorted(files)}}


def encode(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def verify(root=ROOT):
    expected = json.loads((root / LOCK).read_text())
    if not isinstance(expected, dict) or expected.get("schema") != 1 or not isinstance(expected.get("sha256"), dict):
        raise PackError("unsupported or malformed source lock; expected schema 1 with sha256 table")
    actual = snapshot(root)
    if expected != actual:
        changes = [key for key in actual if expected.get(key) != actual[key]]
        changed_files = [name for name, value in actual["sha256"].items()
                         if expected.get("sha256", {}).get(name) != value]
        raise PackError("stale source lock: " + ", ".join(changes + changed_files) + "; review changes, then run lock")
    return actual


def package_files(root):
    names = tracked(root, "native-mods", "docs/native-modding", "tools/native*.py",
                    "tests/test_native*.py", "tools/check_native_boundaries.py", ".github/workflows/native.yml",
                    "native-modpack.toml", LOCK, "LICENSE", "THIRD_PARTY_NOTICES.md",
                    "README.md", "CONTRIBUTING.md", "SECURITY.md", "CODE_OF_CONDUCT.md")
    # Explicit source-only export. Never recurse into resource homes or engine checkout.
    forbidden = {"game-resources", ".local", "target", "saves", ".aws", ".env", "__pycache__"}
    for name in names:
        if forbidden.intersection(Path(name).parts) or Path(name).suffix.lower() in {".jar", ".class", ".mca", ".log"}:
            raise PackError(f"prohibited package entry: {name}")
    return {name: safe_read(root, name) for name in names}


def package(output, root=ROOT):
    receipt = verify(root)
    files = package_files(root)
    if LOCK not in files:
        raise PackError("source lock must be tracked before packaging")
    # Freeze reviewed files; a dirty tracked tool/doc must not silently enter a release.
    dirty = run(["git", "diff", "HEAD", "--name-only", "--", *files], root)
    if dirty:
        raise PackError("commit reviewed package sources first: " + dirty.replace("\n", ", "))
    files["BUILD.json"] = encode({"kind": "native-source-recipe", "horizon": receipt["horizon"],
                                  "rustc": receipt["rustc"], "cargo": receipt["cargo"],
                                  "host": receipt["host"], "assets": "Prepare separately from your own licensed installation",
                                  "warning": "Trusted native process code; rebuild required; no sandbox or binary plugin ABI"})
    files["SHA256SUMS"] = ''.join(f"{digest(value)}  {name}\n" for name, value in sorted(files.items())).encode()
    # Refuse overwrites, including accidental output over tracked sources.
    with zipfile.ZipFile(output, "x", compression=zipfile.ZIP_STORED) as archive:
        for name, value in sorted(files.items()):
            entry = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = 0o100644 << 16
            archive.writestr(entry, value)
    return digest(output.read_bytes())


def build(variant, features, release, root=ROOT):
    receipt = verify(root)
    config = recipe(root / "native-modpack.toml", root)
    known = {m["feature"] for m in config["mods"] if "feature" in m}
    chosen = set(filter(None, features.split(",")))
    if chosen - known or (variant == "vanilla" and chosen):
        raise PackError("unknown feature or pack features selected for vanilla build")
    from check_native_boundaries import audit, metadata, feature_off
    audit(metadata(root), root)
    feature_off(root)
    command = ["cargo", "build", "--manifest-path", str(root / "horizon/Cargo.toml"),
               "-p", "horizon-client", "--bin", "horizon-client", "--bin", "horizon-ctl",
               "--no-default-features", "--locked", "--offline", "--target-dir", str(root / "horizon/target")]
    if variant == "native":
        enabled = ["native-mods", *("horizon-native-pack/" + f for f in sorted(chosen))]
        command += ["--features", ",".join(enabled)]
    if release:
        command.append("--release")
    subprocess.run(command, cwd=root, check=True)
    directory = root / "horizon/target" / ("release" if release else "debug")
    suffix = ".exe" if "windows" in receipt["host"] else ""
    result = {"source_lock": digest((root / LOCK).read_bytes()), "variant": variant,
              "features": sorted(chosen), "host": receipt["host"], "command": command,
              "binaries": {name: digest((directory / (name + suffix)).read_bytes())
                           for name in ["horizon-client", "horizon-ctl"]}}
    (directory / f"native-build-{variant}.json").write_bytes(encode(result))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("lock", help="record reviewed source/toolchain/engine pins; never run this blindly")
    commands.add_parser("verify", help="refuse source/toolchain/lock drift")
    pack = commands.add_parser("package", help="export deterministic source recipe, without engine or Mojang files")
    pack.add_argument("output", type=Path)
    make = commands.add_parser("build", help="verified, headless-feature, locked offline host build")
    make.add_argument("--variant", choices=["vanilla", "native"], required=True)
    make.add_argument("--features", default="", help="comma-separated optional pack features")
    make.add_argument("--release", action="store_true")
    args = parser.parse_args()
    try:
        if args.command == "lock":
            (ROOT / LOCK).write_bytes(encode(snapshot()))
        elif args.command == "verify":
            verify()
        elif args.command == "build":
            build(args.variant, args.features, args.release)
        else:
            print("sha256:", package(args.output))
    except (PackError, OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"native-pack-release: {error}\n")
    print(f"native-pack-release: {args.command} PASS")


if __name__ == "__main__":
    main()
