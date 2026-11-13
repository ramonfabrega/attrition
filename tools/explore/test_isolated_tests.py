"""Authored processes only: no fixtures, no recursive release gate."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import isolated_tests as runner
import resource_budget as budget
import importlib.util
_spec = importlib.util.spec_from_file_location('release_gate', Path(__file__).resolve().parents[1] / 'release_gate.py')
gate = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gate)


class Admission(unittest.TestCase):
    def test_atomic_capacity_and_release(self):
        with tempfile.TemporaryDirectory() as d:
            first = budget.Lease(d, budget.CPU - 1, budget.MEMORY_MIB - 768)
            other = budget.Lease(d, 2, 128)
            memory = budget.Lease(d, 1, 1024)
            self.assertTrue(first.try_acquire())
            self.assertFalse(other.try_acquire())
            self.assertFalse(memory.try_acquire())
            self.assertEqual(other.fds, [])
            first.close()
            self.assertTrue(other.try_acquire())
            self.assertTrue(memory.try_acquire())
            other.close()
            memory.close()

    def test_inherited_locks_keep_orphan_reservation(self):
        with tempfile.TemporaryDirectory() as d:
            lease = budget.Lease(d, budget.CPU, budget.MEMORY_MIB)
            self.assertTrue(lease.try_acquire())
            with subprocess.Popen([sys.executable, '-c', 'import sys; sys.stdin.read()'],
                                  stdin=subprocess.PIPE, pass_fds=lease.fds) as child:
                lease.close()
                other = budget.Lease(d, 1, 128)
                self.assertFalse(other.try_acquire())
                child.communicate()
            # Loop 2984: once under a load average of 21 the lock was still
            # held a moment after the child had exited (the kernel releases
            # a flock when the last descriptor closes, not when `wait`
            # returns); a bounded wait, never a longer one than two seconds.
            deadline = time.monotonic() + 2.0
            while not other.try_acquire():
                self.assertLess(time.monotonic(), deadline, 'the orphan reservation outlived its child by 2 s')
                time.sleep(0.05)
            other.close()

    def test_worktrees_cannot_silently_use_conflicting_host_policies(self):
        with tempfile.TemporaryDirectory() as d:
            budget.Lease(d, 1, 128)
            path = Path(d) / 'policy.json'
            path.write_text('{"schema":999}')
            with self.assertRaises(ValueError):
                budget.Lease(d, 1, 128)
            self.assertEqual(path.read_text(), '{"schema":999}')

    def test_status_uses_live_locks_not_stale_owner_files(self):
        with tempfile.TemporaryDirectory() as d:
            lease = budget.Lease(d, 2, 1024)
            self.assertTrue(lease.try_acquire())
            lease.describe(12345)
            held = budget.status(d)
            self.assertEqual(len(held), 2)
            self.assertEqual(held[0]['owner']['supervisor_pid'], os.getpid())
            self.assertEqual(held[0]['owner']['child_pid'], 12345)
            lease.close()
            self.assertEqual(budget.status(d), [])

    def test_a_concurrent_probe_is_not_a_holder(self):
        # Two RSS samplers probe the slots at once under the shared mutex; one's
        # probe of a free, never-described slot must not read to the other as a
        # lease with invalid owner metadata (2783's lane gate, exit 125 x3).
        import fcntl
        with tempfile.TemporaryDirectory() as d:
            budget.Lease(d, 1, 128)
            (Path(d) / 'admission.lock').touch()
            fd = os.open(Path(d) / 'cpu-0.lock', os.O_RDWR | os.O_CREAT, 0o600)
            try:
                fcntl.flock(fd, fcntl.LOCK_SH | fcntl.LOCK_NB)
                self.assertEqual(budget.status(d), [])
                budget.sample_rss(os.getpid(), d)
            finally:
                os.close(fd)

    def test_invalid_budget(self):
        with tempfile.TemporaryDirectory() as d:
            for cpu, memory in [(0, 1), (budget.CPU + 1, 1), (1, 0), (1, budget.MEMORY_MIB + 1)]:
                with self.assertRaises(ValueError):
                    budget.Lease(d, cpu, memory)

    def test_sample_failure_refuses_launch(self):
        with tempfile.TemporaryDirectory() as d, patch.object(budget.subprocess, 'check_output', side_effect=RuntimeError('denied')), patch.object(budget.subprocess, 'Popen') as spawn:
            with self.assertRaises(RuntimeError):
                budget.supervise(['never'], directory=d)
            spawn.assert_not_called()

    def test_shared_rss_counts_overlapping_trees_and_orphan_groups_once(self):
        table = {10: (1, 10, 100), 11: (10, 10, 50), 12: (1, 10, 25),
                 20: (1, 20, 200), 21: (20, 99, 30), 50: (1, 50, 900)}
        self.assertEqual(budget.rss_for_roots(table, {10, 11, 20}), 405)
        self.assertEqual(budget.rss_for_roots(table, {10}), 175)

    def test_unreadable_live_owner_is_skipped_for_the_sample(self):
        # Loop 2788, the thirty-eighth pass: a lease is locked before its
        # owner record is written, so a sampler that meets the window read
        # `invalid owner metadata` and ended a gate at exit 125 (2783's lane
        # gate, three tests). One sample's unknown slot is not a reason to
        # fail a run: the slot is skipped and the next sample reads it.
        rows = f"{os.getpid()} 1 1 10\n123 1 1 7\n"
        with patch.object(budget.subprocess, 'check_output', return_value=rows), \
                patch.object(budget, 'status', return_value=[{'slot': 0, 'owner': None},
                                                              {'slot': 1, 'owner': {'supervisor_pid': 'x'}}]):
            self.assertEqual(budget.sample_rss(123, '/unused'), (7, 7))

    def test_combined_host_ceiling_stops_an_otherwise_small_workload(self):
        with tempfile.TemporaryDirectory() as d, patch.object(budget, 'sample_rss', return_value=(1, budget.MEMORY_MIB*1024+1)):
            self.assertEqual(budget.supervise([sys.executable, '-c', 'import time; time.sleep(5)'], directory=d, memory_mib=128, limit_mib=256), 137)

    def test_timeout_failure_and_memory_kill(self):
        with tempfile.TemporaryDirectory() as d, patch.object(budget, 'sample_rss', return_value=(1, 1)):
            self.assertEqual(budget.supervise([sys.executable, '-c', 'raise SystemExit(7)'], directory=d), 7)
            self.assertEqual(budget.supervise([sys.executable, '-c', 'import time; time.sleep(5)'], directory=d, timeout=.02), 124)
        with tempfile.TemporaryDirectory() as d, patch.object(budget, 'sample_rss', return_value=(2**30, 2**30)):
            self.assertEqual(budget.supervise([sys.executable, '-c', 'import time; time.sleep(5)'], directory=d, memory_mib=128), 137)

    def test_six_independent_suites_share_cpu_capacity(self):
        with tempfile.TemporaryDirectory() as d:
            processes = []
            code = "import time,sys; print(time.monotonic(), flush=True); time.sleep(.3); print(time.monotonic(), flush=True)"
            for _ in range(6):
                processes.append(subprocess.Popen(
                    [sys.executable, str(Path(budget.__file__)), '--directory', d,
                     '--cpu', str(budget.CPU // 4), '--memory-mib', '128', '--', sys.executable, '-c', code],
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True))
            events = []
            timings = []
            for process in processes:
                out, err = process.communicate(timeout=15)
                self.assertEqual(process.returncode, 0, err)
                receipt = next(line for line in err.splitlines() if line.startswith('resource budget timing: '))
                timing = json.loads(receipt.removeprefix('resource budget timing: '))
                self.assertTrue(timing['admitted'])
                self.assertGreaterEqual(timing['wait_seconds'], 0)
                self.assertGreater(timing['run_seconds'], 0)
                timings.append(timing)
                begin, end = map(float, out.splitlines())
                events.extend([(begin, 1), (end, -1)])
            active = peak = 0
            for _, delta in sorted(events):
                active += delta
                peak = max(peak, active)
            self.assertLessEqual(peak, 4)
            self.assertGreater(peak, 1)
            self.assertEqual(active, 0)
            self.assertEqual(len(timings), 6)

    def test_the_kill_switch_admits_under_a_full_host_and_says_so(self):
        # DECISIONS 75: RON_NO_ADMISSION (or the marker file) skips the lease and
        # keeps the ceilings, so a tranche is reversed by one touch, never a revert.
        with tempfile.TemporaryDirectory() as d:
            held = budget.Lease(d, budget.CPU, budget.MEMORY_MIB)
            self.assertTrue(held.try_acquire())
            env = dict(os.environ, RON_NO_ADMISSION='1')
            done = subprocess.run([sys.executable, str(Path(budget.__file__)), '--directory', d, '--cpu', '1',
                                   '--memory-mib', '128', '--', sys.executable, '-c', 'pass'],
                                  capture_output=True, text=True, timeout=15, env=env)
            held.close()
            self.assertEqual(done.returncode, 0, done.stderr)
            receipt = next(line for line in done.stderr.splitlines() if line.startswith('resource budget timing: '))
            self.assertEqual(json.loads(receipt.removeprefix('resource budget timing: '))['admission'], 'off')
            self.assertIn('admission off', done.stderr)

    def test_cancellation_reaps_workload_and_releases_shared_tokens(self):
        with tempfile.TemporaryDirectory() as d:
            receipt = Path(d) / 'pid'
            code = "import os,time; from pathlib import Path; Path(" + repr(str(receipt)) + ").write_text(str(os.getpid())); time.sleep(30)"
            with subprocess.Popen([sys.executable, str(Path(budget.__file__)), '--directory', d,
                                   '--cpu', '8', '--', sys.executable, '-c', code],
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE) as child:
                deadline = time.monotonic() + 5
                while not receipt.exists() and time.monotonic() < deadline:
                    time.sleep(.02)
                self.assertTrue(receipt.exists())
                workload = int(receipt.read_text())
                child.terminate()
                child.communicate(timeout=5)
                self.assertEqual(child.returncode, 130)
                with self.assertRaises(ProcessLookupError):
                    os.kill(workload, 0)
            lease = budget.Lease(d, 16, 32768)
            self.assertTrue(lease.try_acquire())
            lease.close()


class Manifest(unittest.TestCase):
    def test_target_budgets_preserve_requested_cpu_width(self):
        self.assertEqual(runner.target_budget({'kind': ['lib'], 'name': 'sim'}, 4), (4, 4096))
        # Loop 3363: the sim lib at eight threads peaked at 4218-4262 MiB on
        # three whole gates (fc206a90, 163e8d89 twice) and the budget stopped
        # it with no test red; the limit follows the thread count.
        self.assertEqual(runner.target_budget({'kind': ['lib'], 'name': 'sim'}, 8), (8, 6144))
        self.assertEqual(runner.target_budget({'kind': ['lib'], 'name': 'fixed'}, 8), (8, 512))
        self.assertEqual(runner.target_budget({'kind': ['test'], 'name': 'new'}, 4), (8, 32768))
        # Loop 3777: the emu cache keeps over-cap rows compressed, and under
        # verify at eight threads `sweep_u_z` read 3,859 MiB sampled, 6 %
        # under 4,096; the four sweep targets reserve the lib's 6,144.
        for family in ('a_f', 'g_o', 'p_t', 'u_z'):
            self.assertEqual(runner.target_budget({'kind': ['test'], 'name': f'sweep_{family}'}, 4), (4, 6144))
            self.assertEqual(runner.target_budget({'kind': ['test'], 'name': f'sweep_{family}'}, 8), (8, 6144))

    def test_discovery_is_strict(self):
        for text in ['a: test\na: test\n', 'unexpected\n', 'bench: benchmark\n']:
            with patch.object(runner.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, text)):
                with self.assertRaises(ValueError):
                    runner.discover('fake')

    def test_timing_receipt_is_separate_from_test_cpu(self):
        with tempfile.TemporaryDirectory() as d:
            log = Path(d)/'log'
            receipt = dict(wait_seconds=2.5, run_seconds=1.2, admitted=True)
            log.write_text('resource budget timing: ' + json.dumps(receipt) + '\n')
            self.assertEqual(runner.resource_timing(log), receipt)
            log.write_text('old log without receipt\n')
            self.assertIsNone(runner.resource_timing(log))

    def test_unknown_test_gets_the_old_lane_budget_not_the_host(self):
        self.assertEqual(runner.reservation('new', {}), 8192)
        self.assertEqual((runner.UNKNOWN_MIB, runner.UNKNOWN_LIMIT_MIB), (8192, 32768))
        self.assertLess(runner.UNKNOWN_LIMIT_MIB, budget.MEMORY_MIB)
        self.assertEqual(runner.reservation('small', {'small': {'rss_bytes': 1}}), 256)

    def test_large_failure_log_is_read_with_bounded_memory(self):
        with tempfile.TemporaryDirectory() as d:
            log = Path(d) / 'log'
            log.write_text('x' * 200000 + '\n' +
                           'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out;\n' +
                           '1.00 real 2.00 user 3.00 sys\n123 maximum resident set size\n')
            self.assertEqual(runner.read_result(log), (1, (2., 3.), 123))
            log.write_text('test result: ok. 0 passed; 0 failed;\n')
            self.assertEqual(runner.read_result(log), (0, None, None))

    def test_zero_test_success_is_not_a_pass(self):
        self.assertFalse(runner.ONE.search('test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out;'))
        self.assertFalse(runner.ONE.search('test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;'))

    def test_infrastructure_failures_keep_their_exit_category(self):
        for code in (0, 124, 125, 137):
            with self.subTest(code=code), tempfile.TemporaryDirectory() as d:
                row = dict(name='only', success=False, exit=code, user=0, system=0)
                with patch.object(runner, 'binary_stamp', return_value=(0,)), patch.object(runner, 'discover', side_effect=[['only'], []]), patch.object(runner, 'run_one', return_value=row):
                    self.assertEqual(runner.isolated('fake', Path(d)/'out', Path(d), 1), code or 125)

    def test_exact_skips_preserve_new_tests_and_do_not_skip_prefix_matches(self):
        with tempfile.TemporaryDirectory() as d:
            out = Path(d)/'out'
            seen = []
            def run(binary, name, *args):
                seen.append(name)
                return dict(name=name, success=True, exit=0, user=1, system=0)
            with patch.object(runner, 'binary_stamp', return_value=(0,)), patch.object(runner, 'discover', side_effect=[['slow', 'slow_new', 'new', 'ignored'], ['ignored']]), patch.object(runner, 'run_one', side_effect=run):
                self.assertEqual(runner.isolated('fake', out, Path(d), 2, skips=['slow', 'absent']), 0)
            self.assertCountEqual(seen, ['slow_new', 'new'])
            manifest = json.loads((out/'manifest.json').read_text())
            self.assertEqual(manifest['skipped'], ['slow'])
            self.assertEqual(manifest['ignored'], ['ignored'])
            self.assertEqual(json.loads((out/'summary.json').read_text())['filtered'], 1)

    def test_every_test_once_and_no_fail_fast(self):
        with tempfile.TemporaryDirectory() as d:
            out = Path(d) / 'out'
            seen = []
            def run(binary, name, *args):
                seen.append(name)
                return dict(name=name, success=name != 'bad', exit=0 if name != 'bad' else 101, user=1, system=0)
            with patch.object(runner, 'binary_stamp', return_value=(0,)), patch.object(runner, 'discover', side_effect=[['ok', 'bad', 'ignored', 'new'], ['ignored']]), patch.object(runner, 'run_one', side_effect=run):
                self.assertEqual(runner.isolated('fake', out, Path(d), 2), 101)
            self.assertCountEqual(seen, ['ok', 'bad', 'new'])
            self.assertTrue(json.loads((out/'summary.json').read_text())['complete'])
            with self.assertRaises(FileExistsError):
                runner.isolated('fake', out, Path(d), 2)


    def test_a_test_the_budget_stopped_names_its_stale_profile_row(self):
        # Loop 3167 (the booking gate on e183b2e5, the forty-second
        # tranche's one red): a test reserved from a 346 MB profile row,
        # grew to 849 MiB with the battery, was stopped at a 768 MiB limit
        # and reported as 137 and nothing else; the commander read the
        # budget's stderr by hand. The row is named stale, with the three
        # numbers and the remedy. Red first: no such line, no such fields.
        with tempfile.TemporaryDirectory() as d:
            log = Path(d) / 'one.log'
            log.write_text('running 1 test\nresource budget: RSS 849 MiB / limit 768 MiB; combined 900 MiB / host limit 65536 MiB — stopping owned workload\n')
            self.assertEqual(runner.stopped_at(log), (849, 768))
            log.write_text('test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n')
            self.assertEqual(runner.stopped_at(log), (None, None))
            profile = Path(d) / 'profile.json'
            profile.write_text(json.dumps({'tree': 'abc', 'tests': {'only': {'elapsed': 0.5, 'rss_bytes': 346603520}}}))
            row = dict(name='only', success=False, exit=137, user=0, system=0, stopped_mib=849, limit_mib=768, reservation_mib=512)
            out = io.StringIO()
            with patch.object(runner, 'PROFILE', profile), patch.object(runner, 'binary_stamp', return_value=(0,)), \
                    patch.object(runner, 'discover', side_effect=[['only'], []]), patch.object(runner, 'run_one', return_value=row), \
                    contextlib.redirect_stdout(out):
                self.assertEqual(runner.isolated('fake', Path(d)/'out', Path(d), 1), 137)
            said = out.getvalue()
            self.assertIn('stale profile row: only stopped at 849 MiB against its 768 MiB limit, reserved 512 MiB from a 331 MiB row', said)
            self.assertIn(f'({profile.name}, tree abc): raise its rss_bytes to the measured RSS or re-profile; alone it may pass', said)


class Width(unittest.TestCase):
    """Item 4072 (Loop 4042): on `dc1e1fd39`'s whole gate the coverage pin
    walked 436 windows one at a time for 2,519 s while every other library
    test was done by second 1,070, and seven of eight slots stood idle for
    the last 1,449. A profile row's `cpu: k` makes a test k-wide: the pool
    reserves k slots for it and tells it so (`RON_TEST_CPU`), and the test
    walks k-wide. Red first: the runner passed every test one slot."""

    WORK = {'wide': 0.8, 'a': 0.1, 'b': 0.1, 'c': 0.1, 'd': 0.1}

    def schedule(self, profile, slots=4):
        """The runner over a fake test whose wall is its work over the width
        it was given, as the coverage walk's is; `(starts, wall, rows)`."""
        spans, lock = [], __import__('threading').Lock()
        def run(binary, name, number, output, directory, memory, cancelled, known=True, cpu=1):
            began = time.monotonic()
            time.sleep(self.WORK[name] / cpu)
            with lock:
                spans.append((name, cpu, began, time.monotonic()))
            return dict(name=name, success=True, exit=0, user=0, system=0)
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'profile.json'
            path.write_text(json.dumps({'tree': 'fixture', 'tests': profile}))
            with patch.object(runner, 'PROFILE', path), patch.object(runner, 'binary_stamp', return_value=(0,)), \
                    patch.object(runner, 'discover', side_effect=[list(self.WORK), []]), \
                    patch.object(runner, 'run_one', side_effect=run), contextlib.redirect_stdout(io.StringIO()):
                began = time.monotonic()
                self.assertEqual(runner.isolated('fake', Path(d) / 'out', Path(d), slots), 0)
                wall = time.monotonic() - began
        return spans, wall

    def test_a_wide_test_fills_the_slots_that_stood_idle(self):
        profile = {name: {'elapsed': work * 100, 'rss_bytes': 1} for name, work in self.WORK.items()}
        profile['wide']['cpu'] = 4
        spans, wall = self.schedule(profile)
        widths = {name: cpu for name, cpu, _, _ in spans}
        self.assertEqual(widths, {'wide': 4, 'a': 1, 'b': 1, 'c': 1, 'd': 1})
        # The pool never holds more slot-units than it has.
        edges = sorted([(t0, cpu) for _, cpu, t0, _ in spans] + [(t1, -cpu) for _, cpu, _, t1 in spans])
        held = peak = 0
        for _, delta in edges:
            held += delta
            peak = max(peak, held)
        self.assertLessEqual(peak, 4)
        # The old policy's wall is the wide test's whole work on one slot
        # (0.8 s, three slots idle for 0.7 of it); filled, it is its work
        # over four and the narrow tests behind it (0.2 + 0.1).
        occupied = sum(cpu * (t1 - t0) for _, cpu, t0, t1 in spans) / (4 * wall)
        self.assertLess(wall, 0.6, spans)
        self.assertGreater(occupied, 0.6, (occupied, spans))

    def test_a_row_without_cpu_is_one_slot_and_a_width_is_capped_at_the_pool(self):
        self.assertEqual(runner.width('x', {}, 8), 1)
        self.assertEqual(runner.width('x', {'x': {'rss_bytes': 1}}, 8), 1)
        self.assertEqual(runner.width('x', {'x': {'cpu': 4}}, 8), 4)
        self.assertEqual(runner.width('x', {'x': {'cpu': 16}}, 8), 8)
        self.assertEqual(runner.width('x', {'x': {'cpu': 0}}, 8), 1)

    def test_the_test_is_told_its_width_and_the_budget_reserves_it(self):
        seen = {}
        class Child:
            returncode = 0
            def __init__(self, command, stdout, stderr, env):
                seen.update(command=command, env=env)
            def __enter__(self):
                return self
            def __exit__(self, *exc):
                return False
            def poll(self):
                return 0
        with tempfile.TemporaryDirectory() as d:
            out = Path(d)
            (out / 'tests').mkdir()
            with patch.object(runner.subprocess, 'Popen', Child), patch.object(runner, 'binary_stamp', return_value=(0,)):
                row = runner.run_one('fake', 'wide', 1, out, out, 2048, lambda: False, True, 4)
        command = seen['command']
        self.assertEqual(command[command.index('--cpu') + 1], '4')
        self.assertEqual(seen['env']['RON_TEST_CPU'], '4')
        self.assertEqual(row['cpu_slots'], 4)


