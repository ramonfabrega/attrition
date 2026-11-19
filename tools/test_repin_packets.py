"""`tools/repin_packets.py` — the packets' `PINNED` rows from a print walk (the pass of 2026-11-19).

A two-row table in a temporary file, and a log that moves one of them:

(a) a moved row takes the log's twelve tuples, one comment above it naming
    each line's move, and nothing else in the file changes but the score;
(b) `PACKETS_SCORE` is recomputed from the whole table, a line counted when
    theirs or ours is non-zero and agreeing when none parts;
(c) a row the log does not move is left byte for byte, and a second run of
    the same log moves nothing;
(d) a `PIN` for a run the table does not hold, or on another frame, is
    listed and left alone;
(e) a log with no `PIN` line is refused, exit 2;
(f) the real table parses: its rows' sum is the `PACKETS_SCORE` it pins.
"""
import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import repin_packets as rp  # noqa: E402

ZERO = '(0, 0, 0)'
TABLE = '''pub(crate) const PACKETS_SCORE: (usize, usize, usize) = (2, 2, 4);

pub(super) const PINNED: &[Pinned] = &[
    // Item 1, the first row.
    (
        "run1",
        10,
        [
            (5, 5, 2),
            (3, 3, 0),
''' + ''.join(f'            {ZERO},\n' for _ in range(10)) + '''        ],
    ),
    (
        "run2",
        20,
        [
            (7, 7, 0),
            (4, 5, 9),
''' + ''.join(f'            {ZERO},\n' for _ in range(10)) + '''        ],
    ),
];
'''


def pin(run, frame, first, second):
    rest = ', '.join([ZERO] * 10)
    return f'PIN\t("{run}", {frame}, [{first}, {second}, {rest}]),\n'


class RepinPackets(unittest.TestCase):
    def run_tool(self, log, *extra):
        with tempfile.TemporaryDirectory() as d:
            table, logf = Path(d) / 'packets.rs', Path(d) / 'log'
            table.write_text(TABLE)
            logf.write_text(log)
            out = io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
                code = rp.main([str(logf), '--item', '9', '--note', 'a test', '--file', str(table), *extra])
            first = table.read_text()
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                rp.main([str(logf), '--item', '9', '--note', 'a test', '--file', str(table)])
            return code, out.getvalue(), first, table.read_text()

    def test_a_moved_row_is_rewritten_with_its_comment_and_the_score(self):
        log = 'noise\n' + pin('run1', 10, '(5, 5, 0)', '(3, 3, 0)') + pin('run2', 20, '(7, 7, 0)', '(4, 5, 9)')
        code, out, text, again = self.run_tool(log)
        self.assertEqual(code, 0)
        self.assertIn('    // Item 9, a test: units (5, 5, 2) -> (5, 5, 0).\n    (\n        "run1",', text)
        self.assertIn('            (5, 5, 0),\n            (3, 3, 0),', text)
        self.assertIn('const PACKETS_SCORE: (usize, usize, usize) = (2, 3, 4);', text)
        self.assertIn('turned\trun1\tunits\t(5, 5, 2) -> (5, 5, 0)', out)
        self.assertIn('PACKETS_SCORE lines 2 -> 3 of 4', out)
        # (c): run2's row untouched, and the second run changed nothing.
        self.assertIn(TABLE[TABLE.index('    (\n        "run2"'):], text)
        self.assertEqual(text, again)

    def test_a_dry_run_writes_nothing(self):
        log = pin('run1', 10, '(5, 5, 0)', '(3, 3, 0)')
        with tempfile.TemporaryDirectory() as d:
            table, logf = Path(d) / 'packets.rs', Path(d) / 'log'
            table.write_text(TABLE)
            logf.write_text(log)
            with contextlib.redirect_stdout(io.StringIO()):
                rp.main([str(logf), '--item', '9', '--note', 'n', '--file', str(table), '--dry-run'])
            self.assertEqual(table.read_text(), TABLE)

    def test_a_grown_and_a_parted_line_are_named(self):
        log = pin('run2', 20, '(7, 7, 1)', '(4, 5, 11)')
        _, out, text, _ = self.run_tool(log)
        self.assertIn('parted\trun2\tunits', out)
        self.assertIn('grown\trun2\tbuilds', out)
        self.assertIn('const PACKETS_SCORE: (usize, usize, usize) = (2, 1, 4);', text)

    def test_a_row_the_table_does_not_hold_is_left_alone(self):
        log = pin('run3', 30, '(1, 1, 0)', ZERO) + pin('run1', 11, '(5, 5, 0)', '(3, 3, 0)')
        code, out, text, _ = self.run_tool(log)
        self.assertEqual(code, 0)
        self.assertIn('left alone\trun3: no pinned row', out)
        self.assertIn('left alone\trun1: the table pins frame 10, the log walked 11', out)
        self.assertEqual(text, TABLE)

    def test_a_log_with_no_pin_line_is_refused(self):
        code, out, text, _ = self.run_tool('test result: ok\n')
        self.assertEqual(code, 2)
        self.assertIn('no PIN line', out)
        self.assertEqual(text, TABLE)

    def test_the_real_table_parses_to_its_score(self):
        text = rp.TABLE.read_text()
        m = rp.SCORE.search(text)
        self.assertIsNotNone(m)
        self.assertEqual(rp.score(text), tuple(int(m.group(k)) for k in (2, 3, 4)))


if __name__ == '__main__':
    unittest.main()
