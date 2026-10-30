import importlib.util
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("native_modpack", ROOT / "tools/native_modpack.py")
pack = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pack)


class GeneratorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        shutil.copytree(ROOT / "native-mods", self.root / "native-mods",
                        ignore=shutil.ignore_patterns("target"))
        self.path = self.root / "native-modpack.toml"
        self.path.write_bytes((ROOT / "native-modpack.toml").read_bytes())

    def test_current_recipe_and_repeat_generation(self):
        first = pack.generate(self.path, root=self.root)
        self.assertEqual(first, pack.generate(self.path, root=self.root))
        pack.generate(self.path, check=True, root=self.root)
        (self.root / "native-mods/pack/src/lib.rs").write_text("stale")
        with self.assertRaisesRegex(pack.PackError, "stale"):
            pack.generate(self.path, check=True, root=self.root)

    def test_order_independent_optional_features_and_exact_pins(self):
        data = pack.recipe(self.path, self.root)
        data["mods"].reverse()
        self.assertEqual(pack.generated(pack.recipe(self.path, self.root), self.root),
                         pack.generated(data, self.root))
        files = pack.generated(pack.recipe(self.path, self.root), self.root)
        self.assertIn(b'version = "=0.1.0"', files["native-mods/pack/Cargo.toml"])
        self.assertIn(b'#[cfg(feature = "gameplay-example")]', files["native-mods/pack/src/lib.rs"])

    def test_zero_mod_generation(self):
        data = pack.recipe(self.path, self.root)
        data["mods"] = []
        self.assertIn(b'Runtime::for_side(vec![], side)', pack.generated(data, self.root)["native-mods/pack/src/lib.rs"])

    def test_bad_schema_api_unknown_key_and_rust(self):
        original = self.path.read_text()
        for before, after in [('schema = 1', 'schema = 2'), ('api = 1', 'api = 99'),
                              ('api = 1', 'api = true'), ('schema = 1', 'schmea = 1'),
                              ('rust = "1.99.1"', 'rust = "stable"')]:
            with self.subTest(after=after):
                self.path.write_text(original.replace(before, after))
                with self.assertRaises(pack.PackError):
                    pack.recipe(self.path, self.root)

    def test_bad_mods(self):
        original = self.path.read_text()
        for before, after in [('version = "0.1.0"', 'version = "*"'),
                              ('native-mods/mods/hello', 'native-mods/mods/../api'),
                              ('feature = "test-pack"', 'feature = "default"'),
                              ('horizon-native-hello-mod', 'horizon-native-runtime'),
                              ('horizon-native-hello-mod', 'horizon-client')]:
            with self.subTest(after=after):
                self.path.write_text(original.replace(before, after))
                with self.assertRaises(pack.PackError):
                    pack.recipe(self.path, self.root)
        self.path.write_text(original + original[original.index('[[mods]]'):original.index('[[mods]]', original.index('[[mods]]') + 1)])
        with self.assertRaisesRegex(pack.PackError, "duplicate"):
            pack.recipe(self.path, self.root)

    def test_normalized_crate_collision_and_manifest_symlink(self):
        source = self.root / "native-mods/mods/hello/Cargo.toml"
        saved = source.read_text()
        other = self.root / "native-mods/mods/alias"
        other.mkdir()
        (other / "Cargo.toml").write_text(saved.replace("horizon-native-hello-mod", "horizon_native_hello_mod"))
        self.path.write_text(self.path.read_text() + '\n[[mods]]\npackage = "horizon_native_hello_mod"\npath = "native-mods/mods/alias"\nversion = "0.1.0"\n')
        with self.assertRaisesRegex(pack.PackError, "collide"):
            pack.recipe(self.path, self.root)
        self.path.write_bytes((ROOT / "native-modpack.toml").read_bytes())
        source.unlink()
        target = self.root / "manifest.toml"
        target.write_text(saved)
        source.symlink_to(target)
        with self.assertRaisesRegex(pack.PackError, "symlink Cargo.toml"):
            pack.recipe(self.path, self.root)

    def test_symlink_refused(self):
        link = self.root / "native-mods/mods/link"
        link.symlink_to(self.root / "native-mods/mods/hello", target_is_directory=True)
        self.path.write_text(self.path.read_text().replace('native-mods/mods/hello', 'native-mods/mods/link'))
        with self.assertRaisesRegex(pack.PackError, "symlink"):
            pack.recipe(self.path, self.root)


if __name__ == '__main__':
    unittest.main()
