"""`tools/seams.py`: what the tree says it left out, by name (parked 1146,
1159 and 1193, the twentieth pass).

Three of East Indies' eight words in one tranche were held by something
already written down as left out: a `SEAM` on `soft_collision` (6151), a
"not modelled" row of ORDERS §4.4 (6321), a `SEAM` whose door had been
built four days before (7512). The fixtures below are those three shapes,
authored; the pin at the end is the live tree's.
"""
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import seams  # noqa: E402

SOURCE = '''
impl Sim {
    /// The soft row.
    ///
    /// SEAM: the `TRADE_ROUTE`/`0xf` arm needs an action index this crate
    /// does not carry, and no capture has entered it.
    ///
    /// ~~SEAM: the group arm needs `UnitData::group`.~~ It keeps one now.
    pub(crate) fn soft_collision(&self, a: usize) -> bool {
        // SEAM: a ship never asks. scan: `grep -c BOAT run33.txt`, none;
        // no capture has a boat beside a caravan.
        false
    }

    fn set_blocked_at(&mut self, x: i32) {
        // SEAM: the road's clearing is left out: `World::clear_mesh_at`
        // is not built.
    }

    fn clear_mesh_at(&mut self) {}
}
'''

QUEUE = '''# The queue

## The queue

1214. **East Indies' second word: frame 8907, ours 2 draws against 28**
    (1197), at index 1: theirs `Unit::think_scout+0x941 < Unit::think+0x7da
    < Unit::do_idle+0x94`, which parts `1/35` alone — its `group` (65
    against 79) and `order:kind` among them. No mechanism is named.

## How to maintain this file
'''


class Seams(unittest.TestCase):
    def test_a_block_of_several_seams_is_several(self):
        found = seams.seams(SOURCE)
        self.assertEqual([(fn, struck) for _, fn, _, struck in found],
                         [('soft_collision', False), ('soft_collision', True),
                          ('soft_collision', False), ('set_blocked_at', False)])

    def test_a_doc_comment_s_seam_is_the_function_s_under_it(self):
        line, fn, text, _ = seams.seams(SOURCE)[0]
        self.assertEqual(fn, 'soft_collision')
        self.assertIn('TRADE_ROUTE', text)
        self.assertEqual(line, 3)

    def test_an_absence_with_no_scan_is_unscanned(self):
        found = seams.seams(SOURCE)
        self.assertTrue(seams.unscanned(found[0][2]))
        self.assertFalse(seams.unscanned(found[2][2]))

    def test_a_name_is_a_whole_word_with_its_class_off(self):
        rx = seams.matcher(['Unit::soft_collision+0x94'])
        self.assertTrue(rx.fullmatch('soft_collision'))
        self.assertIsNone(rx.search('same_group_soft_collision_row'))
        self.assertTrue(seams.matcher(['TRADE_ROUTE']).search('the `TRADE_ROUTE`/`0xf` arm'))

    def test_an_item_gives_its_chain_and_not_its_keys(self):
        self.assertEqual(seams.item_names(QUEUE, 1214), ['think_scout', 'do_idle'])
        with self.assertRaisesRegex(ValueError, '1215'):
            seams.item_names(QUEUE, 1215)

    def test_a_seam_its_door_outlived_is_a_door(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / 'world.rs').write_text(SOURCE)
            saved = seams.SIM, seams.ROOT
            seams.SIM = seams.ROOT = Path(tmp)
            try:
                found = seams.doors(Path(tmp))
            finally:
                seams.SIM, seams.ROOT = saved
        self.assertEqual([(fn, built) for _, _, fn, built, _ in found],
                         [('set_blocked_at', ['clear_mesh_at'])])

    def test_a_struck_paragraph_is_not_a_row(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / 'ORDERS.md').write_text(
                '## 4.4 The take\n\n'
                "The waypoint take's region check in `do_move` is not modelled.\n\n"
                '~~The turn in `do_move` is not modelled.~~ Built, item 9.\n\n'
                '`do_move` walks.\n')
            (Path(tmp) / 'JOURNAL.md').write_text('`do_move` is not modelled.\n')
            saved = seams.ROOT
            seams.ROOT = Path(tmp)
            try:
                rows = seams.spec_rows(seams.matcher(['do_move']), Path(tmp))
            finally:
                seams.ROOT = saved
        self.assertEqual([(str(f), line, section) for f, line, section, _ in rows],
                         [('ORDERS.md', 3, '4.4 The take')])


class TheLiveTree(unittest.TestCase):
    # Exact, and it may only fall: a seam that names, as missing, a
    # function this crate carries is struck, re-worded to say what of the
    # function is missing, or booked by the frame it holds (parked by the
    # twentieth pass). A new one is one of those three before it lands.
    DOORS = 15

    def test_the_seams_a_door_outlived_are_the_pinned_count(self):
        self.assertEqual(len(seams.doors()), self.DOORS)

    def test_the_live_tree_has_seams(self):
        n = sum(1 for f in sorted(seams.SIM.rglob('*.rs'))
                for s in seams.seams(f.read_text()) if not s[3])
        self.assertGreater(n, 150)


if __name__ == '__main__':
    unittest.main()
