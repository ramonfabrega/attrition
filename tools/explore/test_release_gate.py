#!/usr/bin/env python3
"""The gate refuses absent installs and never launches a child without RON_INSTALL."""
import importlib.util
import json
import io
import contextlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from types import SimpleNamespace
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


def git_fixture(commits=(), theirs=(), ours=(), theirs_diff=None):
    """A `git` runner for the gate's own reads (`branch_paths`,
    `base_moved`), so no case reads the real tree. Item 3366: 3360's lane
    gate refused for a moved base, and seven of these cases failed above
    the refusal because they read the tree the gate ran in. `theirs_diff`
    (Loop 3694) is the base side's `-U0` hunk text by path."""
    def run(args, **kwargs):
        if args[:2] == ['git', 'merge-base']:
            out = 'f0rk\n'
        elif args[:2] == ['git', 'log']:
            out = ''.join(c + '\n' for c in commits)
        elif args[:3] == ['git', 'diff', '--name-only']:
            out = ''.join(p + '\n' for p in (ours if args[-1] == 'HEAD' else theirs))
        elif args[:3] == ['git', 'diff', '-U0'] and args[4] != 'HEAD':
            out = (theirs_diff or {}).get(args[-1], '')  # the base's side (Loop 3694)
        elif args[:3] in (['git', 'diff', '--name-status'], ['git', 'diff', '-U0']):
            out = ''  # `row_only_edits` (Loop 3608): no row-list edit in these cases
        else:
            raise AssertionError(f'the gate ran an unexpected git: {args}')
        return SimpleNamespace(stdout=out, returncode=0)
    return run


