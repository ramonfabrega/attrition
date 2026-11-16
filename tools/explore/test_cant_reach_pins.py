"""`tools/cant_reach_pins.py`: `--write` re-pins `cant_reach::PINNED` from a
test log and keeps the table's comments (item 4159's third residue: the
rewrite dropped every one, and eight rows were re-pinned by hand).

The fixture carries the table's shapes on b4f919fb6: a rustfmt four-line
row with a comment inside it and a pin of 0 kept by hand, a one-line row
with a comment after it, a four-line row, and a history block between the
last row and `];`.
"""
import io
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import cant_reach_pins  # noqa: E402

TABLE = '''//! The module.

pub(crate) const PINNED: &[(&str, usize)] = &[
    (
        "diff::coverage_pair::run1075_s_garrison_window_is_widened_whole",
        // Item 4159's eighth residue: 6 -> 0, `do_move` keeps the goal on top after a failed plan.
        0,
    ),
    ("diff::golden::ch38", 1), // Item 4185: the Flak Gun's launch row.
    (
        "diff::harness::tests::run96_s_word_frame_is_widened_whole",
        5,
    ),
    // Item 4159's sixth residue: run1075's walk 33 -> 6, the muster chase
    // land-only (`find_muster_spot`'s `FILTER_DOMAIN, 0`).
];

fn other() {}
'''

LOG = '''test diff::harness::tests::run96_s_word_frame_is_widened_whole ... FAILED
  cant_reach tally diff::harness::tests::run96_s_word_frame_is_widened_whole 3
  cant_reach tally diff::golden::ch38 0
  cant_reach tally diff::third::run613_s_walk 2
  cant_reach tally diff::airbase_melee::the_window 4
'''


class CantReachPins(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        self.file = self.root / 'crates/rondata/src/diff/cant_reach.rs'
        self.file.parent.mkdir(parents=True)
        self.file.write_text(TABLE)
        self.log = self.root / 'run.log'
        self.log.write_text(LOG)
        self.saved = (cant_reach_pins.ROOT, cant_reach_pins.FILE, sys.argv)
        cant_reach_pins.ROOT, cant_reach_pins.FILE = self.root, self.file

    def tearDown(self):
        cant_reach_pins.ROOT, cant_reach_pins.FILE, sys.argv = self.saved

    def run_tool(self, *flags):
        sys.argv = ['cant_reach_pins.py', str(self.log), *flags]
        out = io.StringIO()
        with redirect_stdout(out):
            code = cant_reach_pins.main()
        return code, out.getvalue()

    def test_write_moves_the_counts_and_keeps_every_comment(self):
        # A moved row takes the log's count in place; a row the log zeroes
        # leaves and its comment stands where it was; a new row goes in by
        # name; the row the log does not name (run1075's, pinned 0 by hand,
        # its comment inside it) is untouched.
        code, out = self.run_tool('--write')
        self.assertEqual(code, 0, out)
        self.assertEqual(self.file.read_text(), '''//! The module.

pub(crate) const PINNED: &[(&str, usize)] = &[
    ("diff::airbase_melee::the_window", 4),
    (
        "diff::coverage_pair::run1075_s_garrison_window_is_widened_whole",
        // Item 4159's eighth residue: 6 -> 0, `do_move` keeps the goal on top after a failed plan.
        0,
    ),
    // Item 4185: the Flak Gun's launch row.
    (
        "diff::harness::tests::run96_s_word_frame_is_widened_whole",
        3,
    ),
    ("diff::third::run613_s_walk", 2),
    // Item 4159's sixth residue: run1075's walk 33 -> 6, the muster chase
    // land-only (`find_muster_spot`'s `FILTER_DOMAIN, 0`).
];

fn other() {}
''')
        self.assertIn('4 moved; 3 tests, 9 units in all', out)

    def test_a_comment_inside_a_leaving_row_stands_in_its_place(self):
        self.log.write_text('  cant_reach tally diff::harness::tests::run96_s_word_frame_is_widened_whole 0\n'
                            '  cant_reach tally diff::coverage_pair::run1075_s_garrison_window_is_widened_whole 0\n')
        self.file.write_text(TABLE.replace('''        "diff::harness::tests::run96_s_word_frame_is_widened_whole",
        5,''', '''        "diff::harness::tests::run96_s_word_frame_is_widened_whole",
        // Item 4204: 7 -> 5, the head quit.
        5,'''))
        code, out = self.run_tool('--write')
        self.assertEqual(code, 0, out)
        text = self.file.read_text()
        self.assertNotIn('run96_s_word', text)
        self.assertIn('''    ("diff::golden::ch38", 1), // Item 4185: the Flak Gun's launch row.
    // Item 4204: 7 -> 5, the head quit.
    // Item 4159's sixth residue''', text)
        self.assertIn('run1075_s_garrison', text, 'the log names it at its pin: it stays')

    def test_new_rows_at_one_point_go_in_by_name(self):
        self.log.write_text('  cant_reach tally diff::golden::ch40 2\n  cant_reach tally diff::golden::ch39 1\n')
        code, out = self.run_tool('--write')
        self.assertEqual(code, 0, out)
        self.assertIn('''    ("diff::golden::ch38", 1), // Item 4185: the Flak Gun's launch row.
    ("diff::golden::ch39", 1),
    ("diff::golden::ch40", 2),
    (
        "diff::harness::tests::run96''', self.file.read_text())

    def test_the_tree_s_own_table_reads_back_unchanged(self):
        # A log that moves nothing leaves the shipped table byte for byte.
        real = (ROOT / 'crates/rondata/src/diff/cant_reach.rs').read_text()
        text, moved, _ = cant_reach_pins.rewrite(real, {})
        self.assertEqual((text, moved), (real, {}))

    def test_nothing_is_written_without_the_flag(self):
        code, out = self.run_tool()
        self.assertEqual(code, 0, out)
        self.assertEqual(self.file.read_text(), TABLE)
        self.assertIn('diff::harness::tests::run96_s_word_frame_is_widened_whole: 5 -> 3', out)


if __name__ == '__main__':
    unittest.main()
