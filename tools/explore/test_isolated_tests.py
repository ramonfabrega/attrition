"""Authored processes only: no fixtures, no recursive release gate."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import isolated_tests as runner
import resource_budget as budget


class Admission(unittest.TestCase):
    def test_atomic_capacity_and_release(self):
        with tempfile.TemporaryDirectory() as d:
            first = budget.Lease(d, 7, 16000)
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
            lease = budget.Lease(d, 8, 16384)
            self.assertTrue(lease.try_acquire())
            with subprocess.Popen([sys.executable, '-c', 'import sys; sys.stdin.read()'],
                                  stdin=subprocess.PIPE, pass_fds=lease.fds) as child:
                lease.close()
                other = budget.Lease(d, 1, 128)
                self.assertFalse(other.try_acquire())
                child.communicate()
            self.assertTrue(other.try_acquire())
            other.close()

    def test_invalid_budget(self):
        with tempfile.TemporaryDirectory() as d:
            for cpu, memory in [(0, 1), (9, 1), (1, 0), (1, 16385)]:
                with self.assertRaises(ValueError):
                    budget.Lease(d, cpu, memory)

    def test_sample_failure_refuses_launch(self):
        with tempfile.TemporaryDirectory() as d, patch.object(budget.subprocess, 'check_output', side_effect=RuntimeError('denied')), patch.object(budget.subprocess, 'Popen') as spawn:
            with self.assertRaises(RuntimeError):
                budget.supervise(['never'], directory=d)
            spawn.assert_not_called()

    def test_timeout_failure_and_memory_kill(self):
        with tempfile.TemporaryDirectory() as d, patch.object(budget, 'tree_rss', return_value=1):
            self.assertEqual(budget.supervise([sys.executable, '-c', 'raise SystemExit(7)'], directory=d), 7)
            self.assertEqual(budget.supervise([sys.executable, '-c', 'import time; time.sleep(5)'], directory=d, timeout=.02), 124)
        with tempfile.TemporaryDirectory() as d, patch.object(budget, 'tree_rss', return_value=2**30):
            self.assertEqual(budget.supervise([sys.executable, '-c', 'import time; time.sleep(5)'], directory=d, memory_mib=128), 137)

    def test_six_independent_suites_share_cpu_capacity(self):
        with tempfile.TemporaryDirectory() as d:
            processes = []
            code = "import time,sys; print(time.monotonic(), flush=True); time.sleep(.3); print(time.monotonic(), flush=True)"
            for _ in range(6):
                processes.append(subprocess.Popen(
                    [sys.executable, str(Path(budget.__file__)), '--directory', d,
                     '--cpu', '2', '--memory-mib', '128', '--', sys.executable, '-c', code],
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


class Manifest(unittest.TestCase):
    def test_discovery_is_strict(self):
        for text in ['a: test\na: test\n', 'unexpected\n', 'bench: benchmark\n']:
            with patch.object(runner.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, text)):
                with self.assertRaises(ValueError):
                    runner.discover('fake')

    def test_unknown_test_gets_whole_memory_budget(self):
        self.assertEqual(runner.reservation('new', {}), 16384)
        self.assertEqual(runner.reservation('small', {'small': {'rss_bytes': 1}}), 128)

    def test_zero_test_success_is_not_a_pass(self):
        self.assertFalse(runner.ONE.search('test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out;'))
        self.assertFalse(runner.ONE.search('test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out;'))

    def test_every_test_once_and_no_fail_fast(self):
        with tempfile.TemporaryDirectory() as d:
            out = Path(d) / 'out'
            seen = []
            def run(binary, name, *args):
                seen.append(name)
                return dict(name=name, success=name != 'bad', exit=0 if name != 'bad' else 101, user=1, system=0)
            with patch.object(runner, 'discover', side_effect=[['ok', 'bad', 'ignored', 'new'], ['ignored']]), patch.object(runner, 'run_one', side_effect=run):
                self.assertEqual(runner.isolated('fake', out, Path(d), 2), 101)
            self.assertCountEqual(seen, ['ok', 'bad', 'new'])
            self.assertTrue(json.loads((out/'summary.json').read_text())['complete'])
            with self.assertRaises(FileExistsError):
                runner.isolated('fake', out, Path(d), 2)


if __name__ == '__main__':
    unittest.main()
