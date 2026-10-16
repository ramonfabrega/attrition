#!/usr/bin/env python3
"""The gate refuses absent installs and never launches a child without RON_INSTALL."""
import importlib.util
import json
import os
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


# A word filter RELEASE_LOG's rondata ran: what a lane gate names (1567).
WORD = ['the_gate_has_a_bounded_test_width']


def release_ran(kwargs, log=RELEASE_LOG, present=True):
    """What a release child leaves behind: the fixture audit and the log."""
    write_request(kwargs['env']['RON_FIXTURE_AUDIT_DIR'], present)
    Path(kwargs['log']).write_text(log)


class IsolatedGate(unittest.TestCase):
    def test_all_six_phases_and_fixture_reporting_survive_isolation(self):
        with tempfile.TemporaryDirectory() as d:
            install = Path(d)/'install'
            (install/'Data').mkdir(parents=True)
            (install/'Data/rules.xml').write_text('authored fixture')
            calls = []
            def run(command, **kwargs):
                calls.append(command)
                self.assertEqual(kwargs['env']['RON_GATE_RESOURCE_POLICY'], '1')
                if '--release' in command:
                    release_ran(kwargs, RELEASE_LOG.replace('FAILED', 'ok'))
            report = gate.gate(install, report_dir=Path(d)/'report', require_fixtures=True,
                               test_threads=4, isolated=True, run=run)
            self.assertEqual(len(calls), 6)
            self.assertIn('tools/isolated_tests.py', calls[4])
            self.assertEqual(json.loads((report/'fixture-coverage.json').read_text())['missing_fixtures'], 0)
            self.assertEqual(json.loads((report/'gate-policy.json').read_text())['memory_cap_gib'], 32)

    def test_small_checks_share_capacity_and_scoped_gate_cannot_silently_widen(self):
        self.assertEqual(gate.phase_budget(['python3', 'tools/offline_tests.py']), (2, 1024))
        self.assertEqual(gate.phase_budget(['cargo', 'fmt', '--check']), (2, 1024))
        self.assertEqual(gate.phase_budget(['cargo', 'clippy']), (8, 32768))
        self.assertEqual(gate.phase_budget(['zsh', 'tools/guard.sh', '--built']), (8, 32768))
        # Selection and mode are checked separately from the admission wrapper.


