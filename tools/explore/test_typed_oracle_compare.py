import json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
from typed_oracle_compare import run

class RunnerTests(unittest.TestCase):
    def test_git_destinations_and_existing_evidence_are_not_overwritten(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);(root/'.git').mkdir()
            with self.assertRaisesRegex(ValueError,'outside Git'):run(root,root,root,root,root/'out')
            self.assertFalse((root/'out').exists())
            (root/'.git').rmdir();(root/'out').mkdir();(root/'out'/'keep').write_text('evidence')
            with self.assertRaises(FileExistsError):run(root,root,root,root,root/'out')
            self.assertEqual((root/'out'/'keep').read_text(),'evidence')
    def test_failed_decode_leaves_failed_manifest_not_success(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);plan=root/'plan.json';plan.write_text('{}')
            with patch('typed_oracle_compare.experiment',side_effect=ValueError('authored failure')):
                with self.assertRaises(ValueError):run(root,root,root,plan,root/'out')
            manifest=json.loads((root/'out'/'manifest.json').read_text())
            self.assertEqual(manifest['status'],'failed');self.assertFalse(manifest['whole_frame_parity'])
            self.assertFalse((root/'out'/'comparison.json.gz').exists())

if __name__=='__main__':unittest.main()
