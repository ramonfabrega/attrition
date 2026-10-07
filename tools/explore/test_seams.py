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

    fn find_friends(&self) {
        // The enhancer arm (a Peacock's Wine, a Camp's Fur) is not
        // modelled: an enhancer's tile is walked as any other.
        let _ = 0;
    }

    /// SEAMS: `do_cast`'s captain check is not built; the spell's cost row
    /// is not built either.
    fn do_cast(&self) {}
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
                          ('soft_collision', False), ('set_blocked_at', False),
                          ('do_cast', False)])

    def test_a_plural_block_is_a_seam(self):
        # Parked 1240: `do_cast`'s captain check — chapter forty-three's
        # hypothesis — sat in a `SEAMS:` block, which the scan did not list.
        found = [text for _, fn, text, _ in seams.seams(SOURCE) if fn == 'do_cast']
        self.assertEqual(len(found), 1)
        self.assertIn('captain check', found[0])

    def test_a_comment_that_says_not_modelled_is_a_seam_without_the_word(self):
        # Parked 1253 (three reaches): `find_friends`' enhancer arm said so
        # in a comment, and `seams.py find_friends enhancer` answered "0 live
        # seams" — the arm that decided the third map's 5376.
        found = seams.left_out(SOURCE)
        self.assertEqual([(fn, 'enhancer' in text) for _, fn, text in found],
                         [('find_friends', True)])
        rx = seams.matcher(['find_friends'])
        self.assertTrue(any(rx.fullmatch(fn) for _, fn, _ in found))

    def test_each_spelling_of_a_gap_is_a_seam_without_the_word(self):
        # Four landings of the twenty-sixth tranche and four of the
        # twenty-fifth were held by a gap the scan did not print, each
        # spelled another way (parked 1482, 1560, 1521, 1561).
        spellings = {
            'not loaded': 'The free-tech block (Carpentry) is not loaded here.',
            'reads 1': 'The stub reads 1 for every nation.',
            'reads empty': 'The list reads empty until the loader is written.',
            'unreachable': 'The CHAR_DEFAULT snap is unreachable from here (so it is argued).',
            'never asked': 'is_enemy(8) is never asked for: gaia has no treaties.',
            'never reached': 'The ocean branch is never reached by a capture.',
            'no capture reaches': 'The Oil Platform conversion: no capture reaches it.',
            'only by': 'tech_frame is written only by Leader::init.',
            'reached through': 'The other branch is reached through the fog alone.',
        }
        for key, sentence in spellings.items():
            source = f'fn gap() {{\n    // {sentence}\n    let _ = 0;\n}}\n'
            found = seams.left_out(source)
            self.assertEqual([fn for _, fn, _ in found], ['gap'], key)
            with tempfile.TemporaryDirectory() as tmp:
                (Path(tmp) / 'X.md').write_text(f'# X\n\n## 1. A section\n\n{sentence} gap.\n')
                saved = seams.ROOT
                seams.ROOT = Path(tmp)
                try:
                    rows = seams.spec_rows(seams.matcher(['gap']), Path(tmp))
                finally:
                    seams.ROOT = saved
            self.assertEqual([line for _, line, _, _ in rows], [5], key)

    def test_an_item_gives_its_upper_case_names_too(self):
        # 1552's cause was a paragraph naming the Oil Platform (`OILPLATFORM`)
        # and not the field (parked 1560): an item's own type names go in.
        queue = QUEUE.replace('No mechanism is named.',
                              'The oil well stands on `CHAR_FARM`; the `OILPLATFORM` arm. No mechanism.')
        names = seams.item_names(queue, 1214)
        self.assertIn('CHAR_FARM', names)
        self.assertIn('OILPLATFORM', names)
        self.assertNotIn('No', names)

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

    def test_a_list_under_a_lead_in_is_the_lead_in_s_paragraph(self):
        # Parked 1253's third reach: ROADS §9.4's "**Not modelled**" is a
        # bold lead-in and the names sit in the bullets under it, so the
        # list was invisible to a scan of paragraphs.
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / 'ROADS.md').write_text(
                '### 9.4 What is not established\n\n'
                '**Not modelled, and each of them could write a bit:**\n\n'
                '- **`RoadsOut::mark_splits@00891d40`.** It sets cardinal bits.\n'
                '- The redo pass: `leech_codes`, `delete_straglers`.\n\n'
                'A paragraph after the list, naming `leech_codes` again.\n')
            saved = seams.ROOT
            seams.ROOT = Path(tmp)
            try:
                rows = seams.spec_rows(seams.matcher(['leech_codes']), Path(tmp))
            finally:
                seams.ROOT = saved
        self.assertEqual([(str(f), line, section) for f, line, section, _ in rows],
                         [('ROADS.md', 5, '9.4 What is not established')])
        self.assertTrue(rows[0][3].startswith('**Not modelled'))