class SharedEntryPoints(unittest.TestCase):
    def test_default_complete_is_isolated_but_scoped_and_lane_keep_selection(self):
        for options, isolated in [({}, True), ({'isolated': False}, False),
                                  ({'scope': 'HEAD'}, True), ({'lane': True, 'tests': WORD}, True)]:
            with self.subTest(options=options), tempfile.TemporaryDirectory() as d:
                install = Path(d)/'install'
                (install/'Data').mkdir(parents=True)
                (install/'Data/rules.xml').touch()
                calls = []
                def run(command, **kwargs):
                    calls.append(command)
                    self.assertEqual(kwargs['env']['RON_GATE_RESOURCE_POLICY'], '1')
                    self.assertEqual(kwargs['env']['CARGO_BUILD_JOBS'], '8')
                    if '--release' in command:
                        release_ran(kwargs, RELEASE_LOG.replace('FAILED', 'ok'))
                with patch.object(gate, 'branch_paths', return_value=[]):
                    gate.gate(install, report_dir=Path(d)/'report', run=run, **options)
                self.assertEqual('tools/isolated_tests.py' in calls[4], isolated)
                self.assertEqual(len(calls), 6)

    def test_isolated_and_combined_receive_identical_exact_exclusions(self):
        for options in [{}, {'scope': 'HEAD'}, {'lane': True, 'tests': WORD}]:
            commands = []
            for isolated in [False, True]:
                with tempfile.TemporaryDirectory() as d:
                    install = Path(d)
                    (install/'Data').mkdir()
                    (install/'Data/rules.xml').touch()
                    def run(command, **kwargs):
                        if '--release' in command:
                            commands.append(command)
                            release_ran(kwargs, RELEASE_LOG.replace('FAILED', 'ok'))
                    with patch.object(gate, 'branch_paths', return_value=[]):
                        gate.gate(install, report_dir=install/'report', run=run, isolated=isolated, **options)
            selections = [[c[i+1] for i, arg in enumerate(c) if arg == '--skip'] for c in commands]
            self.assertEqual(selections[0], selections[1])
            self.assertTrue(bool(selections[1]) == bool(options))

    def test_standalone_guard_enters_once_and_preserves_filters_and_lane(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            tools = root/'tools'
            (tools/'explore').mkdir(parents=True)
            (root/'bin').mkdir()
            (tools/'guard.sh').write_text((gate.ROOT/'tools/guard.sh').read_text())
            # A recording admission shim avoids process monitoring in this authored test.
            (tools/'resource_budget.py').write_text(
                'import sys,subprocess,json\nfrom pathlib import Path\n'
                'with Path("admissions").open("a") as f: f.write(json.dumps(sys.argv[1:])+"\\n")\n'
                'raise SystemExit(subprocess.run(sys.argv[sys.argv.index("--")+1:]).returncode)\n')
            for name in ['test_brief', 'test_queueledger']:
                (tools/'explore'/f'{name}.py').write_text('import unittest\nclass Authored(unittest.TestCase):\n def test_ok(self): self.assertTrue(True)\n')
            (tools/'queueledger.py').write_text('')
            cargo = root/'bin/cargo'
            cargo.write_text('#!/bin/sh\nprintf "%s|%s|%s|%s\\n" "$*" "$CARGO_BUILD_JOBS" "$RUST_TEST_THREADS" "$RON_LANE" >> cargo-calls\n')
            cargo.chmod(0o700)
            for lane in ['', '1']:
                env = dict(os.environ, PATH=str(root/'bin')+os.pathsep+os.environ['PATH'], RON_LANE=lane)
                env.pop('RON_GUARD_ADMITTED', None)
                done = subprocess.run(['zsh', str(tools/'guard.sh'), '--built', 'authored_filter'],
                                      env=env, capture_output=True, text=True, timeout=10)
                self.assertEqual(done.returncode, 0, done.stderr)
            self.assertEqual(len((root/'admissions').read_text().splitlines()), 2)
            calls = (root/'cargo-calls').read_text().splitlines()
            self.assertEqual(sum('authored_filter' in line for line in calls), 2)
            self.assertEqual(sum('the_handoff' in line for line in calls), 1)
            self.assertTrue(all('|8|2|' in line for line in calls))
            (tools/'check.sh').write_text((gate.ROOT/'tools/check.sh').read_text())
            env.pop('RON_GUARD_ADMITTED', None)
            done = subprocess.run(['zsh', str(tools/'check.sh'), 'test', '-p', 'sim', 'authored_filter'],
                                  env=env, capture_output=True, text=True, timeout=10)
            self.assertEqual(done.returncode, 0, done.stderr)
            self.assertEqual(len((root/'admissions').read_text().splitlines()), 3)
            self.assertIn('test --jobs 8 -p sim authored_filter|8|2|', (root/'cargo-calls').read_text())
            done = subprocess.run(['zsh', str(tools/'check.sh'), 'invalid'],
                                  env=env, capture_output=True, text=True, timeout=10)
            self.assertEqual(done.returncode, 2)
            self.assertEqual(len((root/'admissions').read_text().splitlines()), 3)
            cargo.write_text('#!/bin/sh\nexit 7\n')
            done = subprocess.run(['zsh', str(tools/'check.sh'), 'test'], env=env,
                                  capture_output=True, text=True, timeout=10)
            self.assertEqual(done.returncode, 7)

    def test_combined_release_is_admitted_and_isolated_dispatcher_is_not_double_admitted(self):
        env = {'RON_GATE_RESOURCE_POLICY': '1'}
        for command, admitted in [(['cargo', 'test', '--release'], True),
                                  ([sys.executable, 'tools/isolated_tests.py', '--release'], False),
                                  (['zsh', 'tools/guard.sh', '--built'], True)]:
            with self.subTest(command=command), patch.object(gate.subprocess, 'run') as run:
                gate.run_logged(command, cwd='.', env=env, check=True)
                actual = run.call_args.args[0]
                self.assertEqual(any(str(v).endswith('resource_budget.py') for v in actual), admitted)
                if 'tools/guard.sh' in command:
                    self.assertEqual(run.call_args.kwargs['env']['RON_GUARD_ADMITTED'], '1')
        self.assertNotIn('RON_GUARD_ADMITTED', env)


class StaleAgentWorktrees(unittest.TestCase):
    # Loop 2724: three `agent-*` subagent worktrees of the thirty-fifth
    # tranche stood a day under the main checkout, two with unmerged tips,
    # and nothing named them. The booking gate refuses one older than the
    # limit; a lane gate never looks (the directory is shared by every lane).
    PORCELAIN = """\
worktree /Users/x/code/fun/attrition
HEAD 054d603c12000898e95165608c837e90ffac2580
branch refs/heads/main

worktree /Users/x/code/fun/attrition/.claude/worktrees/agent-a0651062ff871c606
HEAD b4e0ac06aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
branch refs/heads/worktree-agent-a0651062ff871c606

worktree /Users/x/code/fun/attrition/.claude/worktrees/agent-a0bec46101a67bd2c
HEAD e4d4aaa6aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
branch refs/heads/worktree-agent-a0bec46101a67bd2c

worktree /Users/x/code/fun/attrition/.claude/worktrees/replan-pdb
HEAD 2e78ba2eaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
branch refs/heads/worktree-replan-pdb
"""

    def test_a_tree_past_the_limit_is_named_and_a_young_one_is_not(self):
        ages = {'agent-a0651062ff871c606': 21.5, 'agent-a0bec46101a67bd2c': 1.2}
        stale = gate.stale_agent_worktrees(self.PORCELAIN, lambda p: ages[Path(p).name])
        self.assertEqual([(Path(p).name, round(h, 1)) for p, h in stale],
                         [('agent-a0651062ff871c606', 21.5)])

    def test_a_tree_whose_age_is_unknown_is_not_named(self):
        stale = gate.stale_agent_worktrees(self.PORCELAIN, lambda p: None)
        self.assertEqual(stale, [])

    def test_the_booking_gate_refuses_and_the_lane_gate_does_not(self):
        stale = [('/x/.claude/worktrees/agent-a0651062ff871c606', 21.5)]
        with self.assertRaises(ValueError) as cm:
            gate.refuse_stale_agent_worktrees(stale, lane=False)
        self.assertIn('agent-a0651062ff871c606', str(cm.exception))
        self.assertIn('21.5 h', str(cm.exception))
        gate.refuse_stale_agent_worktrees(stale, lane=True)
        gate.refuse_stale_agent_worktrees([], lane=False)


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
                  ['zsh','tools/guard.sh','--built']]
        self.assertEqual(gate.steps_line(commands, commands),
                         'Gate steps: 6 of 6 ran (offline, clippy, fmt, survey, release, guard)')
        self.assertEqual(gate.steps_line(commands, commands[:2]),
                         'Gate steps: 2 of 6 ran (offline, clippy); not reached: fmt, survey, release, guard')
        self.assertEqual(gate.steps_line(commands, []),
                         'Gate steps: 0 of 6 ran (none); not reached: offline, clippy, fmt, survey, release, guard')

    def test_wider_release_is_capped_and_other_children_stay_conservative(self):
        for width in (2, 3, 4, 6, 8):
            with self.subTest(width=width), tempfile.TemporaryDirectory() as tmp:
                path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
                calls=[]
                def run(command, **kwargs):
                    calls.append(command)
                    self.assertEqual(kwargs['env']['RUST_TEST_THREADS'], '2')
                    self.assertNotIn('RON_TEST_MEMCAP_GIB', kwargs['env'])
                    if '--release' in command:
                        # The thirty-fifth steer: 20 GiB for a lane's width, 40 for the booking gate's.
                        self.assertEqual(command[:3], ['zsh','tools/memcap.sh','20' if width <= 4 else '32'])
                        self.assertEqual(command[-2:], ['--', f'--test-threads={width}'])
                        self.assertNotIn('--skip', command)
                        release_ran(kwargs)
                with patch.dict('os.environ', RUST_TEST_THREADS='16', RON_TEST_MEMCAP_GIB='999'):
                    gate.gate(path, report_dir=path/'report', test_threads=width, run=run, isolated=False)
                self.assertEqual(len(calls), 6)
                policy=json.loads((path/'report/gate-policy.json').read_text())
                self.assertEqual(policy['release_test_threads'], width)
                self.assertEqual(policy['memory_cap_gib'], 20 if width <= 4 else 32)

    def test_unmeasured_width_never_runs(self):
        for width in (0, 1, 5, 7, 16, '4', 4.0, True):
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
            gate.gate(path, report_dir=path/'report', run=run, isolated=False)
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
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD)
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
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD)
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
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD)
            self.assertEqual(gate.step_name(calls[-1][0]), 'release')

    def test_a_lane_gate_refuses_a_tree_with_no_journal(self):
        # Parked 1699: 1639's chain died on a `sed`, the journal was never
        # written, and the gate ran green without it.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with self.assertRaises(ValueError) as refused:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, item=999999)
            self.assertIn('item-999999.md', str(refused.exception))
            self.assertEqual(calls, [])
            # An item whose journal is in the tree gates (1735's is).
            self.assertTrue(gate.journal_of(1735))
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report2', run=run, lane=True, tests=WORD, item=1735)
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertIn('Gate pid: ', said)
            self.assertEqual(len(calls), 6)
            with self.assertRaises(ValueError):
                gate.gate(path, report_dir=path/'report3', run=run, item=1735)

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
            root/'crates/rondata/src/census.rs',
            root/'crates/sim/src/docs_guard.rs'])
        for name in gate.COMMANDERS_LINES:
            self.assertIn(f'fn {name}()', source, name)
        guard=(root/'tools/guard.sh').read_text()
        self.assertIn('RON_LANE', guard)

    def test_the_census_s_pin_is_one_of_the_commander_s_lines(self):
        # Loop 2367 (the thirty-fourth pass): two sweep lanes re-pinned
        # `census::SIMULATION_BACKED` in the same landing and seven of the
        # thirteen merges of one commander session backed out on it. The
        # shared pin is the commander's at booking, so its red is forgiven on
        # a lane, like the queue's lines.
        self.assertIn('the_census_s_simulation_share_is_pinned', gate.COMMANDERS_LINES)

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

    # The lane gate's suite (1567, DECISIONS 63 amended): every landing ran
    # the whole release suite three times on one box, and a third lane
    # bought no wall clock. The lane gate leaves the slow timed tests — the
    # two coverage pins first — to the booking gate, and runs the word's.
    TIMINGS = ('# a comment\n'
               '118.13\tdiff::coverage::every_key_the_dump_prints_is_read_or_pinned\n'
               '79.70\tdiff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned\n'
               '50.40\tdiff::harness::tests::run218_s_word_frame_is_widened_whole\n'
               '19.58\tdiff::second::tests::run589_s_word_frame_is_widened_whole\n'
               '1.00\tdiff::golden::at_the_budget\n'
               '0.02\tdiff::floors::tests::the_gate_has_a_bounded_test_width\n')

    def release_command(self, touched=(), **kw):
        kw.setdefault('isolated', False)  # Retain reference-runner selection assertions.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def run(command, **kwargs):
                calls.append(command)
                if '--release' in command:
                    release_ran(kwargs)
            with patch('builtins.print'), patch.object(gate, 'branch_paths', return_value=list(touched)):
                gate.gate(path, report_dir=path/'report', run=run, test_threads=4, **kw)
            policy=json.loads((path/'report/gate-policy.json').read_text())
        return next(c for c in calls if '--release' in c), policy

    def test_a_lane_gate_leaves_the_slow_timed_tests_to_the_booking_gate(self):
        command, policy = self.release_command(lane=True, tests=WORD)
        skipped=[command[i+1] for i, a in enumerate(command) if a == '--skip']
        self.assertIn('--exact', command)
        self.assertIn('diff::coverage::every_key_the_dump_prints_is_read_or_pinned', skipped)
        self.assertIn('diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned', skipped)
        self.assertNotIn('diff::floors::tests::the_gate_has_a_bounded_test_width', skipped)
        self.assertEqual(len(skipped), policy['lane']['skipped'])
        self.assertGreater(len(skipped), 400)
        self.assertEqual(command[:8], ['zsh','tools/memcap.sh','20','cargo','test','--release','--no-fail-fast','--'])

    def test_a_branch_that_can_move_the_diff_pins_widens_the_lane_gate(self):
        # Loop 2415 (the thirty-fifth pass; DECISIONS 71 amended): both kill
        # rules of the thirty-fifth tranche tripped by one route — 2384's
        # 144 `aggression` pins and 2400's floor 17379 → 14461 were red only
        # at the booking gate, because the lane gate leaves the timed
        # `diff::` tests to it. A branch that touches sim code outside
        # `sweep/` or a `rondata::diff` row list can move those pins, so its
        # lane gate skips nothing; and the whole-suite run before the gate
        # (Loop 2409) is then the gate itself.
        self.assertEqual(gate.lane_widens(['crates/sim/src/sweep/x.rs', 'docs/AI.md', 'tools/x.py',
                                           'crates/rondata/src/census.rs']), [])
        self.assertEqual(gate.lane_widens(['crates/sim/src/diplomacy.rs', 'crates/rondata/src/diff/leader.rs',
                                           'crates/rondata/src/gamelog.rs', 'Cargo.lock']),
                         ['Cargo.lock', 'crates/rondata/src/diff/leader.rs', 'crates/rondata/src/gamelog.rs',
                          'crates/sim/src/diplomacy.rs'])
        command, policy = self.release_command(touched=['crates/sim/src/diplomacy.rs', 'docs/AI.md'],
                                               lane=True, tests=WORD)
        self.assertNotIn('--skip', command)
        self.assertEqual(policy['lane']['skipped'], 0)
        self.assertEqual(policy['lane']['widened'], ['crates/sim/src/diplomacy.rs'])
        command, policy = self.release_command(touched=['crates/sim/src/sweep/x.rs'], lane=True, tests=WORD)
        self.assertIn('--skip', command)
        self.assertEqual(policy['lane']['widened'], [])

    def test_a_scoped_booking_gate_runs_the_timed_tests_only_when_a_merge_can_move_a_pin(self):
        # The thirty-fifth steer, Ramon's word: the booking gate after every
        # merge was 22 whole suites a tranche; scoped to the merges since the
        # last whole gate it runs the timed tests when one of them can move a
        # pin, and every third landing takes the whole gate. It forgives
        # nothing and needs no journal.
        command, policy = self.release_command(touched=['docs/AI.md', 'crates/sim/src/sweep/x.rs'],
                                               scope='abc1234')
        self.assertIn('--skip', command)
        self.assertIn('--exact', command)
        self.assertEqual(policy['scope']['widened'], [])
        self.assertGreater(policy['scope']['skipped'], 400)
        self.assertNotIn('lane', policy)
        command, policy = self.release_command(touched=['crates/rondata/src/gamelog.rs'], scope='abc1234')
        self.assertNotIn('--skip', command)
        self.assertEqual(policy['scope']['widened'], ['crates/rondata/src/gamelog.rs'])
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            with self.assertRaises(ValueError):
                gate.gate(path, lane=True, tests=WORD, scope='abc1234', run=lambda *a, **k: self.fail('ran'))

    def test_the_booking_gate_skips_nothing(self):
        command, policy = self.release_command()
        self.assertEqual(command[-2:], ['--', '--test-threads=4'])
        self.assertNotIn('lane', policy)

    def test_the_skips_are_the_over_budget_rows_the_word_does_not_name(self):
        self.assertEqual(gate.lane_skips(self.TIMINGS, ['run589_s_word']), [
            'diff::coverage::every_key_the_dump_prints_is_read_or_pinned',
            'diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned',
            'diff::harness::tests::run218_s_word_frame_is_widened_whole'])
        # A word that reads a new key runs the coverage pins itself.
        self.assertEqual(gate.lane_skips(self.TIMINGS, ['coverage::', 'run218']),
                         ['diff::second::tests::run589_s_word_frame_is_widened_whole'])

    def test_a_lane_gate_names_its_word_and_the_booking_gate_takes_none(self):
        for kw in ({'lane': True}, {'lane': True, 'tests': []}, {'lane': True, 'tests': ['']},
                   {'tests': WORD}):
            with self.subTest(**kw), tempfile.TemporaryDirectory() as tmp:
                path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
                with self.assertRaises(ValueError):
                    gate.gate(path, report_dir=path/'report', run=lambda *a, **k: self.fail('ran'), **kw)

    def test_a_word_filter_that_matched_no_test_is_red(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def run(command, **kwargs):
                calls.append(command)
                if '--release' in command:
                    release_ran(kwargs, log=RELEASE_LOG.replace(' ... FAILED', ' ... ok'))
            with patch('builtins.print'), self.assertRaises(ValueError) as caught:
                gate.gate(path, report_dir=path/'report', run=run, lane=True,
                          tests=WORD + ['run683_s_wrod'])
            self.assertIn('run683_s_wrod', str(caught.exception))
            self.assertNotIn('the_gate_has', str(caught.exception))
            self.assertEqual(gate.step_name(calls[-1]), 'release')

    def test_the_timings_file_is_the_suite_s(self):
        # The constant names a file that exists, holds the two coverage pins
        # over budget, and parses whole.
        text=gate.TIMINGS.read_text()
        skips=gate.lane_skips(text, [])
        self.assertIn('diff::coverage::every_key_the_dump_prints_is_read_or_pinned', skips)
        self.assertGreater(len(skips), 400)


if __name__ == '__main__':
    unittest.main()
