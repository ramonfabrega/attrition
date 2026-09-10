#!/usr/bin/env python3
"""The gate refuses absent installs and never launches a child without RON_INSTALL."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('release_gate', Path(__file__).resolve().parents[1]/'release_gate.py')
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def write_request(directory, present=True, name='fixture', test='test'):
    with (Path(directory)/'123.tsv').open('a') as stream:
        stream.write(f"{int(present)}\t{name.encode().hex()}\t{test.encode().hex()}\n")


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
                if '--release' in command:
                    write_request(kwargs['env']['RON_FIXTURE_AUDIT_DIR'])
            gate.gate(path, report_dir=path/'report', run=run)
            self.assertEqual(len(calls), 6)
            self.assertEqual(calls[0], [sys.executable, 'tools/offline_tests.py'])
            self.assertEqual(calls[1][:4], ['cargo','run','-p','rondata'])
            self.assertIn('--release', calls[2])

    def test_offline_failure_stops_before_install_survey_and_release(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def run(command, **kwargs):
                calls.append(command)
                if command == [sys.executable, 'tools/offline_tests.py']:
                    raise subprocess.CalledProcessError(1, command)
                if '--release' in command:
                    write_request(kwargs['env']['RON_FIXTURE_AUDIT_DIR'])
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run)
            self.assertEqual(calls, [[sys.executable, 'tools/offline_tests.py']])

    def test_failed_survey_stops_before_tests(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def fail(command, **kwargs):
                calls.append(command)
                if command[0] == 'cargo':
                    raise subprocess.CalledProcessError(1, command)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=fail)
            self.assertEqual(len(calls), 2)

    def test_summary_reports_requests_not_skipped_tests(self):
        with tempfile.TemporaryDirectory() as tmp:
            write_request(tmp, False, 'absent', 'test_a')
            write_request(tmp, False, 'absent', 'test_b')
            write_request(tmp, True, 'present', 'test_a')
            summary=gate.summarize_requests(tmp, release_completed=True)
            self.assertEqual(summary['observed_requests'], 3)
            self.assertEqual(summary['missing_fixtures'], 1)
            self.assertEqual(summary['fixtures'][0]['tests'], ['test_a','test_b'])
            self.assertFalse(summary['complete_corpus_claim'])

    def test_malformed_audit_is_not_a_clean_report(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp)/'123.tsv').write_text('0\t00')
            with self.assertRaises(ValueError):
                gate.summarize_requests(tmp, release_completed=True)

    def test_absence_policy_and_unobserved_coverage(self):
        for emit, strict, succeeds in [(True,False,True),(True,True,False),(False,False,False)]:
            with self.subTest(emit=emit, strict=strict), tempfile.TemporaryDirectory() as tmp:
                path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
                def run(command, **kwargs):
                    if '--release' in command and emit:
                        write_request(kwargs['env']['RON_FIXTURE_AUDIT_DIR'], False)
                if succeeds:
                    gate.gate(path, report_dir=path/'report', run=run, require_fixtures=strict)
                else:
                    with self.assertRaises(ValueError):
                        gate.gate(path, report_dir=path/'report', run=run, require_fixtures=strict)
                report=json.loads((path/'report/fixture-coverage.json').read_text())
                self.assertEqual(report['observed_requests'], int(emit))

    def test_failed_release_retains_incomplete_audit(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            def run(command, **kwargs):
                if '--release' in command:
                    write_request(kwargs['env']['RON_FIXTURE_AUDIT_DIR'])
                    raise subprocess.CalledProcessError(1, command)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run)
            report=json.loads((path/'report/fixture-coverage.json').read_text())
            self.assertFalse(report['release_completed'])


if __name__ == '__main__':
    unittest.main()