# The base has not moved and the branch touched nothing.
UNMOVED = git_fixture()
# 3360's shape: the base gained two landings and both sides touched a table.
MOVED = git_fixture(commits=['c2729a5 Book 3337', '4a8c010 Book 3338'],
                    theirs=['crates/rondata/src/diff/battery.rs', 'docs/QUEUE.md'],
                    ours=['crates/rondata/src/diff/battery.rs', 'crates/sim/src/city.rs'])


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
        self.assertEqual(gate.phase_budget(['zsh', 'tools/guard.sh', '--built']), (2, 4096))
        self.assertEqual(gate.phase_limit(['zsh', 'tools/guard.sh', '--built']), 16384)
        self.assertIsNone(gate.phase_limit(['cargo', 'clippy']))
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
                    gate.gate(install, report_dir=Path(d)/'report', run=run, git=UNMOVED, **options)
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
                        gate.gate(install, report_dir=install/'report', run=run, isolated=isolated,
                                  git=UNMOVED, **options)
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
            (tools/'explore'/'test_sweep_gen.py').write_text('import unittest\nclass FamilyRegistrations(unittest.TestCase):\n def test_ok(self): self.assertTrue(True)\n')
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

    def test_the_receipt_carries_the_step_minutes(self):
        # The forty-second pass: the lane gate is 21.6 of the 33.6 minutes a
        # landing waits and ran 11 to 51 minutes on a box at a load of ten,
        # and nothing in the receipt says which step took them. Red first:
        # no `Gate minutes:` line, and `_run_steps` kept no clock.
        self.assertEqual(gate.minutes_line(['offline', 'clippy', 'release'], [12.0, 114.0, 744.6]),
                         'Gate minutes: offline 0.2, clippy 1.9, release 12.4')
        self.assertEqual(gate.minutes_line(['offline', 'clippy'], [12.0]),
                         'Gate minutes: offline 0.2; not reached: clippy')
        self.assertEqual(gate.minutes_line(['offline'], []), 'Gate minutes: none ran; not reached: offline')
        with tempfile.TemporaryDirectory() as d:
            install = Path(d)/'install'
            (install/'Data').mkdir(parents=True)
            (install/'Data/rules.xml').write_text('authored fixture')
            def run(command, **kwargs):
                if '--release' in command:
                    release_ran(kwargs, RELEASE_LOG.replace('FAILED', 'ok'))
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                gate.gate(install, report_dir=Path(d)/'report', test_threads=4, isolated=True, run=run)
            lines = out.getvalue().splitlines()
            self.assertTrue(lines[-4].startswith('Gate minutes: offline '), lines[-4:])
            self.assertTrue(lines[-3].startswith('Gate steps: 6 of 6 ran'), lines[-4:])

    def test_the_receipt_carries_the_host_load(self):
        # Loop 3091: the admission receipt reads seconds while the lane gates
        # ran at half speed; the load at start and end, beside the cores, is
        # what a pass reads for contention. Red first: no such line was printed.
        self.assertEqual(gate.load_line(21.3, 32.04, 16), 'Host load: 21.3 at start, 32.0 at end, on 16 cores')
        self.assertEqual(gate.load_line(None, None, 0), 'Host load: n/a at start, n/a at end, on 0 cores')
        with tempfile.TemporaryDirectory() as d:
            install = Path(d)/'install'
            (install/'Data').mkdir(parents=True)
            (install/'Data/rules.xml').write_text('authored fixture')
            def run(command, **kwargs):
                if '--release' in command:
                    release_ran(kwargs, RELEASE_LOG.replace('FAILED', 'ok'))
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                gate.gate(install, report_dir=Path(d)/'report', test_threads=4, isolated=True, run=run)
            lines = out.getvalue().splitlines()
            self.assertTrue(lines[-1].startswith('Host load: '), lines[-3:])
            self.assertTrue(lines[-2].startswith('Admission wait: '), lines[-3:])

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
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, git=UNMOVED)
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertEqual([gate.step_name(c) for c, _ in calls],
                             ['offline','clippy','fmt','survey','release','guard'])
            self.assertEqual(calls[5][1].get('RON_LANE'), '1')
            self.assertIn("Lane verdict: red only on the commander's lines (2)", said)
            self.assertIn('Gate steps: 6 of 6 ran', said)

    def test_a_wholly_green_lane_gate_prints_its_verdict_before_its_minutes(self):
        # Loop 3761: 3758's lane gate ran six of six with nothing red and
        # printed no `Lane verdict:`, and docs_guard asks every journal to
        # quote one.
        green=RELEASE_LOG.replace(' ... FAILED', ' ... ok').replace(
            'test result: FAILED. 303 passed; 2 failed', 'test result: ok. 305 passed; 0 failed')
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def run(command, **kwargs):
                calls.append((command, kwargs['env']))
                if '--release' in command:
                    release_ran(kwargs, log=green)
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, git=UNMOVED)
            lines=[str(c.args[0]) for c in printed.call_args_list if c.args]
            self.assertEqual(len(calls), 6)
            verdicts=[l for l in lines if l.startswith('Lane verdict:')]
            self.assertEqual(verdicts, ['Lane verdict: green, nothing red'])
            minutes=next(i for i, l in enumerate(lines) if l.startswith('Gate minutes:'))
            self.assertLess(lines.index(verdicts[0]), minutes)
            # A forgiven red prints its own verdict and never a green one.
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report2', run=run, lane=True, tests=WORD, git=UNMOVED)
            said=[str(c.args[0]) for c in printed.call_args_list if c.args]
            self.assertEqual(len([l for l in said if l.startswith('Lane verdict:')]), 1)
            self.assertNotIn('Lane verdict: green, nothing red', said)

    def test_a_lane_gate_red_on_the_worker_s_own_test_stops_and_names_it(self):
        own=RELEASE_LOG.replace(
            'test diff::floors::the_gate_has_a_bounded_test_width ... ok',
            'test diff::floors::the_gate_has_a_bounded_test_width ... FAILED')
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(own)
            with patch('builtins.print') as printed, self.assertRaises(subprocess.CalledProcessError):
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, git=UNMOVED)
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
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, git=UNMOVED)
            self.assertEqual(gate.step_name(calls[-1][0]), 'release')

    def test_a_lane_gate_refuses_a_tree_with_no_journal(self):
        # Parked 1699: 1639's chain died on a `sed`, the journal was never
        # written, and the gate ran green without it.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with self.assertRaises(ValueError) as refused:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, item=999999,
                          git=UNMOVED)
            self.assertIn('item-999999.md', str(refused.exception))
            self.assertEqual(calls, [])
            # An item whose journal is in the tree gates (1735's is).
            self.assertTrue(gate.journal_of(1735))
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report2', run=run, lane=True, tests=WORD, item=1735,
                          git=UNMOVED)
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

    def test_the_journal_quote_guard_is_forgiven_on_a_lane_and_the_verdict_names_the_remedy(self):
        # Loop 3277 (the forty-fourth pass): the journal guard asks a lane's
        # journal to quote `Lane verdict:` and `Gate minutes:` — lines that
        # exist only once the lane gate has run — so fourteen of twenty
        # first lane gates were red on it alone, and the lane that added the
        # lines did not re-run. The guard is a line the worker cannot write
        # before the gate: forgiven on a lane, held at the booking gate, and
        # the verdict says what to add.
        name = 'a_journal_quotes_its_lane_verdict_and_gate_minutes'
        self.assertIn(name, gate.COMMANDERS_LINES)
        forgiven, line = gate.lane_verdict([f'sim::docs_guard::{name}'])
        self.assertTrue(forgiven, line)
        self.assertIn("red only on the commander's lines", line)
        self.assertIn('quote this gate', line)
        forgiven, line = gate.lane_verdict([f'sim::docs_guard::{name}', 'sim::x::a_red_of_the_worker_s_own'])
        self.assertFalse(forgiven)
        self.assertNotIn('quote this gate', line)
        guard = (Path(__file__).resolve().parents[2] / 'tools/guard.sh').read_text()
        self.assertIn(f'--skip {name}', guard)

    def test_the_loop_line_guard_is_forgiven_on_a_lane_and_the_verdict_says_the_commander_files_it(self):
        # Loop 3328 (the forty-fifth pass): `a_journal_s_loop_line_is_filed_or_says_none`
        # asks that a journal's "for the Loop" line be filed in the parked
        # file, and only the commander files it, at the merge (CLAUDE.md:
        # a worker never edits the queue or the parked file's Loop section
        # by number). 3314's lane gate exited 1 three times on it alone,
        # 3318 filed its own line to get past it, 3322 wrote `none` and
        # moved its note out of the heading. Forgiven on a lane, held at
        # the booking gate, and the verdict says whose line it is.
        name = 'a_journal_s_loop_line_is_filed_or_says_none'
        self.assertIn(name, gate.COMMANDERS_LINES)
        forgiven, line = gate.lane_verdict([f'sim::docs_guard::{name}'])
        self.assertTrue(forgiven, line)
        self.assertIn("red only on the commander's lines", line)
        self.assertIn('the commander files', line)
        forgiven, line = gate.lane_verdict([f'sim::docs_guard::{name}', 'sim::x::a_red_of_the_worker_s_own'])
        self.assertFalse(forgiven)
        self.assertNotIn('the commander files', line)
        guard = (Path(__file__).resolve().parents[2] / 'tools/guard.sh').read_text()
        self.assertIn(f'--skip {name}', guard)

    def test_a_lane_gate_on_a_moved_base_names_the_paths_both_sides_touched_and_refuses_the_done_line(self):
        # Loop 3341 (the forty-fifth pass): 3335 announced its landing on a
        # base four landings behind and 3338 on one two behind; both merges
        # backed out at `ccc merge` on the battery tables every promotion
        # edits, and each cost the lane an update, a resolve and a second
        # gate after it had said landed. The lane gate reads how far the
        # base has moved since the branch left it and which paths both
        # sides touched; with any such path it refuses the done line and
        # names `ccc update`. Made to fail first on 3338's shape.
        def git(args):
            if args[:2] == ['git', 'merge-base']:
                return 'f0rk\n'
            if args[:2] == ['git', 'log']:
                return 'c2729a5 Book 3337\n4a8c010 Book 3338\n'
            if args[:3] == ['git', 'diff', '--name-only'] and args[-1] == 'worktree-replan-pdb':
                return 'crates/rondata/src/diff/battery.rs\ncrates/rondata/src/diff/testkit.rs\ndocs/QUEUE.md\n'
            return 'crates/rondata/src/diff/battery.rs\ncrates/sim/src/city.rs\n'
        def run(command, **kwargs):
            return SimpleNamespace(stdout=git(command), returncode=0)
        commits, overlap = gate.base_moved('worktree-replan-pdb', run=run)
        self.assertEqual(len(commits), 2)
        self.assertEqual(overlap, ['crates/rondata/src/diff/battery.rs'])
        line = gate.base_line('worktree-replan-pdb', commits, overlap)
        self.assertIn('Base moved: 2 commit(s)', line)
        self.assertIn('battery.rs', line)
        self.assertIn('ccc update', line)
        # No overlap: the line says land on this verdict after the update.
        line = gate.base_line('worktree-replan-pdb', commits, [])
        self.assertIn('no path both sides touched', line)
        self.assertIn('ccc update', line)
        # Not moved at all.
        self.assertIn('has not moved', gate.base_line('worktree-replan-pdb', [], []))
        # Through the gate: green steps, a base that moved with an overlap
        # while the gate ran (item 3366 reads it before the first step too,
        # where it had not), and the gate exits red on the line alone.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed, \
                 patch.object(gate, 'base_moved', side_effect=[([], []), (commits, overlap)]), \
                 self.assertRaises(ValueError) as refused:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, git=UNMOVED)
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertIn('Base moved: 2 commit(s)', said)
            self.assertIn('Gate steps: 6 of 6 ran', said)
            self.assertIn('battery.rs', str(refused.exception))

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

    def release_command(self, touched=(), row_only=(), **kw):
        kw.setdefault('isolated', False)  # Retain reference-runner selection assertions.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls=[]
            def run(command, **kwargs):
                calls.append(command)
                if '--release' in command:
                    release_ran(kwargs)
            with patch('builtins.print'), patch.object(gate, 'branch_paths', return_value=list(touched)), \
                    patch.object(gate, 'row_only_edits', return_value=set(row_only)):
                gate.gate(path, report_dir=path/'report', run=run, test_threads=4, git=UNMOVED, **kw)
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
        self.assertEqual(command[:7], ['zsh','tools/memcap.sh','20','cargo','test','--release','--no-fail-fast'])
        self.assertEqual(command[command.index('--'):command.index('--') + 3],
                         ['--', '--test-threads=4', '--exact'])

    def test_a_lane_that_touches_no_sim_path_leaves_the_sim_crate_to_the_booking_gate(self):
        # Loop 3653 (the fifty-seventh pass): every stand-up's lane gate of
        # the fifty-seventh tranche fired Loop 3608's rule — 758 timed tests
        # left to the booking gate, the receipts say — and still ran 16–19
        # release minutes, because the timings file times the data crate
        # alone and the sim crate's five test binaries (the four emulator
        # sweep targets and the lib) ran whole: 682 s at four threads in the
        # pass's replay of 3626's gate, against the data crate's kept set. A
        # branch that touches no path under `crates/sim/` cannot change
        # those binaries, so its lane gate leaves them to the booking gate,
        # which always runs them; the paperwork guards in `sim` run in the
        # guard step (`guard.sh --built`) on every lane as before.
        rows = ['crates/rondata/src/diff.rs', 'crates/rondata/src/diff/battery.rs',
                'crates/rondata/src/diff/greek_gunpowder_pair.rs', 'docs/AI.md', 'docs/RUNS.md']
        command, policy = self.release_command(touched=rows, row_only=rows[:3], lane=True, tests=WORD)
        self.assertEqual(command[command.index('--no-fail-fast'):command.index('--')],
                         ['--no-fail-fast', '--workspace', '--exclude', 'sim'])
        self.assertIn('--skip', command)
        self.assertEqual(policy['lane']['sim_crate'], 'left to the booking gate')
        # A sweep lane touches `sweep/`, which never widens: the timed set is
        # still left, and its own sweeps run.
        command, policy = self.release_command(touched=['crates/sim/src/sweep/x.rs'], lane=True, tests=WORD)
        self.assertNotIn('--exclude', command)
        self.assertIn('--skip', command)
        self.assertEqual(policy['lane']['sim_crate'], 'run')
        # A sim edit widens, and runs everything.
        command, policy = self.release_command(touched=['crates/sim/src/fight.rs'], lane=True, tests=WORD)
        self.assertNotIn('--exclude', command)
        self.assertNotIn('--skip', command)
        self.assertEqual(policy['lane']['sim_crate'], 'run')
        # `--lane --whole` runs every test; the booking gate is never cut.
        command, policy = self.release_command(touched=rows, row_only=rows[:3], lane=True, whole=True)
        self.assertNotIn('--exclude', command)
        self.assertEqual(policy['lane']['sim_crate'], 'run')
        command, policy = self.release_command(touched=rows, scope='abc1234')
        self.assertNotIn('--exclude', command)
        self.assertNotIn('lane', policy)
        # The default runner is the per-binary dispatcher (`isolated_tests.py`):
        # the second replay of 3626's gate ran every sweep with the receipt
        # saying `left to the booking gate`, because only the combined
        # command carried the exclusion. Both paths carry it.
        command, policy = self.release_command(touched=rows, row_only=rows[:3], lane=True, tests=WORD, isolated=True)
        self.assertIn('tools/isolated_tests.py', command)
        self.assertEqual(command[command.index('--exclude-package') + 1], 'sim')
        self.assertEqual(policy['lane']['sim_crate'], 'left to the booking gate')
        command, policy = self.release_command(touched=['crates/sim/src/sweep/x.rs'], lane=True, tests=WORD, isolated=True)
        self.assertNotIn('--exclude-package', command)
        command, policy = self.release_command(touched=rows, scope='abc1234', isolated=True)
        self.assertNotIn('--exclude-package', command)

    def test_a_widened_lane_that_touches_no_sim_path_still_leaves_the_sim_crate(self):
        # Loop 3653's sequel (the fifty-eighth pass): the cut was tied to a
        # lane that keeps the fast suite, and two of the tranche's nine
        # stand-ups widened on one data-crate file each — 3629 on a comment
        # moved in `battery.rs`, 3665 on a debug print in `harness.rs` —
        # and ran the sim crate's five binaries for eleven minutes apiece
        # (release 18.0 and 18.3 against the seven cut stand-ups' 5.6–6.9).
        # The sim crate depends on nothing in the workspace, so a data-crate
        # edit that widens the timed set cannot move its binaries: the cut
        # reads the branch's paths alone, widened or not.
        command, policy = self.release_command(touched=['crates/rondata/src/diff/harness.rs'],
                                               lane=True, tests=WORD)
        self.assertNotIn('--skip', command)
        self.assertEqual(command[command.index('--no-fail-fast'):command.index('--')],
                         ['--no-fail-fast', '--workspace', '--exclude', 'sim'])
        self.assertEqual(policy['lane']['sim_crate'], 'left to the booking gate')
        command, policy = self.release_command(touched=['crates/rondata/src/diff/harness.rs'],
                                               lane=True, tests=WORD, isolated=True)
        self.assertEqual(command[command.index('--exclude-package') + 1], 'sim')
        # What the binaries are built from besides the crate: the workspace
        # manifest, the lock, the toolchain pin and cargo's config keep them.
        for path in ('Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', '.cargo/config.toml'):
            command, policy = self.release_command(touched=[path, 'docs/AI.md'], lane=True, tests=WORD)
            self.assertNotIn('--exclude', command, path)
            self.assertEqual(policy['lane']['sim_crate'], 'run', path)

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
        # Loop 3087 (the fortieth pass): `blind::NEVER` is the residue of
        # functions the documents cite and no trace enters — a sweep that
        # cites one moves it, and no timed `diff::` pin reads it. Seven of
        # the fortieth tranche's eleven scoped booking gates ran every
        # timed test, three of them widened by `blind.rs` alone (441d4f8a,
        # 45b67af5, 88071064): six minutes a gate the chain waited on.
        self.assertEqual(gate.lane_widens(['crates/rondata/src/blind.rs']), [])
        self.assertEqual(gate.lane_widens(['crates/rondata/src/blind.rs', 'crates/sim/src/x.rs']),
                         ['crates/sim/src/x.rs'])
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

    def test_a_pair_stand_up_s_row_list_edits_do_not_widen_the_lane_gate(self):
        # Loop 3608 (the fifty-fifth pass): eight of the fifty-fifth tranche's
        # twenty landings stood an early-age pair up and touched no sim path,
        # and each lane gate ran the whole timed suite (release 16.0–17.7
        # minutes) for a `mod` line in `diff.rs`, a scored triple in
        # `battery.rs`, a ledger row in `ledger.rs` and the new module itself.
        # None of those can move another test's pin; the lane gate reads the
        # hunks and a row-only edit does not widen it. The booking gate's
        # `--scope` keeps the path rule.
        fork = 'f0f0f0f0'
        hunks = {
            'crates/rondata/src/diff.rs': '+mod german_ancient_pair;\n+#[cfg(test)]\n',
            'crates/rondata/src/diff/battery.rs': ("+    // The German Ancient pair's (item 3596): the Germans from an Ancient\n"
                                                   "+    // start, Difficulty 5, Great Sahara.\n+    (7, 12, 5),\n"),
            'crates/rondata/src/ledger.rs': ('+    (\n+        "diff/german_ancient_pair.rs",\n'
                                             '+        "the Ancient-start German pair\'s walks: it compares no field itself (item 3596)",\n'
                                             '+    ),\n'),
            'crates/rondata/src/gamelog.rs': '+    pub fn hits(&self) -> u32 { 0 }\n',
        }
        status = ('A\tcrates/rondata/src/diff/german_ancient_pair.rs\nM\tcrates/rondata/src/diff.rs\n'
                  'M\tcrates/rondata/src/diff/battery.rs\nM\tcrates/rondata/src/ledger.rs\n'
                  'M\tcrates/rondata/src/gamelog.rs\nA\tdocs/journal/2026-10-24-item-3596.md\n')
        def run(command, **kwargs):
            if command[:2] == ['git', 'merge-base']:
                return SimpleNamespace(stdout=fork + '\n')
            if command[:3] == ['git', 'diff', '--name-status']:
                return SimpleNamespace(stdout=status)
            if command[:3] == ['git', 'diff', '-U0']:
                path = command[-1]
                head = f'diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n@@ -1,0 +2,3 @@\n'
                return SimpleNamespace(stdout=head + hunks.get(path, ''))
            raise AssertionError(command)
        self.assertEqual(gate.row_only_edits('worktree-replan-pdb', run=run),
                         {'crates/rondata/src/diff/german_ancient_pair.rs', 'crates/rondata/src/diff.rs',
                          'crates/rondata/src/diff/battery.rs', 'crates/rondata/src/ledger.rs'})
        # A removed line, a `pub use`, or a row of another shape is an edit, not a row.
        hunks['crates/rondata/src/diff.rs'] = '-mod old_pair;\n+mod german_ancient_pair;\n'
        hunks['crates/rondata/src/diff/battery.rs'] = '+    (7, 12, 5),\n+    pub const HELD: usize = 3;\n'
        self.assertEqual(gate.row_only_edits('worktree-replan-pdb', run=run),
                         {'crates/rondata/src/diff/german_ancient_pair.rs', 'crates/rondata/src/ledger.rs'})
        hunks['crates/rondata/src/diff.rs'] = '+pub use german_ancient_pair::walk;\n'
        self.assertNotIn('crates/rondata/src/diff.rs', gate.row_only_edits('worktree-replan-pdb', run=run))
        touched = ['crates/rondata/src/diff/german_ancient_pair.rs', 'crates/rondata/src/diff.rs',
                   'crates/rondata/src/diff/battery.rs', 'crates/rondata/src/ledger.rs',
                   'crates/rondata/src/census.rs', 'docs/AI.md']
        rows = touched[:4]
        self.assertEqual(gate.lane_widens(touched), sorted(rows))
        self.assertEqual(gate.lane_widens(touched, row_only=rows), [])
        self.assertEqual(gate.lane_widens(touched + ['crates/sim/src/fight.rs'], row_only=rows),
                         ['crates/sim/src/fight.rs'])
        command, policy = self.release_command(touched=touched, row_only=rows, lane=True, tests=WORD)
        self.assertIn('--skip', command)
        self.assertEqual(policy['lane']['widened'], [])
        self.assertEqual(policy['lane']['row_only'], sorted(rows))
        command, policy = self.release_command(touched=touched, row_only=rows, scope='abc1234')
        self.assertNotIn('--skip', command)
        self.assertEqual(policy['scope']['widened'], sorted(rows))

    def test_a_pair_stand_up_s_window_line_and_test_kit_rows_do_not_widen_the_lane_gate(self):
        # The fifty-sixth pass (Loop 3608's sequel): every stand-up of the
        # fifty-sixth tranche also added one `|| super::<pair>::<window>(),`
        # line to the coverage driver and two literal rows to the test kit
        # — its `WIDENINGS` tuple and its `AiWord` — so `row_only_edits`
        # left those two paths and zero of twenty lane gates skipped the
        # timed suite (release 16–24 minutes in every journal). Both are
        # row lists: the window line is read by the `coverage::` pins the
        # lane's `--tests` names, the test-kit rows by the fast floors
        # guards every lane gate runs. Eight of the tranche's twenty gates
        # would have kept the fast suite under this reading.
        fork = 'f0f0f0f0'
        hunks = {
            'crates/rondata/src/diff/coverage.rs': '+        || super::french_classical_pair::french_classical_closing_window(),\n',
            'crates/rondata/src/diff/testkit.rs': (
                "+    // **The French at a held Classical age** (item 3607): Great Sahara —\n"
                "+    // whole on its first try, 6106 of 6106, the closing state compared.\n"
                "+    (\n"
                '+        "FRENCH_CLASSICAL_PAIR_WORD",\n'
                "+        crate::diff::french_classical_pair::FRENCH_CLASSICAL_PAIR_WORD,\n"
                '+        Some("french_classical_pair_run1262_s_closing_frame_is_widened_whole"),\n'
                "+        3607,\n"
                "+        Some(crate::diff::french_classical_pair::WIDENING_FRENCH_CLASSICAL_CLOSING),\n"
                "+    ),\n"
                "+    AiWord {\n"
                '+        line: "Late ages",\n'
                '+        map: "GreatSaharaFrenchClassical",\n'
                '+        named: "Great Sahara (French, Classical)",\n'
                "+        word: crate::diff::french_classical_pair::FRENCH_CLASSICAL_PAIR_WORD,\n"
                "+        length: 6_106,\n"
                '+        endpoint: Some("french_classical_pair_run1262_s_closing_state"),\n'
                '+        window: Some("french_classical_closing_window"),\n'
                "+    },\n"
                '+    ("run1262_french_classical_pair_closing_state", 2),\n'),
        }
        status = ('A\tcrates/rondata/src/diff/french_classical_pair.rs\nM\tcrates/rondata/src/diff/coverage.rs\n'
                  'M\tcrates/rondata/src/diff/testkit.rs\n')
        def run(command, **kwargs):
            if command[:2] == ['git', 'merge-base']:
                return SimpleNamespace(stdout=fork + '\n')
            if command[:3] == ['git', 'diff', '--name-status']:
                return SimpleNamespace(stdout=status)
            if command[:3] == ['git', 'diff', '-U0']:
                path = command[-1]
                head = f'diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n@@ -1,0 +2,3 @@\n'
                return SimpleNamespace(stdout=head + hunks.get(path, ''))
            raise AssertionError(command)
        rows = {'crates/rondata/src/diff/french_classical_pair.rs', 'crates/rondata/src/diff/coverage.rs',
                'crates/rondata/src/diff/testkit.rs'}
        self.assertEqual(gate.row_only_edits('worktree-replan-pdb', run=run), rows)
        self.assertEqual(gate.lane_widens(sorted(rows) + ['docs/AI.md'], row_only=sorted(rows)), [])
        # A window swapped, a helper, a `use`, or a field of another name is an edit.
        hunks['crates/rondata/src/diff/coverage.rs'] = ('-        || super::maya_ancient_pair::maya_ancient_frame_4429_window(),\n'
                                                       '+        || super::maya_ancient_pair::maya_ancient_frame_4529_window(),\n')
        self.assertNotIn('crates/rondata/src/diff/coverage.rs', gate.row_only_edits('worktree-replan-pdb', run=run))
        for edit in ('+pub(crate) fn frame(&self) -> u32 { self.frame }\n', '+use super::french_classical_pair::walk;\n',
                     '+        length: LENGTH + 1,\n', '+        stride: 100,\n'):
            hunks['crates/rondata/src/diff/testkit.rs'] = edit
            self.assertNotIn('crates/rondata/src/diff/testkit.rs', gate.row_only_edits('worktree-replan-pdb', run=run), edit)

    def test_a_lane_gate_told_not_to_widen_leaves_the_timed_tests_to_the_booking_gate(self):
        # DECISIONS 83 (the forty-fourth steer, Ramon's word): every lane of
        # the forty-fourth tranche touched a sim path, so every lane gate ran
        # the whole timed suite at four threads — 6 to 26 minutes, median
        # 18.9 — and fifteen of the twenty moved no pin. The AI lane widens;
        # a sweep or chapter lane takes `--no-widen` from its brief and the
        # booking gate holds the pins.
        command, policy = self.release_command(touched=['crates/sim/src/diplomacy.rs'],
                                               lane=True, tests=WORD, no_widen=True)
        self.assertIn('--skip', command)
        self.assertEqual(policy['lane']['widened'], ['crates/sim/src/diplomacy.rs'])
        self.assertTrue(policy['lane']['no_widen'])
        with self.assertRaises(ValueError):
            self.release_command(touched=[], scope='abc1234', no_widen=True)

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
                gate.gate(path, report_dir=path/'report', run=run, lane=True, git=UNMOVED,
                          tests=WORD + ['run683_s_wrod'])
            self.assertIn('run683_s_wrod', str(caught.exception))
            self.assertNotIn('the_gate_has', str(caught.exception))
            self.assertEqual(gate.step_name(calls[-1]), 'release')

    def test_a_lane_gate_refused_for_a_moved_base_says_so_before_the_offline_step(self):
        # Item 3366: 3360's lane gate printed its `Base moved:` refusal as
        # the log's last line, under seven offline cases that failed for
        # reading the same moved tree; it read as a broken tool. A lane gate
        # reads the base before its first step, and when both sides touched
        # a path it says so first and runs nothing.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed, self.assertRaises(ValueError) as refused:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, git=MOVED)
            lines=[str(c.args[0]) for c in printed.call_args_list if c.args]
            self.assertEqual(calls, [])
            moved=[line for line in lines if line.startswith('Base moved:')]
            self.assertEqual(len(moved), 1)
            self.assertIn('battery.rs', moved[0])
            self.assertIn('ccc update', moved[0])
            # Nothing but the pid line comes before it.
            self.assertEqual([l for l in lines[:lines.index(moved[0])] if not l.startswith('Gate pid:')], [])
            self.assertIn('battery.rs', str(refused.exception))
            self.assertIn('before the offline step', str(refused.exception))
        # A base that moved with no path both sides touched still gates.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD,
                          git=git_fixture(commits=['c2729a5 Book 3337'], theirs=['docs/QUEUE.md'],
                                          ours=['crates/sim/src/city.rs']))
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertEqual(len(calls), 6)
            self.assertIn('no path both sides touched', said)

    def test_a_moved_base_whose_only_overlap_is_the_commander_s_paperwork_gates_and_lands(self):
        # Loop 3401 (the forty-seventh pass): 3393's second to fourth lane
        # gates ended on `Base moved:`, each forced a `ccc update` and
        # another full lane gate (~55 minutes), and every update merged
        # clean: the third's and fourth's overlap was docs/PARKED.md alone,
        # whose tail the commander's bookings append to while the lane
        # renames its own reserved slot. The commander's paperwork does not
        # refuse; any other path still does. Made to fail first.
        commits = ['c2729a5 Book 3337', '4a8c010 Book 3338']
        line = gate.base_line('worktree-replan-pdb', commits, ['docs/PARKED.md'])
        self.assertIn("commander's paperwork (docs/PARKED.md)", line)
        self.assertIn('land on this verdict', line)
        self.assertNotIn('gate again', line)
        # (a) Before the offline step: PARKED.md alone, the gate runs all six.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD,
                          git=git_fixture(commits=commits,
                                          theirs=['docs/PARKED.md', 'docs/QUEUE.md'],
                                          ours=['docs/PARKED.md', 'crates/sim/src/city.rs']))
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertEqual(len(calls), 6)
            self.assertIn("commander's paperwork (docs/PARKED.md)", said)
            self.assertIn('land on this verdict', said)
        # The re-read after the steps follows the same rule.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed, \
                 patch.object(gate, 'base_moved', side_effect=[([], []), (commits, ['docs/PARKED.md'])]):
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, git=UNMOVED)
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertIn('Gate steps: 6 of 6 ran', said)
            self.assertIn('land on this verdict', said)
        # (b) PARKED.md beside a real path still refuses, for the real path.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed, self.assertRaises(ValueError) as refused:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD,
                          git=git_fixture(commits=commits,
                                          theirs=['docs/PARKED.md', 'crates/rondata/src/diff/battery.rs'],
                                          ours=['docs/PARKED.md', 'crates/rondata/src/diff/battery.rs']))
            lines=[str(c.args[0]) for c in printed.call_args_list if c.args]
            self.assertEqual(calls, [])
            moved=[line for line in lines if line.startswith('Base moved:')]
            self.assertEqual(len(moved), 1)
            self.assertIn('1 path(s) both sides touched (crates/rondata/src/diff/battery.rs)', moved[0])
            self.assertIn('gate again', moved[0])
            self.assertIn('battery.rs', str(refused.exception))
            self.assertNotIn('PARKED', str(refused.exception))
            self.assertIn('before the offline step', str(refused.exception))

    def test_a_moved_base_whose_overlap_is_row_lists_on_the_base_s_side_gates_and_lands(self):
        # Loop 3694 (the fifty-ninth pass's finding, built by the sixtieth):
        # the residue lanes of the fifty-ninth ran their lane gate three to
        # five times each on `Base moved:`, because every landing touches
        # the rubric pin, `docs/AI.md`, the test-kit rows and the coverage
        # driver's window line, and the sixtieth's two lanes still took a
        # `--keep-conflicts` update and a second gate on the same paths
        # (3704 three gates, 3706 four). A path whose hunks on the base's
        # side are rows a merge keeps both of — a row list's added rows,
        # a document, the census pin and its comment — is no reason to gate
        # again: the lane updates, keeps both sides, re-pins, runs the
        # guards and lands on this verdict; the booking gate is the check on
        # every merge. A removed row, or any other path, still refuses.
        commits = ['c2729a5 Book 3702 and 3709', '4a8c010 Merge worktree-att-3702 (2 commits)']
        rows = {'crates/rondata/src/diff/coverage.rs':
                    '@@ -40,0 +41 @@\n+        || super::mongol_medieval_pair::window(),\n',
                'crates/rondata/src/diff/testkit.rs':
                    '@@ -7519,0 +7520,2 @@\n+    ("mongol_medieval_pair_first_parting", 131),\n'
                    '+    ("run1378_s_closing_state", 131),\n',
                'crates/rondata/src/census.rs':
                    '@@ -544,2 +544,3 @@\n-/// Item 3701: 226 -> 227.\n-pub const RUBRIC_UNIT_TYPES_SEEN: usize = 227;\n'
                    '+/// Item 3701: 226 -> 227.\n+/// Item 3702: 227 -> 228, the Steppe Nomad.\n'
                    '+pub const RUBRIC_UNIT_TYPES_SEEN: usize = 228;\n',
                'docs/AI.md': '@@ -34467 +34467,40 @@\n-## 362. Reserved\n+## 362. The Mongols at Medieval\n+\n+Prose.\n'}
        overlap = sorted(rows)
        line = gate.base_line('worktree-replan-pdb', commits, overlap, row_lists=overlap)
        self.assertIn("row lists on the base's side", line)
        self.assertIn('--keep-conflicts', line)
        self.assertIn('land on this verdict', line)
        self.assertNotIn('gate again', line)
        # (a) Before the offline step: the four alone, the gate runs all six
        # and the receipt names the classes.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD,
                          git=git_fixture(commits=commits, theirs=overlap + ['docs/QUEUE.md'],
                                          ours=overlap + ['crates/sim/src/launch.rs'], theirs_diff=rows))
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertEqual(len(calls), 6)
            self.assertIn("row lists on the base's side", said)
            self.assertIn('land on this verdict', said)
            policy = json.loads((path/'report'/'gate-policy.json').read_text())
            self.assertEqual(policy['lane']['base_overlap'],
                             {'blocking': [], 'row_lists': overlap, 'paperwork': []})
        # (b) A removed row on the base's side is an edit: refused, for that path.
        removed = dict(rows)
        removed['crates/rondata/src/diff/coverage.rs'] = '@@ -41 +40,0 @@\n-        || super::old_pair::window(),\n'
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed, self.assertRaises(ValueError) as refused:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD,
                          git=git_fixture(commits=commits, theirs=overlap, ours=overlap, theirs_diff=removed))
            self.assertEqual(calls, [])
            self.assertIn('coverage.rs', str(refused.exception))
            self.assertNotIn('testkit.rs', str(refused.exception))
            moved=[str(c.args[0]) for c in printed.call_args_list if c.args and str(c.args[0]).startswith('Base moved:')]
            self.assertEqual(len(moved), 1)
            self.assertIn('gate again', moved[0])
        # (c) A sim path beside them still refuses, for the sim path alone.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed, self.assertRaises(ValueError) as refused:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD,
                          git=git_fixture(commits=commits, theirs=overlap + ['crates/sim/src/launch.rs'],
                                          ours=overlap + ['crates/sim/src/launch.rs'], theirs_diff=rows))
            self.assertEqual(calls, [])
            self.assertIn('launch.rs', str(refused.exception))
            self.assertNotIn('coverage.rs', str(refused.exception))

    # Item 3373: `writers::a_field_the_sim_writes_is_read_by_something` timed
    # at 1.08 s, over the lane budget, so `--lane --no-widen` left it to the
    # booking gate, and 3362's booking gate went red on it after a green
    # lane gate. A paperwork guard runs in every lane gate whatever its time.
    PAPERWORK_TIMINGS = ('1.08\twriters::a_field_the_sim_writes_is_read_by_something\n'
                         '1.20\tledger::tests::the_widening_ledger_counts_what_nothing_compares\n'
                         '3.00\tdiff::floors::tests::the_handoff_s_scoreboard_is_the_floors\n'
                         '0.12\twriters::compared_fields_have_writers\n'
                         '2.00\tdiff::harness::tests::run97_s_build_queues_are_the_original_s\n')

    def test_the_paperwork_guards_run_in_every_lane_gate_whatever_their_timing(self):
        skips=gate.lane_skips(self.PAPERWORK_TIMINGS, ['run589'])
        self.assertEqual(skips, ['diff::harness::tests::run97_s_build_queues_are_the_original_s'])
        self.assertEqual(gate.lane_paperwork(self.PAPERWORK_TIMINGS), [
            'diff::floors::tests::the_handoff_s_scoreboard_is_the_floors',
            'ledger::tests::the_widening_ledger_counts_what_nothing_compares',
            'writers::a_field_the_sim_writes_is_read_by_something'])
        for pattern in ('writers::', 'docs_guard', 'ledger::', 'the_handoff_s_'):
            self.assertIn(pattern, gate.PAPERWORK_GUARDS)
        # The suite's own file: the writers guard is over budget there and
        # is kept.
        text=gate.TIMINGS.read_text()
        self.assertNotIn('writers::a_field_the_sim_writes_is_read_by_something', gate.lane_skips(text, []))
        # The lane gate's line counts what the budget kept for the paperwork.
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            def run(command, **kwargs):
                if '--release' in command:
                    release_ran(kwargs, log=RELEASE_LOG.replace(' ... FAILED', ' ... ok'))
            (path/'timings.txt').write_text(self.PAPERWORK_TIMINGS)
            with patch('builtins.print') as printed, patch.object(gate, 'TIMINGS', path/'timings.txt'), \
                 patch.object(gate, 'ROOT', path):
                gate.gate(path, report_dir=path/'report', run=run, lane=True, tests=WORD, git=UNMOVED,
                          isolated=False)
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            policy=json.loads((path/'report/gate-policy.json').read_text())
        suite=next(l for l in said.splitlines() if l.startswith('Lane suite:'))
        self.assertIn('3 over budget kept as paperwork', suite)
        self.assertEqual(policy['lane']['paperwork_kept'], 3)

    def test_the_timings_file_is_the_suite_s(self):
        # The constant names a file that exists, holds the two coverage pins
        # over budget, and parses whole.
        text=gate.TIMINGS.read_text()
        skips=gate.lane_skips(text, [])
        self.assertIn('diff::coverage::every_key_the_dump_prints_is_read_or_pinned', skips)
        self.assertGreater(len(skips), 400)


