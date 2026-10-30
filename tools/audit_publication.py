#!/usr/bin/env python3
"""Offline Git object audit. Reports locations, never matched credential values.

Checks reachable objects, including deleted files, without reading ignored player
resources. Heuristics supplement source/rights review; they cannot prove absence
of secrets. Use the engine's own check_public_tree.py for its path policy.
"""
import argparse
import json
from pathlib import Path, PurePosixPath
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PRIVATE = {"game-resources", ".aws", ".agents", ".codex", ".local", "target",
           "saves", "backups", "__pycache__", ".venv", "node_modules"}
EXTENSIONS = {".jar", ".class", ".mca", ".mcr", ".hprof", ".jfr", ".exe", ".dll", ".so", ".zip"}
PATTERNS = {
    "github-token": re.compile(rb"\b(?:gh[pousr]_[A-Za-z0-9]{36,255}|github_pat_[A-Za-z0-9_]{60,255})\b"),
    "aws-key-id": re.compile(rb"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b"),
    "private-key": re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"),
    "credential-url": re.compile(rb"https?://[^\s/:@]+:[^\s/@]+@"),
    "personal-path": re.compile(rb"/(?:home|Users)/[A-Za-z0-9_.-]+/"),
}


def git(root, *args):
    return subprocess.check_output(["git", "--no-replace-objects", *args], cwd=root)


def prohibited(name):
    path = PurePosixPath(name)
    return (bool(PRIVATE.intersection(path.parts)) or path.suffix.lower() in EXTENSIONS
            or any(p == ".env" or p.startswith(".env.") for p in path.parts)
            or path.name in {"README-SETUP.md", "horizon_official_server.txt", "credentials", "id_rsa", "id_ed25519"})


def scan(root, all_refs=False, engine=False):
    roots = ["--all"] if all_refs else ["HEAD"]
    # Check every historical path separately: one blob can have several names.
    paths = set(filter(None, git(root, "log", "--format=", "--name-only", "-z", "-m", *roots, "--").decode().split("\0")))
    paths = {p.lstrip("\n") for p in paths}
    findings = []
    if not engine:
        findings += [{"kind": "private-path", "path": p} for p in sorted(paths) if prohibited(p)]
    objects = git(root, "rev-list", "--objects", *roots).splitlines()
    names = dict((line.split(b" ", 1) + [b""])[:2] for line in objects)
    # cat-file batch avoids a process for each historical blob/commit.
    meta = subprocess.run(["git", "cat-file", "--batch-check"], cwd=root,
                          input=b"\n".join(names) + b"\n", stdout=subprocess.PIPE, check=True).stdout
    selected, binaries, counts = [], [], {"blobs": 0, "commits": 0}
    for line in meta.splitlines():
        oid, kind, size = line.split()
        if kind not in {b"blob", b"commit"}:
            continue
        counts["blobs" if kind == b"blob" else "commits"] += 1
        name = names[oid].decode(errors="replace")
        if int(size) > 5 * 1024 * 1024:
            findings.append({"kind": "large-object", "path": name, "oid": oid.decode(), "bytes": int(size)})
        selected.append(oid)
    # Input/output are files, avoiding pipe deadlocks on large engine histories.
    import tempfile
    with tempfile.TemporaryFile() as request, tempfile.TemporaryFile() as response:
        request.write(b"\n".join(selected) + b"\n")
        request.seek(0)
        subprocess.run(["git", "cat-file", "--batch"], cwd=root, stdin=request, stdout=response, check=True)
        response.seek(0)
        for oid in selected:
            header = response.readline().split()
            content = response.read(int(header[2]))
            assert response.read(1) == b"\n"
            name = names[oid].decode(errors="replace")
            if b"\0" in content:
                binaries.append({"path": name, "oid": oid.decode(), "bytes": len(content)})
                continue
            for label, pattern in PATTERNS.items():
                if pattern.search(content):
                    findings.append({"kind": label, "path": name or "<commit message>", "oid": oid.decode()})
    return {"scope": "all local refs" if all_refs else "HEAD ancestry", **counts,
            "historical_paths": len(paths), "findings": findings, "binary_blobs": binaries}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=ROOT)
    parser.add_argument("--all-refs", action="store_true")
    parser.add_argument("--engine", action="store_true", help="defer engine path policy to upstream's guard")
    args = parser.parse_args()
    report = scan(args.repository, args.all_refs, args.engine)
    print(json.dumps(report, indent=2))
    return bool(report["findings"])


if __name__ == "__main__":
    raise SystemExit(main())
