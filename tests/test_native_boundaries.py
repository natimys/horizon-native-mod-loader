from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from check_native_boundaries import audit
from native_modpack import PackError


def graph():
    packages = [
        {"id": "mod", "name": "example-mod", "manifest_path": str(ROOT / "native-mods/mods/example/Cargo.toml"), "dependencies": []},
        {"id": "api", "name": "horizon-native-api", "manifest_path": str(ROOT / "native-mods/api/Cargo.toml"), "dependencies": []},
        {"id": "lib", "name": "ordinary-library", "manifest_path": "/registry/ordinary/Cargo.toml", "dependencies": []},
        {"id": "engine", "name": "horizon-world", "manifest_path": str(ROOT / "horizon/crates/horizon-world/Cargo.toml"), "dependencies": []}]
    nodes = [{"id": p["id"], "deps": []} for p in packages]
    nodes[0]["deps"] = [{"pkg": "api", "name": "renamed_api", "dep_kinds": [{"kind": None, "target": None}]}]
    return {"packages": packages, "resolve": {"nodes": nodes}}


class BoundaryTests(unittest.TestCase):
    def test_public_api_only(self):
        self.assertEqual(audit(graph()), 1)

    def test_renamed_transitive_target_and_build_edges(self):
        for kind in [None, "build", "dev"]:
            data = graph()
            data["resolve"]["nodes"][0]["deps"].append({"pkg": "lib", "name": "lib", "dep_kinds": [{"kind": kind, "target": "cfg(windows)"}]})
            data["resolve"]["nodes"][2]["deps"] = [{"pkg": "engine", "name": "innocent_alias", "dep_kinds": [{"kind": kind, "target": "cfg(windows)"}]}]
            with self.assertRaisesRegex(PackError, "example-mod -> ordinary-library -> horizon-world"):
                audit(data)

    def test_engine_renamed_package_and_private_native_layers(self):
        for name in ["innocent-renamed-engine", "horizon-native-runtime", "horizon-native-ui", "horizon-native-pack"]:
            data = graph()
            data["packages"][3]["name"] = name
            data["resolve"]["nodes"][0]["deps"].append({"pkg": "engine"})
            with self.assertRaises(PackError):
                audit(data)

    def test_api_has_no_engine_or_library_dependencies(self):
        data = graph()
        data["resolve"]["nodes"][1]["deps"] = [{"pkg": "lib"}]
        with self.assertRaisesRegex(PackError, "horizon-native-api -> ordinary-library"):
            audit(data)

    def test_registry_pins_and_external_paths(self):
        for dependency in [{"name": "lib", "source": "registry+x", "req": "^1.0.0"},
                           {"name": "lib", "source": "git+x", "req": "*"},
                           {"name": "lib", "path": "/outside"}]:
            data = graph()
            data["packages"][0]["dependencies"] = [dependency]
            with self.assertRaises(PackError):
                audit(data)
        data = graph()
        data["packages"][0]["dependencies"] = [{"name": "lib", "source": "registry+x", "req": "=1.0.0"}]
        self.assertEqual(audit(data), 1)


if __name__ == "__main__":
    unittest.main()
