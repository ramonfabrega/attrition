"""Build/allocator policy and runner plumbing, using authored/mocked children."""
import os
from pathlib import Path
import sys
import tempfile
import tomllib
import unittest
from unittest.mock import patch, MagicMock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import test_runtime as runtime
import isolated_tests as runner
import release_gate as gate



class GoldenCaseRegistration(unittest.TestCase):
    def check_family(self, folder):
        import re
        parent = (folder / 'golden.rs').read_text()
        modules = re.findall(r'^mod (cases_\w+);$', parent, re.M)
        files = {p.stem for p in (folder / 'golden').glob('cases_*.rs')}
        self.assertEqual(len(modules), len(set(modules)))
        self.assertEqual(set(modules), files)
        wrappers = re.findall(
            r'#\[test\]\nfn (\w+)\(\) \{\n    (cases_\w+)::(\w+)\(\);\n\}', parent)
        self.assertEqual(len(wrappers), parent.count('#[test]'))
        actual = []
        for module in modules:
            source = (folder / 'golden' / f'{module}.rs').read_text()
            self.assertNotIn('#[test]', source)
            actual += [(name, module, name) for name in re.findall(
                r'^pub\(super\) fn (\w+)\(\)', source, re.M)]
        self.assertEqual(sorted(wrappers), sorted(actual))
        self.assertEqual(len(wrappers), len({name for name, _, _ in wrappers}))

    def test_golden_cases_have_one_original_name_and_one_body(self):
        self.check_family(Path(__file__).resolve().parents[2] / 'crates/rondata/src/diff')

    def test_missing_misdirected_duplicate_or_unmounted_cases_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            (folder / 'golden').mkdir()
            parent = folder / 'golden.rs'
            body = folder / 'golden/cases_setup.rs'
            registration = ('mod cases_setup;\n#[test]\nfn original() {\n'
                            '    cases_setup::original();\n}\n')
            definition = 'pub(super) fn original() {}\n'
            parent.write_text(registration)
            body.write_text(definition)
            self.check_family(folder)
            for bad in [registration.replace('::original()', '::other()'),
                        registration + registration,
                        registration.replace('mod cases_setup;\n', '')]:
                parent.write_text(bad)
                with self.assertRaises(AssertionError):
                    self.check_family(folder)
            parent.write_text(registration)
            body.write_text('')
            with self.assertRaises(AssertionError):
                self.check_family(folder)


class HarnessCaseRegistration(unittest.TestCase):
    def check_family(self, folder):
        import re
        parent = (folder / 'harness.rs').read_text().split('pub(crate) mod tests {', 1)[1]
        modules = re.findall(r'^    mod (cases_\w+);$', parent, re.M)
        files = {p.stem for p in (folder / 'harness/tests').glob('cases_*.rs')}
        self.assertEqual(len(modules), len(set(modules)))
        self.assertEqual(set(modules), files)
        wrappers = re.findall(
            r'    #\[test\]\n(?:    #\[ignore\]\n)?    fn (\w+)\(\) \{\n        (cases_\w+)::(\w+)\(\);\n    \}', parent)
        self.assertEqual(len(wrappers), parent.count('#[test]'))
        actual = []
        for module in modules:
            source = (folder / 'harness/tests' / f'{module}.rs').read_text()
            self.assertTrue(source.startswith('pub(super) use bodies::*;\n\nmod bodies {\n    use super::super::*;\n'))
            self.assertNotIn('#[test]', source)
            actual += [(name, module, name) for name in re.findall(
                r'^    pub\(in super::super\) fn (\w+)\(\)', source, re.M)]
        self.assertEqual(sorted(wrappers), sorted(actual))
        self.assertEqual(len(wrappers), len({name for name, _, _ in wrappers}))

    def test_harness_cases_have_one_original_name_and_one_body(self):
        self.check_family(Path(__file__).resolve().parents[2] / 'crates/rondata/src/diff')

    def test_nested_missing_misdirected_duplicate_or_unmounted_cases_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            (folder / 'harness/tests').mkdir(parents=True)
            parent = folder / 'harness.rs'
            body = folder / 'harness/tests/cases_00.rs'
            registration = ('pub(crate) mod tests {\n    mod cases_00;\n    #[test]\n'
                            '    #[ignore]\n    fn original() {\n'
                            '        cases_00::original();\n    }\n}\n')
            definition = ('pub(super) use bodies::*;\n\nmod bodies {\n    use super::super::*;\n'
                          '    pub(in super::super) fn original() {}\n}\n')
            parent.write_text(registration)
            body.write_text(definition)
            self.check_family(folder)
            for bad in [registration.replace('::original()', '::other()'),
                        registration + registration,
                        registration.replace('    mod cases_00;\n', '')]:
                parent.write_text(bad)
                with self.assertRaises(AssertionError):
                    self.check_family(folder)
            parent.write_text(registration)
            for bad in ['', definition + definition]:
                body.write_text(bad)
                with self.assertRaises(AssertionError):
                    self.check_family(folder)


