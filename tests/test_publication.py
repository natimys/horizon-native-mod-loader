import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("audit", ROOT / "tools/audit_publication.py")
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class PublicationAuditTests(unittest.TestCase):
    def test_deleted_credentials_and_renamed_private_files_still_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            def git(*args):
                return subprocess.run(["git", "-c", "user.name=Test", "-c",
                                       "user.email=test@example.invalid", *args], cwd=root,
                                      check=True, stdout=subprocess.DEVNULL)
            git("init", "-q")
            (root / "secret.txt").write_text("ghp_" + "a" * 36)
            (root / "game-resources").mkdir()
            (root / "game-resources/private.txt").write_text("synthetic fixture")
            git("add", "secret.txt", "game-resources/private.txt")
            git("commit", "-qm", "synthetic sensitive history")
            git("rm", "secret.txt")
            git("mv", "game-resources/private.txt", "public.txt")
            git("commit", "-qm", "deletion does not remove history")
            report = audit.scan(root)
            self.assertEqual(report["commits"], 2)
            self.assertTrue(any(f["kind"] == "github-token" for f in report["findings"]))
            self.assertTrue(any(f.get("path") == "game-resources/private.txt" for f in report["findings"]))
            self.assertNotIn("ghp_" + "a" * 36, str(report))

    def test_public_sources_and_original_image_are_allowed(self):
        self.assertIsNone(audit.PATTERNS["personal-path"].search(b"temporary working/home/XDG/launcher paths"))
        self.assertIsNotNone(audit.PATTERNS["personal-path"].search(b'"/' + b'home/person/project/"'))
        self.assertFalse(audit.prohibited("native-mods/mods/hello/assets/token.png"))
        self.assertFalse(audit.prohibited("docs/native-modding/TOOLCHAIN.md"))
        for name in [".env", "nested/.env.local", ".codex/config.toml", "build/example.jar", "saves/level.dat"]:
            self.assertTrue(audit.prohibited(name), name)

    def test_service_tokens_and_literal_secret_assignments(self):
        for prefix in [b"sk-proj-", b"xoxb-"]:
            self.assertIsNotNone(audit.PATTERNS["service-token"].search(prefix + b"a" * 40))
        self.assertIsNotNone(audit.PATTERNS["service-token"].search(b"AIza" + b"a" * 35))
        self.assertIsNotNone(audit.PATTERNS["secret-assignment"].search(b'api_key = "' + b"a" * 32 + b'"'))
        self.assertIsNone(audit.PATTERNS["secret-assignment"].search(b'api_key = read_from_environment()'))


if __name__ == "__main__":
    unittest.main()
