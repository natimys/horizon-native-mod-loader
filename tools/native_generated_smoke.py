#!/usr/bin/env python3
"""Compile real zero/one/two/gameplay generated packs in an asset-free scratch copy."""
from pathlib import Path
import shutil
import subprocess
import tempfile

from native_modpack import ROOT, generated, recipe


def main():
    config = recipe(ROOT / "native-modpack.toml")
    with tempfile.TemporaryDirectory(prefix="native-generated-") as tmp:
        root = Path(tmp)
        shutil.copytree(ROOT / "native-mods", root / "native-mods", ignore=shutil.ignore_patterns("target"))
        by_package = {m["package"]: m for m in config["mods"]}
        for names in [[], ["hello"], ["hello", "companion"], ["hello", "workshop"]]:
            mods = [dict(by_package[f"horizon-native-{name}-mod"]) for name in names]
            for mod in mods:
                mod.pop("feature", None)
            for name, value in generated({**config, "mods": mods}, root).items():
                (root / name).write_bytes(value)
            ids = sorted({"hello": "hello_mod", "companion": "companion", "workshop": "workshop"}[n] for n in names)
            expected = ', '.join('"' + name + '"' for name in ids)
            test = '''use horizon_native_pack::create_for_side;
use horizon_native_runtime::{HostSide, State};
#[test]
fn generated_contract() {
    for side in [HostSide::Client, HostSide::Server, HostSide::Combined] {
        let mut runtime = create_for_side(side).unwrap();
        let expected: Vec<&str> = vec![EXPECTED];
        assert_eq!(runtime.metadata().map(|m| m.id).collect::<Vec<_>>(), expected);
        runtime.freeze().unwrap();
        let content = runtime.content().clone();
        runtime.initialize().unwrap();
        assert_eq!(runtime.content(), &content);
        if side == HostSide::Client { assert!(!runtime.has_gameplay_subscribers()); }
        assert!(runtime.shutdown().errors.is_empty());
        assert_eq!(runtime.state(), State::Stopped);
        assert!(runtime.shutdown().actions.is_empty());
    }
}
'''.replace('EXPECTED', expected)
            (root / "native-mods/pack/tests/generated_contract.rs").write_text(test)
            # Only local std-only packages: regenerate the scratch lock after an
            # intentional composition change, then prove the actual run is locked.
            subprocess.run(["cargo", "generate-lockfile", "--manifest-path", str(root / "native-mods/Cargo.toml"), "--offline"], check=True)
            subprocess.run(["cargo", "test", "--manifest-path", str(root / "native-mods/Cargo.toml"),
                            "-p", "horizon-native-pack", "--test", "generated_contract", "--locked", "--offline"], check=True)
            print("generated-pack PASS:", names, flush=True)


if __name__ == "__main__":
    main()