class WholeLaneGate(unittest.TestCase):
    """Loop 3489 (the fifty-first pass): a lane whose item moves no test —
    a retirement, a tooling item — had no lane gate that ran as briefed:
    `--lane` without `--tests` is refused, so it ran the unflagged booking
    gate, which forgives nothing. `--lane --whole` runs every test, the
    timed ones included, and keeps the lane's forgiveness and verdict."""

    release_command = GateTests.release_command
    lane_run = GateTests.lane_run

    def test_a_whole_lane_gate_runs_every_timed_test(self):
        # Not widened (a tools/ path), and still nothing skipped.
        command, policy = self.release_command(touched=['tools/retire.py'], lane=True, whole=True)
        self.assertNotIn('--skip', command)
        self.assertEqual(policy['lane']['skipped'], 0)
        self.assertTrue(policy['lane']['whole'])
        self.assertEqual(policy['lane']['tests'], [])
        self.assertFalse(policy['verify_caches'])
        # The worded lane gate is unchanged: it skips.
        command, policy = self.release_command(touched=['tools/retire.py'], lane=True, tests=WORD)
        self.assertIn('--skip', command)
        self.assertFalse(policy['lane']['whole'])

    def test_a_whole_lane_gate_forgives_the_commander_s_lines_and_prints_its_verdict(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
            calls, run = self.lane_run(RELEASE_LOG)
            with patch('builtins.print') as printed:
                gate.gate(path, report_dir=path/'report', run=run, lane=True, whole=True, git=UNMOVED)
            said='\n'.join(str(c.args[0]) for c in printed.call_args_list if c.args)
            self.assertEqual([gate.step_name(c) for c, _ in calls],
                             ['offline','clippy','fmt','survey','release','guard'])
            self.assertIn("Lane verdict: red only on the commander's lines (2)", said)
            self.assertIn('Lane suite: every test, the timed ones included (--whole', said)
            release = next(c for c, _ in calls if '--release' in c)
            self.assertNotIn('--skip', release)

    def test_whole_refuses_with_tests_with_no_widen_and_without_lane(self):
        for kw, says in (({'whole': True}, '--whole is a lane gate'),
                         ({'lane': True, 'whole': True, 'tests': WORD}, '--whole runs every test'),
                         ({'lane': True, 'whole': True, 'no_widen': True}, '--whole runs every test'),
                         ({'lane': True}, '--whole')):
            with self.subTest(**kw), tempfile.TemporaryDirectory() as tmp:
                path=Path(tmp);(path/'Data').mkdir();(path/'Data/rules.xml').touch()
                with patch('builtins.print'), self.assertRaises(ValueError) as caught:
                    gate.gate(path, report_dir=path/'report', run=lambda *a, **k: self.fail('ran'),
                              git=UNMOVED, **kw)
                self.assertIn(says, str(caught.exception))
                self.assertIn('Loop 3489', str(caught.exception))

    def test_the_command_line_refuses_whole_before_anything_runs(self):
        script = Path(gate.__file__)
        for argv, says in ((['--whole'], '--whole is a lane gate'),
                           (['--lane', '--whole', '--tests', 'x'], '--whole runs every test')):
            with self.subTest(argv=argv), tempfile.TemporaryDirectory() as tmp:
                done = subprocess.run([sys.executable, str(script), tmp, *argv],
                                      capture_output=True, text=True, timeout=60)
                self.assertEqual(done.returncode, 2, done.stderr)
                self.assertIn(says, done.stderr)
                self.assertNotIn('Gate pid', done.stdout)


if __name__ == '__main__':
    unittest.main()
