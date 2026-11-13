"""`tools/suite_timings.py`: the lane gate's timings file (the thirty-sixth
pass). The thirty-fifth pass re-timed the whole suite alone — 849 tests, 59
minutes, the loop held the while (Loop 2524) — because 71 tests were
untimed; ten were untimed again one tranche later. `--stale-only` times the
untimed tests alone and merges them into the file, so the step is minutes.
"""
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import suite_timings  # noqa: E402

OLD = """# Every rondata release test timed alone, 2026-10-15, tools/suite_timings.py (tree bb8dad3f).
# One test per process, --test-threads 1, sequential, RON_INSTALL set; seconds then the test.
# Sum 10 s over 2 tests; the run 1 min wall. Top 10 = 10 s, top 30 = 10, top 100 = 10; 0 tests under 1 s.
# By module: golden 10 s / 2 tests.
7.00\tdiff::golden::a
3.00\tdiff::golden::b
"""


class StaleOnly(unittest.TestCase):
    def test_new_rows_merge_under_the_kept_ones(self):
        text = suite_timings.merged(OLD, [(2.5, 'diff::golden::c'), (4.0, 'diff::golden::b')],
                                    '2026-10-16', 'abcd1234')
        rows = suite_timings.timed(text)
        self.assertEqual(rows, {'diff::golden::a': 7.0, 'diff::golden::b': 4.0, 'diff::golden::c': 2.5})
        self.assertIn('Sum 14 s over 3 tests', text)
        self.assertIn('tree abcd1234', text)
        # Slowest first, as the whole file is written.
        self.assertLess(text.index('diff::golden::a'), text.index('diff::golden::c'))

    def test_the_stale_names_are_the_untimed_ones(self):
        names = [('bin', 'diff::golden::a'), ('bin', 'diff::golden::c'), ('bin', 'x::y')]
        self.assertEqual(suite_timings.stale_only(names, OLD),
                         [('bin', 'diff::golden::c'), ('bin', 'x::y')])


TIME_L = """running 1 test
test diff::golden::a ... ok
        1.98 real         1.71 user         0.12 sys
           507510784  maximum resident set size
         30012345678  instructions retired
          5730000000  cycles elapsed
"""


class Work(unittest.TestCase):
    """Item 4216's baseline: CPU seconds and instructions beside the wall."""

    def test_cpu_is_user_plus_system_and_instructions_are_read(self):
        cpu, instructions = suite_timings.work(TIME_L)
        self.assertAlmostEqual(cpu, 1.83)
        self.assertEqual(instructions, 30012345678)

    def test_a_line_time_did_not_print_reads_minus_one(self):
        self.assertEqual(suite_timings.work('test result: ok.'), (-1.0, -1))

    def test_the_cpu_file_sums_what_was_read(self):
        text = suite_timings.render_cpu([(1.5, 2_000_000_000, 'a'), (4.0, 8_500_000_000, 'b'), (-1.0, -1, 'c')],
                                        '2026-11-13', 'abcd1234', '; the load 3.0')
        self.assertIn('Sum 6 CPU-seconds and 0.01e12 instructions over 3 tests; the load 3.0.', text)
        rows = [l for l in text.splitlines() if not l.startswith('#')]
        self.assertEqual(rows, ['4.00\t8.50\tb', '1.50\t2.00\ta', '-1.00\t-0.00\tc'])


class Snapshot(unittest.TestCase):
    def test_each_executable_is_copied_under_its_own_name(self):
        import tempfile
        copied = []
        with tempfile.TemporaryDirectory() as d:
            out = suite_timings.snapshot(['/t/release/deps/rondata-abc', '/t/release/deps/survey-def'],
                                         Path(d) / 'kept', copy=lambda a, b: copied.append((a, str(b))))
            self.assertEqual(out, [str(Path(d) / 'kept/rondata-abc'), str(Path(d) / 'kept/survey-def')])
            self.assertTrue((Path(d) / 'kept').is_dir())
        self.assertEqual([a for a, _ in copied], ['/t/release/deps/rondata-abc', '/t/release/deps/survey-def'])
        self.assertEqual([b for _, b in copied], out)


if __name__ == '__main__':
    unittest.main()
