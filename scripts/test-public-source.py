import importlib.util
from pathlib import Path
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location('public_source', Path(__file__).with_name('public_source.py'))
publication = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publication)


class PublicationTests(unittest.TestCase):
    def test_binary_guard_rejects_wide_and_ascii_home_paths(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'sample.exe'
            for encoding in ['utf-8', 'utf-16-le']:
                path.write_bytes(('D:' + '\\Users\\' + 'private-person\\file.rs').encode(encoding))
                with self.assertRaises(ValueError) as raised:
                    publication.audit_binary(path)
                self.assertNotIn('private-person', str(raised.exception))
            path.write_bytes(b'/cargo/registry/src/lib.rs')
            publication.audit_binary(path)

    def make_tree(self, root):
        for name in publication.ROOT_FILES:
            (root / name).write_text('public fixture\n', encoding='utf-8')
        for name in publication.FOLDERS:
            (root / name).mkdir()

    def test_private_files_and_paths_are_rejected_without_echoing_values(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_tree(root)
            (root / 'examples/.env.production').write_text('private value')
            (root / 'docs/example.md').write_text('C:' + '/Users/' + 'private-person/project')
            _, findings = publication.audit(root)
            self.assertEqual({f['rule'] for f in findings}, {'private-or-binary-file', 'developer-home-path'})
            self.assertNotIn('private-person', str(findings))
            with self.assertRaises(ValueError):
                publication.package(root, root / 'source.zip')

    def test_package_is_deterministic_and_excludes_local_state(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_tree(root)
            for name in ['.git', '.venv', 'target', 'outputs']:
                (root / name).mkdir()
                (root / name / 'private.txt').write_text('not distributable')
            (root / 'catalog/local.duckdb').write_bytes(b'local data')
            first, second = root / 'one.zip', root / 'two.zip'
            publication.package(root, first)
            publication.package(root, second)
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with zipfile.ZipFile(first) as archive:
                self.assertFalse(any('private' in p or 'duckdb' in p for p in archive.namelist()))
            with self.assertRaises(FileExistsError):
                publication.package(root, first)


if __name__ == '__main__':
    unittest.main()