class Lanes(unittest.TestCase):
    """Loop 4042's remainder: on `0e28830d6`'s whole gate the release step
    was 1,711 s — the build 2 s, then the nine non-`rondata` binaries one
    after another for 454 s (the four sweep targets 390 s, the `sim` library
    64 s; Cargo's order puts them before the library, not after), then the
    isolated library 1,250 s with its eight slots 99.3 % held, then 5 s of
    small binaries and the doctests. The binaries run beside the library
    instead: a second lane, one binary at a time as before, which starts
    once the library's head test is running (so the head, the eight-wide
    coverage walk, is admitted first), with its output kept to a log per
    binary and printed whole after the library's own block. Red first: the
    booked runner ran every binary in Cargo's order, one after another."""

    SIDE = 0.2
    LIBRARY = 1.2

    def targets(self, d, exits=None, verdicts=None, sleeps=None):
        """Real executables in Cargo's order: a fake libtest binary prints
        one verdict and stamps its span; the library is a file only (the
        isolated runner is the fake)."""
        exits, verdicts, sleeps = exits or {}, verdicts or {}, sleeps or {}
        rows = []
        order = [('rondata', 'bin'), ('sweep_a_f', 'test'), ('sim', 'lib'), ('sweep_u_z', 'test'),
                 ('rondata', 'lib'), ('client', 'bin')]
        for k, (name, kind) in enumerate(order):
            exe = Path(d) / f'{name}-{kind}{k}'
            key = f'{name}-{kind}'
            if key == 'rondata-lib':
                exe.write_text('library\n')
            else:
                passed = 0 if key == 'rondata-bin' else 2
                verdict = (f'test result: ok. {passed} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; '
                           'finished in 0.40s')
                exe.write_text(
                    f'#!{sys.executable}\nimport sys, time\n'
                    f'open({str(Path(d) / "spans")!r}, "a").write("{key} start %r\\n" % time.time())\n'
                    f'time.sleep({sleeps.get(key, self.SIDE)!r})\n'
                    f'print("\\nrunning {passed} tests")\n'
                    + ''.join(f'print({verdict!r})\n' for _ in range(verdicts.get(key, 1)))
                    + f'open({str(Path(d) / "spans")!r}, "a").write("{key} end %r\\n" % time.time())\n'
                    f'sys.exit({exits.get(key, 0)})\n')
            exe.chmod(0o755)
            rows.append((str(exe), {'src_path': f'/w/{name}/{kind}.rs', 'name': name, 'kind': [kind]}))
        return rows

    def release(self, d, targets, library_exit=0, beside=True):
        """`beside`: the fake library runs until every other binary has
        started (at most 15 s), so the claim does not rest on the host's
        speed: the booked runner, which starts nothing during the library,
        waits out the cap and is red."""
        spans = Path(d) / 'spans'
        others = len(targets) - 1
        real_run = subprocess.run
        doc = []

        def run(command, *args, **kwargs):
            if 'cargo' in command and '--no-run' in command:
                return subprocess.CompletedProcess(command, 0, '', '')
            if 'cargo' in command and '--doc' in command:
                doc.append(time.time())
                return subprocess.CompletedProcess(command, 0)
            return real_run(command, *args, **kwargs)

        def isolated(binary, output, directory, slots=8, skips=(), **kw):
            with spans.open('a') as sink:
                sink.write(f'rondata-lib start {time.time()!r}\n')
            if 'started' in kw:
                time.sleep(0.05)
                kw['started'].set()
            deadline = time.monotonic() + (15 if beside else self.LIBRARY)
            while time.monotonic() < deadline:
                if beside and spans.read_text().count(' start ') - 1 >= others:
                    break
                time.sleep(0.05)
            time.sleep(0.2)
            print('test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.20s')
            with spans.open('a') as sink:
                sink.write(f'rondata-lib end {time.time()!r}\n')
            return library_exit

        out = io.StringIO()
        with patch.object(runner.subprocess, 'run', side_effect=run), \
                patch.object(runner.subprocess, 'check_output', return_value='abc\n'), \
                patch.object(runner, 'kept_targets', return_value=(targets, [])), \
                patch.object(runner, 'isolated', side_effect=isolated), contextlib.redirect_stdout(out):
            began = time.monotonic()
            rc = runner.release(Path(d) / 'out', Path(d) / 'budget', 8, 8)
            wall = time.monotonic() - began
        seen = {}
        for line in spans.read_text().splitlines():
            key, edge, when = line.split()
            seen.setdefault(key, {})[edge] = float(when)
        return rc, seen, out.getvalue(), doc, wall

    def summary(self, d, text):
        log = Path(d) / 'release.log'
        log.write_text(text)
        return [(row['name'], row['result'], row['passed'])
                for row in gate.summarize_tests(log, release_completed=True)['binaries']]

    def test_the_binaries_run_beside_the_library_and_each_verdict_is_read(self):
        with tempfile.TemporaryDirectory() as d:
            rc, seen, text, doc, wall = self.release(d, self.targets(d))
            self.assertEqual(rc, 0, text)
            library = seen['rondata-lib']
            side = {k: v for k, v in seen.items() if k != 'rondata-lib'}
            self.assertEqual(sorted(side), ['client-bin', 'rondata-bin', 'sim-lib', 'sweep_a_f-test', 'sweep_u_z-test'])
            # The library's head is admitted first; every other binary starts
            # while the library is still running, one at a time.
            for key, span in side.items():
                self.assertGreater(span['start'], library['start'], (key, seen))
                self.assertLess(span['start'], library['end'], (key, seen))
            ordered = sorted(side.values(), key=lambda s: s['start'])
            for before, after in zip(ordered, ordered[1:]):
                self.assertLessEqual(before['end'], after['start'], seen)
            # The doctests still come last, after both lanes.
            self.assertEqual(len(doc), 1)
            self.assertGreater(doc[0], max(s['end'] for s in seen.values()))
            # The gate's reader keys every verdict to its own binary: the
            # library's block is never split by another binary's lines.
            self.assertEqual(sorted(self.summary(d, text)), sorted([
                ('rondata', 'ok', 0), ('sweep_a_f', 'ok', 2), ('sim', 'ok', 2), ('sweep_u_z', 'ok', 2),
                ('rondata', 'ok', 3), ('client', 'ok', 2)]))
            # Every binary has its exit, in Cargo's order, and the lanes'
            # spans are kept for the booking gate's measurement.
            out = Path(d) / 'out'
            self.assertEqual(json.loads((out / 'exit-codes.json').read_text()), [0] * 7)
            lanes = json.loads((out / 'lanes.json').read_text())
            self.assertEqual(len(lanes['side']), 5)
            self.assertIn('Release lanes:', text)

    def test_a_red_binary_beside_the_library_is_still_red_and_nothing_is_skipped(self):
        with tempfile.TemporaryDirectory() as d:
            rc, seen, text, _, _ = self.release(d, self.targets(d, exits={'sweep_a_f-test': 101}))
            self.assertEqual(rc, 101)
            self.assertEqual(len(seen), 6, seen)
            self.assertEqual(json.loads((Path(d) / 'out' / 'exit-codes.json').read_text()), [0, 101, 0, 0, 0, 0, 0])
        with tempfile.TemporaryDirectory() as d:
            rc, seen, _, _, _ = self.release(d, self.targets(d), library_exit=101)
            self.assertEqual(rc, 101)
            self.assertEqual(len(seen), 6, seen)

    def test_a_binary_that_exits_zero_with_no_verdict_is_not_a_pass(self):
        with tempfile.TemporaryDirectory() as d:
            rc, _, text, _, _ = self.release(d, self.targets(d, verdicts={'sim-lib': 0}))
            self.assertEqual(rc, 125, text)
            self.assertIn('printed 0 verdicts', text)

    def test_a_cancelled_library_stops_the_binary_beside_it(self):
        with tempfile.TemporaryDirectory() as d:
            targets = self.targets(d, sleeps={'rondata-bin': 30})
            rc, seen, _, doc, wall = self.release(d, targets, library_exit=130, beside=False)
            self.assertEqual(rc, 130)
            self.assertEqual(doc, [])
            self.assertLess(wall, 10, seen)
            self.assertNotIn('end', seen.get('rondata-bin', {}), seen)
            self.assertNotIn('client-bin', seen)

    def test_a_library_that_changed_after_discovery_raises_and_leaves_no_lane_waiting(self):
        import signal
        def late(_sig, _frame):
            raise TimeoutError('the release step hung on its lane')
        with tempfile.TemporaryDirectory() as d:
            targets = self.targets(d)
            library = next(exe for exe, t in targets if t['name'] == 'rondata' and 'lib' in t['kind'])
            real, calls = runner.binary_stamp, []
            def stamp(path):
                if str(path) == library:
                    calls.append(path)
                    return (len(calls),)
                return real(path)
            previous = signal.signal(signal.SIGALRM, late)
            signal.alarm(20)
            try:
                with patch.object(runner, 'binary_stamp', side_effect=stamp):
                    with self.assertRaisesRegex(ValueError, 'changed after Cargo discovery'):
                        self.release(d, targets)
            finally:
                signal.alarm(0)
                signal.signal(signal.SIGALRM, previous)

    def test_the_lane_beside_waits_for_the_library_s_head_to_run(self):
        # `started` is set once the head test's log shows libtest's banner
        # (the budget admitted it), before the head is done; a head that
        # never prints one sets it when it completes.
        import threading
        for banner in (True, False):
            with self.subTest(banner=banner), tempfile.TemporaryDirectory() as d:
                times = {}
                def run(binary, name, number, output, directory, memory, cancelled, known=True, cpu=1):
                    if name == 'head':
                        times['head_start'] = time.monotonic()
                        if banner:
                            (output / 'tests' / f'{number:04d}.log').write_text('\nrunning 1 test\n')
                        time.sleep(0.8)
                        times['head_end'] = time.monotonic()
                    return dict(name=name, success=True, exit=0, user=0, system=0)
                started = threading.Event()
                def watch():
                    started.wait()
                    times['started'] = time.monotonic()
                watcher = threading.Thread(target=watch)
                watcher.start()
                path = Path(d) / 'profile.json'
                path.write_text(json.dumps({'tree': 'fixture', 'tests': {'head': {'elapsed': 9, 'rss_bytes': 1},
                                                                         'tail': {'elapsed': 1, 'rss_bytes': 1}}}))
                with patch.object(runner, 'PROFILE', path), patch.object(runner, 'binary_stamp', return_value=(0,)), \
                        patch.object(runner, 'discover', side_effect=[['tail', 'head'], []]), \
                        patch.object(runner, 'run_one', side_effect=run), contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(runner.isolated('fake', Path(d) / 'out', Path(d), 2, started=started), 0)
                watcher.join(5)
                self.assertGreaterEqual(times['started'], times['head_start'])
                if banner:
                    self.assertLess(times['started'], times['head_end'] - 0.3, times)
                else:
                    self.assertGreaterEqual(times['started'], times['head_end'], times)


