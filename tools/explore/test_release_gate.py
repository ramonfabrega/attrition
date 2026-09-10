#!/usr/bin/env python3
"""The gate refuses absent installs and never launches a child without RON_INSTALL."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('release_gate', Path(__file__).resolve().parents[1]/'release_gate.py')
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class GateTests(unittest.TestCase):
    def test_missing_install_never_runs(self):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaises(ValueError):
                gate.gate(tmp, run=lambda *a, **k: self.fail('ran without install'))

    def test_every_child_has_explicit_install(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').write_text('authored placeholder')
            calls=[]
            def run(command, **kwargs):
                self.assertEqual(kwargs['env']['RON_INSTALL'], str(path.resolve()))
                self.assertTrue(kwargs['check'])
                calls.append(command)
            gate.gate(path, run=run)
            self.assertEqual(len(calls), 5)
            self.assertEqual(calls[0][:4], ['cargo','run','-p','rondata'])
            self.assertIn('--release', calls[1])

    def test_failed_survey_stops_before_tests(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def fail(command, **kwargs):
                calls.append(command)
                raise subprocess.CalledProcessError(1, command)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, run=fail)
            self.assertEqual(len(calls), 1)


if __name__ == '__main__':
    unittest.main()
