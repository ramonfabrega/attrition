"""`tools/repin_held.py` — a held-out row's `moved` score from the score test's print (the pass of 2026-11-27).

A two-entry table in a temporary file, rows 13 and 33 of a sixth battery, and logs that score them:

(a) a row's `moved` takes the log's states, counts, first partings and end, and nothing
    else in the file changes: `found`, the comment over `moved`, the other entry;
(b) a row with `moved: None` is held to `found`: scored as found it is left alone, and parted
    from it it takes a `moved` while `found` stands;
(c) a second run of the same log moves nothing and writes nothing;
(d) an entry whose comment names another row is refused and nothing is written;
(e) a log with no `HELD OUT row` line is refused, exit 2;
(f) a row the log does not score is refused;
(g) the real table parses: 56 entries, each under a comment that names its row, and each
    `moved` read back as the values it holds.
"""
import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import repin_held as rh  # noqa: E402

TABLE = '''pub(crate) const SCORES_SIXTH: [Option<FirstScore>; 8] = [
    // run1788
    // row 13, Amazon, as found.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [7675, 7675, 12001, 9001, 12001, 7675, 7675, 7751, 7700, 9000, 8286, 7676],
            first: &[
                ("units", (7675, Some(11), 12)),
            ],
            over: None,
        },
        // Moved by item 1: a sentence the worker wrote.
        moved: Some(Score {
            states: 12001,
            agreed: [7675, 7675, 12001, 9001, 12001, 7675, 7675, 7751, 7700, 9000, 8286, 7676],
            first: &[
                ("units", (7675, Some(11), 12)),
            ],
            over: None,
        }),
    }),
    // run1789
    // row 33, as found.
    Some(FirstScore {
        last: None,
        found: Score {
            states: 12001,
            agreed: [12001; 12],
            first: &[],
            over: None,
        },
        moved: None,
    }),
];
'''
AGREED_13 = '[8000, 8100, 12001, 9001, 12001, 8200, 8000, 8300, 7700, 9000, 8286, 8001]'
LOG_13 = (f'HELD OUT row 13 run1788: states 12001 agreed {AGREED_13} '
          'first [("units", (8000, Some(21), 22)), ("leaders", (8300, None, 5))] over Some(11000)\n')
LOG_33 = ('HELD OUT row 33 run1789: states 12001 agreed [12001, 12001, 12001, 12001, 12001, 12001, '
          '12001, 12001, 12001, 12001, 12001, 12001] first [] over None\n')


class RepinHeld(unittest.TestCase):
    def run_tool(self, table, log, *args):
        with tempfile.TemporaryDirectory() as d:
            f, l = Path(d, 'held_out.rs'), Path(d, 'run.log')
            f.write_text(table)
            l.write_text(log)
            out, err = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                code = rh.main([str(l), '--file', str(f), *args])
            return code, f.read_text(), out.getvalue(), err.getvalue()

    def test_a_row_s_moved_takes_the_log_and_nothing_else_changes(self):
        code, text, out, _ = self.run_tool(TABLE, 'noise\n' + LOG_13, '--row', '13', '--write')
        self.assertEqual(code, 0)
        self.assertIn('units agreeing 7,675 -> 8,000', out)
        self.assertIn('units first 7675 -> 8000', out)
        self.assertIn('leaders first none -> 8300', out)
        self.assertIn("ours' end None -> Some(11000)", out)
        a, b = TABLE.index('        moved: Some(Score {'), TABLE.index('    // run1789')
        self.assertEqual(text[:a], TABLE[:a], 'found and the comment over moved stand')
        self.assertEqual(text[text.index('    // run1789'):], TABLE[b:], 'the other entry stands')
        got = rh.read_moved(text[a:text.index('    // run1789')])
        self.assertEqual(got, (12001, [8000, 8100, 12001, 9001, 12001, 8200, 8000, 8300, 7700, 9000, 8286, 8001],
                               [('units', 8000, 'Some(21)', 22), ('leaders', 8300, 'None', 5)], 'Some(11000)'))

    def test_a_row_never_moved_is_held_to_found_and_takes_a_moved_when_it_parts(self):
        code, text, out, _ = self.run_tool(TABLE, LOG_33, '--write')
        self.assertEqual((code, text), (0, TABLE), 'as found: nothing to write')
        self.assertIn('nothing moved', out)
        log = LOG_33.replace('[12001, 12001,', '[11000, 12001,').replace('first []', 'first [("units", (11000, Some(1), 2))]')
        code, text, out, _ = self.run_tool(TABLE, log, '--write')
        self.assertEqual(code, 0)
        self.assertIn('row 33 (SCORES_SIXTH[1].moved): units agreeing 12,001 -> 11,000; units first none -> 11000', out)
        self.assertNotIn('moved: None', text)
        self.assertIn('        found: Score {\n            states: 12001,\n            agreed: [12001; 12],', text)
        self.assertIn('                ("units", (11000, Some(1), 2)),\n            ],\n            over: None,\n        }),', text)

    def test_a_second_run_moves_nothing(self):
        _, text, _, _ = self.run_tool(TABLE, LOG_13, '--write')
        code, again, out, _ = self.run_tool(text, LOG_13, '--write')
        self.assertEqual((code, again), (0, text))
        self.assertIn('nothing moved', out)

    def test_without_write_nothing_is_written(self):
        code, text, out, _ = self.run_tool(TABLE, LOG_13)
        self.assertEqual((code, text), (0, TABLE))
        self.assertIn('--write', out)

    def test_an_entry_under_another_row_s_comment_is_refused(self):
        table = TABLE.replace('// row 13, Amazon', '// row 14, Amazon')
        code, text, _, err = self.run_tool(table, LOG_13, '--write')
        self.assertEqual((code, text), (1, table))
        self.assertIn('names no row 13', err)

    def test_a_log_with_no_score_line_is_refused(self):
        code, text, _, err = self.run_tool(TABLE, 'test result: ok\n', '--write')
        self.assertEqual((code, text), (2, TABLE))
        self.assertIn('no `HELD OUT row` line', err)

    def test_a_row_the_log_does_not_score_is_refused(self):
        code, text, _, err = self.run_tool(TABLE, LOG_13, '--row', '33', '--write')
        self.assertEqual((code, text), (1, TABLE))
        self.assertIn('row 33: the log prints no score', err)

    def test_the_real_table_parses(self):
        text = rh.HELD.read_text()
        seen = 0
        for residue, const in rh.BATTERIES.items():
            if f'const {const}:' not in text:
                continue
            for k in range(8):
                start, end, named = rh.entry_span(text, residue + 20 * k)
                self.assertEqual(named, const)
                a, b = rh.moved_span(text, start, end)
                got = rh.read_moved(text[a:b])
                if got is not None:
                    self.assertEqual(rh.read_moved(rh.literal(*got)), got)
                    self.assertEqual(len(got[1]), 12)
                seen += 1
        self.assertGreaterEqual(seen, 56)


if __name__ == '__main__':
    unittest.main()