if __name__ == '__main__':
    unittest.main()


FAKE_BINARY = r"""#!/usr/bin/env python3
# A test binary's shape, for the group unit: `--exact a b c --test-threads=1`
# prints libtest's lines. A test named `fails` fails, `dies` ends the
# process with 9 before its verdict, and `dropped` is never run.
import os, sys, time
names = [a for a in sys.argv[1:] if not a.startswith('--')]
print(f'\nrunning {len(names)} tests')
passed = failed = 0
for name in names:
    if name == 'dropped':
        continue
    sys.stdout.write(f'test {name} ... ')
    sys.stdout.flush()
    time.sleep(0.05)
    if name == 'dies':
        os._exit(9)
    if name == 'fails':
        failed += 1
        print('FAILED')
    else:
        passed += 1
        print('ok')
    with open(os.environ['FAKE_RAN'], 'a') as ran:
        ran.write(f"{name} {os.getpid()} {os.environ.get('RON_SUITE_GROUP')} {os.environ.get('RON_SUITE_GROUP_TESTS')}\n")
    if os.environ.get('FAKE_SHARE'):
        with open(os.environ['FAKE_SHARE'], 'a') as share:
            share.write(f"{name} {os.environ.get('RON_SHARE_WALKS')}\n")
print(f"\ntest result: {'FAILED' if failed else 'ok'}. {passed} passed; {failed} failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.10s\n")
sys.exit(101 if failed else 0)
"""


