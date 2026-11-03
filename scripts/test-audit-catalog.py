import importlib.util
import json
from pathlib import Path
import unittest
import sys
sys.dont_write_bytecode = True

spec = importlib.util.spec_from_file_location('audit', Path(__file__).with_name('audit-catalog.py'))
audit = importlib.util.module_from_spec(spec); spec.loader.exec_module(audit)

class RegistryAuditTests(unittest.TestCase):
    def test_coordinates_are_encoded_and_unknown_registries_explicit(self):
        self.assertIn('%40scope%2Fsdk', audit.coordinate_url('npm', '@scope/sdk'))
        self.assertIsNone(audit.coordinate_url('cargo', 'rig'))
        with self.assertRaises(ValueError): audit.coordinate_url('composer', '../../etc')

    def test_identity_is_checked_without_promoting_semantics(self):
        payload = json.dumps({'info': {'name': 'Some_AI', 'version': '1.0', 'summary': 'AI SDK'}}).encode()
        self.assertEqual(audit.interpret('pypi', 'some-ai', payload)['observed_version'], '1.0')
        with self.assertRaises(ValueError): audit.interpret('pypi', 'other', payload)
        self.assertEqual(audit.audit(('framework', 'cargo', 'rig'))['status'], 'registry_adapter_unavailable')

if __name__ == '__main__': unittest.main()
