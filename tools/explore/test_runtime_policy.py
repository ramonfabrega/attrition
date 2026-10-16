"""Allocator policy and actual runner plumbing, using authored/mocked children."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch, MagicMock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import test_runtime as runtime
import isolated_tests as runner
import release_gate as gate


class RuntimePolicy(unittest.TestCase):
    def test_darwin_default_is_child_local_and_preserves_overrides(self):
        original = {'RON_INSTALL': '/authored'}
        env = runtime.child_environment(original, platform='darwin')
        self.assertEqual(env['MallocLargeCache'], '0')
        self.assertEqual(original, {'RON_INSTALL': '/authored'})
        for value in ('0', '1'):
            self.assertEqual(runtime.child_environment({'MallocLargeCache': value}, platform='darwin')['MallocLargeCache'], value)

    def test_portability_optout_and_invalid_mode(self):
        for platform in ('linux', 'win32'):
            self.assertNotIn('MallocLargeCache', runtime.child_environment({}, platform=platform))
        self.assertNotIn('MallocLargeCache', runtime.child_environment({'RON_TEST_ALLOCATOR': 'system'}, platform='darwin'))
        self.assertEqual(runtime.child_environment({'RON_TEST_ALLOCATOR': 'system', 'MallocLargeCache': '1'}, platform='darwin')['MallocLargeCache'], '1')
        with self.assertRaises(ValueError):
            runtime.child_environment({'RON_TEST_ALLOCATOR': 'typo'}, platform='darwin')

    def test_isolated_child_receives_and_reports_effective_policy(self):
        with tempfile.TemporaryDirectory() as d, patch.dict(os.environ, {}, clear=True), patch.object(runtime.sys, 'platform', 'darwin'):
            root = Path(d)
            (root / 'tests').mkdir()
            child = MagicMock()
            child.poll.return_value = 0
            child.returncode = 0
            child.__enter__.return_value = child
            with patch.object(runner.subprocess, 'Popen', return_value=child) as spawn, patch.object(runner, 'binary_stamp', return_value=(1,)), patch.object(runner, 'read_result', return_value=(1, (1.0, 0.0), 123)):
                result = runner.run_one(root / 'binary', 'authored', 0, root, root / 'resources', 128, lambda: False)
            self.assertTrue(result['success'])
            self.assertEqual(spawn.call_args.kwargs['env']['MallocLargeCache'], '0')
            self.assertEqual(result['allocator']['MallocLargeCache'], '0')
            self.assertNotIn('MallocLargeCache', os.environ)

    def test_gate_children_and_manifest_share_policy(self):
        import json
        with tempfile.TemporaryDirectory() as d, patch.dict(os.environ, {}, clear=True), patch.object(runtime.sys, 'platform', 'darwin'):
            root = Path(d)
            (root / 'Data').mkdir()
            (root / 'Data/rules.xml').write_text('authored')
            calls = []
            def run(command, **kwargs):
                calls.append(command)
                self.assertEqual(kwargs['env']['MallocLargeCache'], '0')
                if '--release' in command:
                    (Path(kwargs['env']['RON_FIXTURE_AUDIT_DIR']) / '123.tsv').write_text('1\t66697874757265\t74657374\n')
                    Path(kwargs['log']).write_text('     Running unittests src/lib.rs (target/release/deps/rondata-authored)\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n')
            report = gate.gate(root, report_dir=root / 'report', run=run)
            self.assertEqual(len(calls), 6)
            self.assertEqual(json.loads((report / 'gate-policy.json').read_text())['test_allocator']['MallocLargeCache'], '0')
            self.assertNotIn('MallocLargeCache', os.environ)
