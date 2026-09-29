#!/usr/bin/env python3
"""The gate refuses absent installs and never launches a child without RON_INSTALL."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('release_gate', Path(__file__).resolve().parents[1]/'release_gate.py')
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def write_request(directory, present=True, name='fixture', test='test'):
    with (Path(directory)/'123.tsv').open('a') as stream:
        stream.write(f"{int(present)}\t{name.encode().hex()}\t{test.encode().hex()}\n")


# A release log in cargo's own shape: `Running` on stderr and the results on
# stdout, both in one stream. rondata red on the two queue tests by design,
# the sim suite green behind it — the run 339 says the gate never showed.
RELEASE_LOG = """\
     Running unittests src/lib.rs (target/release/deps/fixed-0a1b2c)

running 13 tests
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/main.rs (target/release/deps/rondata-3d4e5f)

running 305 tests
test diff::endpoint::the_handoff_s_endpoint_is_the_pinned_counts ... FAILED
test diff::floors::the_handoff_s_scoreboard_is_the_floors ... FAILED
test diff::floors::the_gate_has_a_bounded_test_width ... ok

failures:

---- diff::floors::the_handoff_s_scoreboard_is_the_floors stdout ----
docs/QUEUE.md's Scoreboard line is stale

failures:
    diff::endpoint::the_handoff_s_endpoint_is_the_pinned_counts
    diff::floors::the_handoff_s_scoreboard_is_the_floors

test result: FAILED. 303 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 101.20s

     Running unittests src/lib.rs (target/release/deps/sim-6a7b8c)

running 831 tests
test result: ok. 831 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.87s

   Doc-tests sim

running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

error: 1 target failed:
    `-p rondata --bin rondata`