FIELD_SOURCE = '''
impl Sim {
    /// SEAM: the pushed animal's turn is not carried.
    fn push_collider(&mut self, u: usize, o: usize) {
        self.units[u].collide_o = o;
        self.units[u].collide_frame += 1;
    }

    fn read_collider(&self, u: usize) -> bool {
        // The spellcasters' turn (`6f9441`): the generals behind
        // `num_standard` — a seam.
        self.units[u].collide_o != 0
    }

    fn unrelated(&self) -> usize {
        self.units.len()
    }

    fn literal(&self) -> Unit {
        Unit { collide_o: 0, ..Unit::default() }
    }

    fn spanning_literal(&self) -> Unit {
        Unit {
            total_time: 0,
            ..Unit::default()
        }
    }
}

/// A declaration after a function: its fields are nobody's writes.
pub(crate) struct Later {
    pub total_time: u32,
    collide_o: usize,
    /// Read by `read_never`, written by no non-test line.
    pub never: u8,
}

impl Later {
    fn read_never(&self) -> u8 {
        self.never
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_test_that_writes_the_field_is_not_a_writer() {
        s.units[0].collide_o = 3;
    }
}
'''


class ByField(unittest.TestCase):
    # Parked 1340, six reaches in one tranche (with 1337): the answer sat
    # in a `SEAM` on the first parted field's writer or reader, not on
    # the word's chain, so `seams.py --item` could not print it and a
    # second call by name did, five times. `--field` names those
    # functions itself, writers first.

    def test_a_field_s_writers_and_readers_are_named(self):
        self.assertEqual(seams.field_functions(FIELD_SOURCE, 'collide_o'),
                         {'push_collider': 'w', 'read_collider': 'r', 'literal': 'w'})

    def test_a_struct_literal_that_spans_lines_is_a_write(self):
        # Parked 1397, two reaches (1380's `total_time`, 1407's `last_x`):
        # a `field: value,` line of a literal read as a declaration's.
        self.assertEqual(seams.field_functions(FIELD_SOURCE, 'total_time'),
                         {'spanning_literal': 'w'})

    def test_a_struct_s_fields_no_non_test_line_writes_are_listed(self):
        # Parked 1382 and 1400, three reaches: `combat::Side`'s fields,
        # `Muster::library_cities`, `gather_stamp` — each read by the sim,
        # written by nothing but a test, and reading as modelled.
        d = Path(tempfile.mkdtemp())
        (d / 'a.rs').write_text(FIELD_SOURCE)
        self.assertEqual(seams.unwritten('Later', d),
                         (['total_time', 'collide_o', 'never'], ['never']))
        self.assertEqual(seams.unwritten('Nobody', d), ([], []))

    def test_a_name_that_is_no_field_says_where_it_occurs(self):
        # Parked 1397's second reach: `--field FleeTo` printed "writers
        # none" for a variant used in three places.
        self.assertEqual(seams.functions_naming(FIELD_SOURCE, 'collide_frame'), ['push_collider'])

    def test_a_field_s_seam_is_found_through_its_writer(self):
        fns = seams.field_functions(FIELD_SOURCE, 'collide_o')
        rx = seams.matcher(['collide_o'] + list(fns))
        hits = [fn for _, fn, text, struck in seams.seams(FIELD_SOURCE)
                if not struck and (rx.search(text) or rx.fullmatch(fn))]
        self.assertEqual(hits, ['push_collider'])

    def test_a_lower_case_seam_is_a_seam_without_the_word(self):
        # Parked 1322: `army.rs`'s "The spellcasters' turn — a seam." had
        # no `SEAM:` and was half of 1305's word.
        found = seams.left_out(FIELD_SOURCE)
        self.assertEqual([fn for _, fn, _ in found], ['read_collider'])


class TheLiveTree(unittest.TestCase):
    # Exact, and it may only fall: a seam that names, as missing, a
    # function this crate carries is struck, re-worded to say what of the
    # function is missing, or booked by the frame it holds (parked by the
    # twentieth pass). A new one is one of those three before it lands.
    # 15 → 14: item 1223 built `disembark`'s squad arm, whose SEAM named
    # `push_group` as missing. 14 → 17 on the twenty-first pass, the
    # instrument widened and not the tree: a plural `SEAMS:` block is a
    # seam now (parked 1240), and three such blocks name a built function —
    # `spellcaster.rs`'s crate doc, `think_civilian_transport`'s and
    # `do_cast`'s. Each is 1224's to strike, re-word or book.
    DOORS = 17

    def test_the_seams_a_door_outlived_are_the_pinned_count(self):
        self.assertEqual(len(seams.doors()), self.DOORS)

    def test_the_live_tree_has_seams(self):
        n = sum(1 for f in sorted(seams.SIM.rglob('*.rs'))
                for s in seams.seams(f.read_text()) if not s[3])
        self.assertGreater(n, 150)


if __name__ == '__main__':
    unittest.main()
