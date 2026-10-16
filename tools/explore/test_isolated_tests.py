"""Authored processes only: no fixtures, no recursive release gate."""
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


class Admission(unittest.TestCase):
    def test_atomic_capacity_and_release(self):
        with tempfile.TemporaryDirectory() as d:
            first = budget.Lease(d, 15, 32000)
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
            lease = budget.Lease(d, 16, 32768)
            self.assertTrue(lease.try_acquire())
            with subprocess.Popen([sys.executable, '-c', 'import sys; sys.stdin.read()'],
                                  stdin=subprocess.PIPE, pass_fds=lease.fds) as child:
                lease.close()
                other = budget.Lease(d, 1, 128)
                self.assertFalse(other.try_acquire())
                child.communicate()
            self.assertTrue(other.try_acquire())
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

    def test_invalid_budget(self):
        with tempfile.TemporaryDirectory() as d:
            for cpu, memory in [(0, 1), (17, 1), (1, 0), (1, 32769)]:
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

    def test_unreadable_live_owner_refuses_global_monitoring(self):
        rows = f"{os.getpid()} 1 1 10\n"
        with patch.object(budget.subprocess, 'check_output', return_value=rows), patch.object(budget, 'status', return_value=[{'owner': None}]):
            with self.assertRaises(RuntimeError):
                budget.sample_rss(123, '/unused')

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
                     '--cpu', '4', '--memory-mib', '128', '--', sys.executable, '-c', code],
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True))
            events = []
            for process in processes:
                out, err = process.communicate(timeout=15)
                self.assertEqual(process.returncode, 0, err)
                begin, end = map(float, out.splitlines())
                events.extend([(begin, 1), (end, -1)])
            active = peak = 0
            for _, delta in sorted(events):
                active += delta
                peak = max(peak, active)
            self.assertLessEqual(peak, 4)
            self.assertGreater(peak, 1)
            self.assertEqual(active, 0)

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
        self.assertEqual(runner.target_budget({'kind': ['lib'], 'name': 'fixed'}, 8), (8, 512))
        self.assertEqual(runner.target_budget({'kind': ['test'], 'name': 'new'}, 4), (8, 32768))

    def test_discovery_is_strict(self):
        for text in ['a: test\na: test\n', 'unexpected\n', 'bench: benchmark\n']:
            with patch.object(runner.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, text)):
                with self.assertRaises(ValueError):
                    runner.discover('fake')

    def test_unknown_test_gets_whole_memory_budget(self):
        self.assertEqual(runner.reservation('new', {}), 32768)
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


if __name__ == '__main__':
    unittest.main()
