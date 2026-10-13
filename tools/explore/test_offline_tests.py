"""`tools/offline_tests.py` runs every `tools/explore/test_*.py`, or names
the one it does not with a reason (parked 1872, the thirtieth pass: the
sweep generator's, the run-wiring printer's and the births probe's suites
were landed by two passes and run by no gate).
"""
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('offline_tests', ROOT / 'tools/offline_tests.py')
offline = importlib.util.module_from_spec(spec)
spec.loader.exec_module(offline)


class EverySuiteIsRunOrNamed(unittest.TestCase):
    def test_every_explore_suite_is_run_or_named(self):
        here = {p.stem for p in (ROOT / 'tools/explore').glob('test_*.py')}
        run = set(offline.SUITES)
        named = set(offline.NOT_RUN)
        self.assertEqual(sorted(here - run - named), [], 'suites no gate runs and nothing names')
        self.assertEqual(sorted(run & named), [], 'a suite both run and excused')
        self.assertEqual(sorted(named - here), [], 'an excuse for a suite that is gone')
        self.assertIn('test_offline_tests', run)


if __name__ == '__main__':
    unittest.main()