class RuntimePolicy(unittest.TestCase):
    def test_release_build_reuses_compilation_without_relaxing_checks(self):
        manifest = Path(__file__).resolve().parents[2] / 'Cargo.toml'
        profile = tomllib.loads(manifest.read_text())['profile']['release']
        self.assertIs(profile.get('incremental'), True)
        self.assertEqual(profile.get('codegen-units'), 256)
        self.assertIs(profile.get('overflow-checks'), True)
        self.assertEqual(profile.get('opt-level', 3), 3)
        self.assertEqual(profile.get('panic', 'unwind'), 'unwind')

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

    def test_auto_is_workload_scoped_and_dispatchers_preserve_choice(self):
        for name in ('diff::third_pair::authored', 'diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned'):
            self.assertEqual(runtime.child_environment({}, platform='darwin', test_name=name)['MallocLargeCache'], '0')
        for name in ('authored', 'diff::coverage::every_key_the_dump_prints_is_read_or_pinned'):
            self.assertNotIn('MallocLargeCache', runtime.child_environment({}, platform='darwin', test_name=name))
            self.assertEqual(runtime.child_environment({'RON_TEST_ALLOCATOR': 'bounded'}, platform='darwin', test_name=name)['MallocLargeCache'], '0')
            self.assertEqual(runtime.child_environment({'MallocLargeCache': '1'}, platform='darwin', test_name=name)['MallocLargeCache'], '1')
        dispatch = runtime.child_environment({}, platform='darwin', dispatcher=True)
        self.assertNotIn('MallocLargeCache', dispatch)
        self.assertNotIn('MallocLargeCache', runtime.child_environment(dispatch, platform='darwin', test_name='authored'))
        self.assertEqual(runtime.child_environment(dispatch, platform='darwin', test_name='diff::third_pair::authored')['MallocLargeCache'], '0')

    def test_isolated_child_receives_and_reports_effective_policy(self):
        with tempfile.TemporaryDirectory() as d, patch.dict(os.environ, {}, clear=True), patch.object(runtime.sys, 'platform', 'darwin'):
            root = Path(d)
            (root / 'tests').mkdir()
            child = MagicMock()
            child.poll.return_value = 0
            child.returncode = 0
            child.__enter__.return_value = child
            with patch.object(runner.subprocess, 'Popen', return_value=child) as spawn, patch.object(runner, 'binary_stamp', return_value=(1,)), patch.object(runner, 'read_result', return_value=(1, (1.0, 0.0), 123)):
                result = runner.run_one(root / 'binary', 'diff::third_pair::authored', 0, root, root / 'resources', 128, lambda: False)
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
            report = gate.gate(root, report_dir=root / 'report', isolated=False, run=run)
            self.assertEqual(len(calls), 6)
            self.assertEqual(json.loads((report / 'gate-policy.json').read_text())['test_allocator']['MallocLargeCache'], '0')
            self.assertNotIn('MallocLargeCache', os.environ)

    def test_isolated_gate_does_not_force_policy_on_unrelated_tests(self):
        import json
        with tempfile.TemporaryDirectory() as d, patch.dict(os.environ, {}, clear=True), patch.object(runtime.sys, 'platform', 'darwin'):
            root = Path(d)
            (root / 'Data').mkdir()
            (root / 'Data/rules.xml').write_text('authored')
            def run(command, **kwargs):
                self.assertNotIn('MallocLargeCache', kwargs['env'])
                if '--release' in command:
                    (Path(kwargs['env']['RON_FIXTURE_AUDIT_DIR']) / '123.tsv').write_text('1\t66697874757265\t74657374\n')
                    Path(kwargs['log']).write_text('     Running unittests src/lib.rs (target/release/deps/rondata-authored)\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n')
            report = gate.gate(root, report_dir=root / 'report', isolated=True, run=run)
            policy = json.loads((report / 'gate-policy.json').read_text())['test_allocator']
            self.assertEqual(policy['requested'], 'auto')
            self.assertIsNone(policy['MallocLargeCache'])
