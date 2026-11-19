"""`tools/offline_tests.py` runs every `tools/explore/test_*.py`, or names
the one it does not with a reason (parked 1872, the thirtieth pass: the
sweep generator's, the run-wiring printer's and the births probe's suites
were landed by two passes and run by no gate).
"""
import importlib.util
import os
import subprocess
import sys
import tempfile
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
        self.assertLess(offline.PROCESS_SUITES, run)
        self.assertEqual(len(run), len(offline.SUITES))

    def test_a_failure_in_either_worker_fails_the_command_and_keeps_both_logs(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            driver = directory / 'driver.py'
            driver.write_text(
                f'import sys\nsys.path.insert(0, {str(ROOT / "tools")!r})\n'
                'import offline_tests as runner\n'
                'runner.SUITES = ("foreground", "background")\n'
                'runner.PROCESS_SUITES = {"background"}\n'
                'runner.__file__ = __file__\nraise SystemExit(runner.main())\n')
            for name in ('foreground', 'background'):
                (directory / f'{name}.py').write_text(
                    'import os, unittest\nclass Verdict(unittest.TestCase):\n'
                    '    def test_passes(self):\n'
                    '        self.assertNotEqual(os.environ["FAIL_WORKER"], __name__)\n')
            for failing in ('neither', 'background', 'foreground'):
                with self.subTest(failing=failing):
                    result = subprocess.run([sys.executable, str(driver), '--jobs', '2'],
                                            env=dict(os.environ, FAIL_WORKER=failing),
                                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                    self.assertEqual(result.returncode, int(failing != 'neither'), result.stdout)
                    for name in ('foreground', 'background'):
                        verdict = 'FAIL' if name == failing else 'ok'
                        self.assertIn(f'({name}.Verdict.test_passes) ... {verdict}', result.stdout)
                    if failing == 'neither':
                        self.assertIn('Offline tests: 2 ran', result.stdout)


if __name__ == '__main__':
    unittest.main()
