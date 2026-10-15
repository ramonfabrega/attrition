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


if __name__ == '__main__':
    unittest.main()