class Groups(unittest.TestCase):
    """Item 4216 (DECISIONS 132 §11): a group's selected tests run in one
    process. Red first: the runner knew no unit but one test a process, so
    `group_units`, `run_group` and the rows' `group` did not exist."""

    def run_library(self, names, groups, ignored=(), peaks=None):
        """The runner over the fake binary; `(exit, rows by name, the
        lines each test's process wrote, what the runner printed)`."""
        with tempfile.TemporaryDirectory() as d:
            d = Path(d)
            binary = d / 'fake-binary'
            binary.write_text(FAKE_BINARY)
            binary.chmod(0o755)
            profile = d / 'profile.json'
            profile.write_text(json.dumps({'tree': 'fixture', 'tests': {}}))
            said = io.StringIO()
            with patch.dict(os.environ, {'FAKE_RAN': str(d / 'ran')}), patch.object(runner, 'PROFILE', profile), \
                    patch.object(runner, 'discover', side_effect=[list(names), list(ignored)]), \
                    patch.object(runner, 'ONE', runner.re.compile(r'^test result: ok\. 1 passed; 0 failed;', runner.re.M)), \
                    contextlib.redirect_stdout(said):
                code = runner.isolated(binary, d / 'out', d / 'budget', 4, groups=groups, peaks=peaks)
            rows = {}
            for line in (d / 'out' / 'measurements.jsonl').read_text().splitlines():
                row = json.loads(line)
                self.assertNotIn(row['name'], rows, 'a test has one row')
                rows[row['name']] = row
            ran = [line.split() for line in (d / 'ran').read_text().splitlines()] if (d / 'ran').exists() else []
            summary = json.loads((d / 'out' / 'summary.json').read_text())
            manifest = json.loads((d / 'out' / 'manifest.json').read_text())
        return code, rows, ran, said.getvalue(), summary, manifest

    def test_a_group_s_tests_run_in_one_process_and_each_has_its_own_row(self):
        code, rows, ran, said, summary, manifest = self.run_library(
            ['a', 'b', 'c', 'lone', 'd'], {'game:one': ['a', 'c', 'd'], 'game:two': ['lone', 'absent']})
        self.assertEqual(code, 0, said)
        self.assertEqual(set(rows), {'a', 'b', 'c', 'lone', 'd'})
        self.assertTrue(all(row['success'] for row in rows.values()), rows)
        pids = {name: pid for name, pid, _, _ in ran}
        # One process for the group's three, in the order listed; the test no
        # group names and the group with one selected member each alone.
        self.assertEqual(len({pids['a'], pids['c'], pids['d']}), 1)
        self.assertEqual(len(set(pids.values())), 3)
        self.assertEqual([name for name, pid, _, _ in ran if pid == pids['a']], ['a', 'c', 'd'])
        self.assertEqual({(key, size) for name, _, key, size in ran if name in 'acd'}, {('game:one', '3')})
        self.assertEqual({(key, size) for name, _, key, size in ran if name in ('b', 'lone')}, {('None', 'None')})
        self.assertEqual([rows[n].get('group') for n in ('a', 'b', 'c', 'lone', 'd')],
                         ['game:one', None, 'game:one', None, 'game:one'])
        self.assertEqual(manifest['groups'], {'game:one': ['a', 'c', 'd']})
        self.assertEqual((summary['groups'], summary['grouped_tests'], summary['processes'], summary['not_run']),
                         (1, 3, 3, 0))
        self.assertIn('Suite: 3 processes for 5 tests: 1 groups hold 3, 2 ran alone', said)
        # The group's CPU is shared out, so the suite's sum counts it once.
        self.assertAlmostEqual(sum(rows[n]['user'] + rows[n]['system'] for n in 'acd'),
                               sum(rows[n]['user'] + rows[n]['system'] for n in 'acd'))
        self.assertEqual(sum(rows[n]['resource_timing'] is not None for n in 'acd'), 1)

    def test_a_failing_test_is_its_own_red_and_the_rest_of_its_group_still_run(self):
        code, rows, ran, said, summary, _ = self.run_library(
            ['a', 'fails', 'c'], {'g': ['a', 'fails', 'c']})
        self.assertEqual(code, 101, said)
        self.assertEqual({n: (r['success'], r['exit'], r['not_run']) for n, r in rows.items()},
                         {'a': (True, 0, False), 'fails': (False, 101, False), 'c': (True, 0, False)})
        self.assertIn('test fails ... FAILED', said)
        self.assertIn('test c ... ok', said)
        self.assertNotIn('not run', said)

    def test_a_group_that_dies_reports_what_it_did_not_reach_as_not_run_never_green(self):
        code, rows, ran, said, summary, _ = self.run_library(
            ['a', 'dies', 'c', 'e'], {'g': ['a', 'dies', 'c', 'e']})
        self.assertNotIn(code, (0, 101), said)
        self.assertEqual([name for name, *_ in ran], ['a'])
        self.assertEqual({n: (r['success'], r['not_run']) for n, r in rows.items()},
                         {'a': (True, False), 'dies': (False, True), 'c': (False, True), 'e': (False, True)})
        for name in ('dies', 'c', 'e'):
            self.assertIn(f'not run: {name}: the process of its group g ended (exit 9) before it reported', said)
            self.assertIn(f'test {name} ... FAILED', said)
        self.assertEqual(summary['not_run'], 3)
        self.assertFalse(summary['success'])
        self.assertIn('; 3 not run', said)

    def test_a_group_that_exits_clean_one_verdict_short_is_red(self):
        # The first act of stage 1: the count of tests run equals the count
        # listed. A process that drops a listed test and exits zero is not a
        # pass for it.
        code, rows, ran, said, summary, _ = self.run_library(
            ['a', 'dropped', 'c'], {'g': ['a', 'dropped', 'c']})
        self.assertEqual(code, 125, said)
        self.assertEqual({n: (r['success'], r['exit'], r['not_run']) for n, r in rows.items()},
                         {'a': (True, 0, False), 'dropped': (False, 125, True), 'c': (True, 0, False)})
        self.assertIn('not run: dropped: the process of its group g ended (exit 0) before it reported', said)

    def test_groups_off_runs_every_test_alone(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'groups.json'
            path.write_text(json.dumps({'schema': 1, 'groups': {'g': ['a', 'b']}}))
            self.assertEqual(runner.load_groups(path, {}), {'g': ['a', 'b']})
            self.assertEqual(runner.load_groups(path, {'RON_SUITE_GROUPS': 'off'}), {})
            self.assertEqual(runner.load_groups(Path(d) / 'absent.json', {}), {})
            with self.assertRaises(ValueError):
                runner.load_groups(path, {'RON_SUITE_GROUPS': 'sometimes'})
            path.write_text(json.dumps({'schema': 1, 'groups': {'g': ['a', 'b'], 'h': ['b']}}))
            with self.assertRaisesRegex(ValueError, 'b is in two groups: g and h'):
                runner.load_groups(path, {})

    def test_a_wide_test_and_a_lone_member_are_units_of_one(self):
        units = runner.group_units(['a', 'wide', 'b', 'c', 'x'], {'g': ['a', 'wide', 'b'], 'h': ['c', 'gone']},
                                   {'wide': {'cpu': 4}}, 8)
        self.assertEqual(units, [('g', [(0, 'a'), (2, 'b')]), (None, [(1, 'wide')]), (None, [(3, 'c')]),
                                 (None, [(4, 'x')])])
        # Every selected test is in exactly one unit.
        self.assertEqual(sorted(i for _, members in units for i, _ in members), [0, 1, 2, 3, 4])

    def test_a_verdict_line_is_read_once_and_a_whole_result_covers_a_broken_line(self):
        with tempfile.TemporaryDirectory() as d:
            log = Path(d) / 'log'
            clock = iter(range(10, 100))
            verdicts = runner.Verdicts(log, clock=lambda: next(clock))
            verdicts.poll()
            log.write_text('running 3 tests\ntest a ... ok\ntest b ... ')
            verdicts.poll()
            self.assertEqual(verdicts.seen, {'a': ('ok', 10)})
            with log.open('a') as sink:
                # Another writer broke b's line; libtest's result still counts three.
                sink.write('resource budget: note\nok\ntest c ... FAILED\n' + 'x' * 200000 + '\n'
                           'test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished\n'
                           '        1.00 real         2.00 user         3.00 sys\n  123  maximum resident set size\n'
                           '  456  instructions retired\n')
            verdicts.poll()
            self.assertEqual(verdicts.seen, {'a': ('ok', 10), 'c': ('FAILED', 11)})
            self.assertEqual((verdicts.result, verdicts.cpu, verdicts.rss, verdicts.instructions),
                             (('ok', 3, 0, 0), (2.0, 3.0), 123, 456))
            common = dict(reservation_mib=1, resource_timing={'wait_seconds': 0})
            rows = runner.group_rows('g', ['a', 'b', 'c'], verdicts, exit_code=0, began=9, stamp_held=True, common=common)
            self.assertEqual([(r['name'], r['success'], r['not_run']) for r in rows],
                             [('a', True, False), ('b', True, False), ('c', True, False)])
            self.assertAlmostEqual(sum(r['user'] for r in rows), 2.0)
            # The same lines from a process that did not exit zero: b has no verdict.
            rows = runner.group_rows('g', ['a', 'b', 'c'], verdicts, exit_code=101, began=9, stamp_held=True, common=common)
            self.assertEqual([(r['name'], r['success'], r['exit'], r['not_run']) for r in rows],
                             [('a', True, 0, False), ('b', False, 125, True), ('c', False, 101, False)])
            # A binary that changed under the run passes nothing.
            rows = runner.group_rows('g', ['a'], verdicts, exit_code=0, began=9, stamp_held=False, common=common)
            self.assertFalse(rows[0]['success'])

    def test_a_group_s_allowance_follows_its_members(self):
        self.assertIsNone(runner.group_timeout(['a', 'b'], {'a': 100, 'b': 10}))
        self.assertEqual(runner.group_timeout(['a', 'b'], {'a': 600, 'b': 10}), 2440)
        long = 'diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned'
        self.assertEqual(runner.group_timeout(['a', long], {'a': 1}), 3600)
        self.assertEqual(runner.weights(['a', 'b', 'c'], {'a': {'elapsed': 3}}, {'a': 9, 'b': 2}), {'a': 3, 'b': 2, 'c': 0})

    def test_a_group_s_memory_is_its_own_measured_peak_and_never_a_member_s_row(self):
        # Red first: the first whole run gave a group 1.5 times its largest
        # member's profile row, and `game:run346` (28 tests) was stopped at
        # 789 MiB against 768.
        self.assertEqual(runner.group_memory('g', {'g': 789}), (1280, runner.UNKNOWN_LIMIT_MIB))
        self.assertEqual(runner.group_memory('g', {'g': 1}), (256, runner.UNKNOWN_LIMIT_MIB))
        self.assertEqual(runner.group_memory('g', {'h': 789}), (runner.UNKNOWN_MIB, runner.UNKNOWN_LIMIT_MIB))
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'groups.json'
            path.write_text(json.dumps({'schema': 1, 'groups': {'g': ['a', 'b']}, 'memory_mib': {'g': 789}}))
            self.assertEqual(runner.load_group_memory(path), {'g': 789})
            path.write_text(json.dumps({'schema': 1, 'groups': {'g': ['a', 'b']}}))
            self.assertEqual(runner.load_group_memory(path), {})
            self.assertEqual(runner.load_group_memory(Path(d) / 'absent.json'), {})
        # Through the real process path: the members' profile rows say 1
        # byte each, and the group still reserves from its own peak.
        seen = []
        real = runner.wrapper
        def spy(directory, memory, cpu=1, limit=None, timeout=None):
            seen.append((memory, limit))
            return real(directory, memory, cpu=cpu, limit=limit, timeout=timeout)
        with patch.object(runner, 'wrapper', side_effect=spy):
            code, rows, *_ = self.run_library(['a', 'b', 'c', 'd'], {'g': ['a', 'b'], 'h': ['c', 'd']},
                                              peaks={'g': 789})
        self.assertEqual(code, 0)
        self.assertEqual(sorted(seen), [(1280, runner.UNKNOWN_LIMIT_MIB), (runner.UNKNOWN_MIB, runner.UNKNOWN_LIMIT_MIB)])
        self.assertEqual({rows[n]['reservation_mib'] for n in 'ab'}, {1280})
        self.assertEqual({rows[n]['limit_mib'] for n in 'abcd'}, {runner.UNKNOWN_LIMIT_MIB})

    def test_a_group_is_told_whether_its_tests_share_a_walk_and_a_lone_test_is_told_nothing(self):
        with tempfile.TemporaryDirectory() as d:
            for asked, want in ((None, runner.SHARE_WALKS), ('on', 'on'), ('off', 'off')):
                share = Path(d) / f'share-{asked}'
                env = {'FAKE_SHARE': str(share), **({'RON_SUITE_SHARE_WALKS': asked} if asked else {})}
                with patch.dict(os.environ, env):
                    os.environ.pop('RON_SHARE_WALKS', None)
                    if not asked:
                        os.environ.pop('RON_SUITE_SHARE_WALKS', None)
                    code, *_ = self.run_library(['a', 'b', 'lone'], {'g': ['a', 'b']})
                self.assertEqual(code, 0)
                self.assertEqual(sorted(share.read_text().split('\n')[:-1]),
                                 [f'a {want}', f'b {want}', 'lone None'])
        with self.assertRaises(ValueError):
            runner.share_walks({'RON_SUITE_SHARE_WALKS': 'sometimes'})

    def test_the_groups_file_names_each_test_once_and_no_wide_test(self):
        if not runner.GROUPS.is_file():
            self.skipTest('no groups file in this tree')
        groups = runner.load_groups(runner.GROUPS, {})
        profile = json.loads(runner.PROFILE.read_text())['tests']
        self.assertTrue(groups)
        for key, tests in groups.items():
            self.assertGreater(len(tests), 1, key)
            self.assertEqual(tests, sorted(tests), key)
            for name in tests:
                self.assertEqual(runner.width(name, profile, 8), 1, name)
        # A measured peak names a group the file holds.
        self.assertLessEqual(set(runner.load_group_memory(runner.GROUPS)), set(groups))


class ExcludedPackages(unittest.TestCase):
    def test_a_package_left_to_the_booking_gate_keeps_its_binaries_out_of_the_run(self):
        # Loop 3653 (the fifty-seventh pass): the gate passes
        # `--exclude-package sim` for a lane that touches no sim path.
        manifests = {'/w/crates/sim/Cargo.toml': '[package]\nname = "sim"\n',
                     '/w/crates/rondata/Cargo.toml': '[package]\nname = "rondata"\n'}
        def row(exe, manifest, src, name, kind):
            return {'reason': 'compiler-artifact', 'executable': exe, 'manifest_path': manifest,
                    'profile': {'test': True}, 'target': {'src_path': src, 'name': name, 'kind': [kind]}}
        rows = [row('/t/sim-1', '/w/crates/sim/Cargo.toml', '/w/crates/sim/src/lib.rs', 'sim', 'lib'),
                row('/t/sweep_a_f-1', '/w/crates/sim/Cargo.toml', '/w/crates/sim/tests/sweep_a_f.rs', 'sweep_a_f', 'test'),
                row('/t/rondata-1', '/w/crates/rondata/Cargo.toml', '/w/crates/rondata/src/lib.rs', 'rondata', 'lib'),
                {'reason': 'build-script-executed', 'executable': None, 'profile': {'test': False}}]
        read = manifests.__getitem__
        kept, left = runner.kept_targets(rows, exclude=('sim',), read=read)
        self.assertEqual([exe for exe, _ in kept], ['/t/rondata-1'])
        self.assertEqual(left, [('sim', '/w/crates/sim/src/lib.rs'), ('sim', '/w/crates/sim/tests/sweep_a_f.rs')])
        kept, left = runner.kept_targets(rows, read=read)
        self.assertEqual(len(kept), 3)
        self.assertEqual(left, [])