"""


def release_ran(kwargs, log=RELEASE_LOG, present=True):
    """What a release child leaves behind: the fixture audit and the log."""
    write_request(kwargs['env']['RON_FIXTURE_AUDIT_DIR'], present)
    Path(kwargs['log']).write_text(log)


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
                    release_ran(kwargs)
            gate.gate(path, report_dir=path/'report', run=run)
            self.assertEqual(len(calls), 6)
            self.assertEqual(calls[0], [sys.executable, 'tools/offline_tests.py'])
            self.assertEqual(calls[3][:4], ['cargo','run','-p','rondata'])
            self.assertIn('--release', calls[4])

    def test_clippy_and_fmt_run_before_the_release_suite(self):
        # 639: a worker whose suite was red only on the queue's lines stopped
        # there, and clippy — after it in the list — never ran; the commander's
        # gate then failed on that worker's own `collapsible_if`. The cheap
        # checks go first, and a suite that fails still leaves them said.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def run(command, **kwargs):
                calls.append(command)
                if '--release' in command:
                    release_ran(kwargs)
                    raise subprocess.CalledProcessError(101, command)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run)
            names=[gate.step_name(c) for c in calls]
            self.assertEqual(names, ['offline','clippy','fmt','survey','release'])
            self.assertLess(names.index('clippy'), names.index('release'))
            self.assertLess(names.index('fmt'), names.index('release'))

    def test_the_steps_line_names_what_a_red_gate_never_reached(self):
        # The other half of 639: "red only on my lines" is read off the gate's
        # last line, never inferred from what a worker remembers running.
        commands=[[sys.executable,'tools/offline_tests.py'],['cargo','clippy'],['cargo','fmt','--check'],
                  ['cargo','run','-p','rondata','--','x'],['zsh','tools/memcap.sh','20','cargo','test','--release'],
                  ['zsh','tools/guard.sh']]
        self.assertEqual(gate.steps_line(commands, commands),
                         'Gate steps: 6 of 6 ran (offline, clippy, fmt, survey, release, guard)')
        self.assertEqual(gate.steps_line(commands, commands[:2]),
                         'Gate steps: 2 of 6 ran (offline, clippy); not reached: fmt, survey, release, guard')
        self.assertEqual(gate.steps_line(commands, []),
                         'Gate steps: 0 of 6 ran (none); not reached: offline, clippy, fmt, survey, release, guard')

    def test_wider_release_is_capped_and_other_children_stay_conservative(self):
        for width in (2, 3, 4):
            with self.subTest(width=width), tempfile.TemporaryDirectory() as tmp:
                path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
                calls=[]
                def run(command, **kwargs):
                    calls.append(command)
                    self.assertEqual(kwargs['env']['RUST_TEST_THREADS'], '2')
                    self.assertNotIn('RON_TEST_MEMCAP_GIB', kwargs['env'])
                    if '--release' in command:
                        self.assertEqual(command[:3], ['zsh','tools/memcap.sh','20'])
                        self.assertEqual(command[-2:], ['--', f'--test-threads={width}'])
                        self.assertNotIn('--skip', command)
                        release_ran(kwargs)
                with patch.dict('os.environ', RUST_TEST_THREADS='16', RON_TEST_MEMCAP_GIB='999'):
                    gate.gate(path, report_dir=path/'report', test_threads=width, run=run)
                self.assertEqual(len(calls), 6)
                policy=json.loads((path/'report/gate-policy.json').read_text())
                self.assertEqual(policy['release_test_threads'], width)
                self.assertEqual(policy['memory_cap_gib'], 20)

    def test_unmeasured_width_never_runs(self):
        for width in (0, 1, 5, 16, '4', 4.0, True):
            with self.subTest(width=width), tempfile.TemporaryDirectory() as tmp:
                path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
                with self.assertRaises(ValueError):
                    gate.gate(path, test_threads=width, run=lambda *a, **k: self.fail('ran invalid width'))

    def test_offline_failure_stops_before_install_survey_and_release(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def run(command, **kwargs):
                calls.append(command)
                if command == [sys.executable, 'tools/offline_tests.py']:
                    raise subprocess.CalledProcessError(1, command)
                if '--release' in command:
                    release_ran(kwargs)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run)
            self.assertEqual(calls, [[sys.executable, 'tools/offline_tests.py']])

    def test_failed_survey_stops_before_tests(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def fail(command, **kwargs):
                calls.append(command)
                if command[:2] == ['cargo', 'run']:
                    raise subprocess.CalledProcessError(1, command)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=fail)
            self.assertEqual([gate.step_name(c) for c in calls], ['offline','clippy','fmt','survey'])

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
                        release_ran(kwargs, present=False)
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
                    release_ran(kwargs)
                    raise subprocess.CalledProcessError(1, command)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run)
            report=json.loads((path/'report/fixture-coverage.json').read_text())
            self.assertFalse(report['release_completed'])
            # 339: the red binary does not hide the ones behind it.
            tests=json.loads((path/'report/test-summary.json').read_text())
            self.assertFalse(tests['release_completed'])
            self.assertEqual([row['name'] for row in tests['binaries']], ['fixed','rondata','sim','doc-tests sim'])
            self.assertEqual(tests['failed_tests'], [
                'rondata::diff::endpoint::the_handoff_s_endpoint_is_the_pinned_counts',
                'rondata::diff::floors::the_handoff_s_scoreboard_is_the_floors'])
            self.assertEqual(tests['binaries_failed'], 1)
            self.assertEqual(tests['binaries'][2], {'name':'sim','result':'ok','passed':831,'failed':0})

    def test_release_runs_every_binary_past_a_red_one(self):
        # 339: cargo's default stops at the first failing binary, and on a
        # word-moving item rondata is red by design — so the sim suite,
        # no_float, soak and docs_guard included, never ran in a worker's gate.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def run(command, **kwargs):
                calls.append(command)
                if '--release' in command:
                    self.assertIn('--no-fail-fast', command)
                    self.assertLess(command.index('--no-fail-fast'), command.index('--'))
                    release_ran(kwargs)
            gate.gate(path, report_dir=path/'report', run=run)
            self.assertIn('--no-fail-fast', calls[4])

    def test_release_without_a_test_result_is_unobserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            def run(command, **kwargs):
                if '--release' in command:
                    release_ran(kwargs, log='     Running unittests src/lib.rs (target/release/deps/sim-6a7b8c)\n')
            with self.assertRaises(ValueError):
                gate.gate(path, report_dir=path/'report', run=run)

    def test_the_default_runner_keeps_the_exit_code_and_the_log(self):
        with tempfile.TemporaryDirectory() as tmp:
            log=Path(tmp)/'release-tests.log'
            script='import sys; print("test result: ok. 1 passed; 0 failed; 0 ignored"); print("boom", file=sys.stderr); sys.exit(137)'
            with self.assertRaises(subprocess.CalledProcessError) as caught:
                gate.run_logged([sys.executable,'-c',script], cwd=tmp, env={}, check=True, log=str(log))
            self.assertEqual(caught.exception.returncode, 137)
            text=log.read_text()
            self.assertIn('test result: ok. 1 passed', text)
            self.assertIn('boom', text)
            self.assertEqual(gate.summarize_tests(log, release_completed=False)['binaries'][0]['passed'], 1)

    # The lane gate (parked 969, the seventeenth pass). Twenty of twenty
    # worker gates in one tranche exited 1 by design — a worker re-pins a
    # word and may not write the queue's line for it — so none reached
    # `guard`, and four had a red of their own among the expected ones.
    def lane_run(self, log, returncode=101):
        calls=[]
        def run(command, **kwargs):
            calls.append((command, kwargs['env']))
            if '--release' in command:
                release_ran(kwargs, log=log)
                raise subprocess.CalledProcessError(returncode, command)
        return calls, run

    def test_a_lane_gate_red_only_on_the_commander_s_lines_reaches_the_guard(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report', run=run, lane=True)
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertEqual([gate.step_name(c) for c, _ in calls],
                             ['offline','clippy','fmt','survey','release','guard'])
            self.assertEqual(calls[5][1].get('RON_LANE'), '1')
            self.assertIn("Lane verdict: red only on the commander's lines (2)", said)
            self.assertIn('Gate steps: 6 of 6 ran', said)

    def test_a_lane_gate_red_on_the_worker_s_own_test_stops_and_names_it(self):
        own=RELEASE_LOG.replace(
            'test diff::floors::the_gate_has_a_bounded_test_width ... ok',
            'test diff::floors::the_gate_has_a_bounded_test_width ... FAILED')
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(own)
            with patch('builtins.print') as printed, self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run, lane=True)
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertEqual(gate.step_name(calls[-1][0]), 'release')
            self.assertIn("Lane verdict: red on the worker's own (1)", said)
            self.assertIn('rondata::diff::floors::the_gate_has_a_bounded_test_width', said)

    def test_a_lane_gate_never_forgives_a_kill(self):
        # memcap's 137 with the commander's tests red before it died: the
        # suite did not finish, so "only" is not something the log can say.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG, returncode=137)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run, lane=True)
            self.assertEqual(gate.step_name(calls[-1][0]), 'release')

    def test_the_commander_s_gate_forgives_nothing(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run)
            self.assertEqual(gate.step_name(calls[-1][0]), 'release')
            self.assertNotIn('RON_LANE', calls[-1][1])

    def test_the_commander_s_lines_are_tests_that_exist(self):
        # A renamed test would leave the list forgiving nothing, in silence.
        root=Path(__file__).resolve().parents[2]
        source='\n'.join(p.read_text() for p in [
            root/'crates/rondata/src/diff/floors.rs',
            root/'crates/rondata/src/diff/endpoint.rs',
            root/'crates/sim/src/docs_guard.rs'])
        for name in gate.COMMANDERS_LINES:
            self.assertIn(f'fn {name}()', source, name)
        guard=(root/'tools/guard.sh').read_text()
        self.assertIn('RON_LANE', guard)

    def test_every_test_that_reads_the_handoff_is_one_of_the_commander_s_lines(self):
        # The other direction (the nineteenth pass): a handoff guard added
        # and left off the list turns every lane's gate red on a line the
        # worker may not write.
        import re
        root=Path(__file__).resolve().parents[2]
        source='\n'.join(p.read_text() for p in [
            root/'crates/rondata/src/diff/floors.rs',
            root/'crates/rondata/src/diff/endpoint.rs'])
        reads=set(re.findall(r'fn (the_handoff_s_\w+)\(\)', source))
        self.assertGreaterEqual(len(reads), 5)
        self.assertEqual(reads - set(gate.COMMANDERS_LINES), set())


if __name__ == '__main__':
    unittest.main()
