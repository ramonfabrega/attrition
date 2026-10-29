"""`tools/standing.py`: a widening's standing keys, beside who in `sim`
reads them (parked 1162 and 1187, the twentieth pass).

The brief has asked since items 769 and 834 for "a standing floor row whose
field has a live reader" to be named, in prose; who=1's `pop_cap`, `pop`,
`escrow`, wealth and `trade_val` each stood on a widening's first block,
read as noise, until each was a word. The fixtures are those shapes.
"""
import os
import sys
import tempfile
import unittest
from unittest.mock import patch
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import standing  # noqa: E402

PRINT = '''running 1 test
  first 7377 1/-1 leader:pop: ours 41 theirs 42
  first 7377 1/-1 leader:escrow[2]: ours 0 theirs 120
  first 7377 1/33 g.cur_time[0]: ours 21 theirs 25
  first 7377 1/34 g.cur_time[0]: ours 3 theirs 9
  first 7377 0/2000 city:shade: ours 0 theirs 84
  first 7383 1/35 order:move.dest: ours 0 theirs 1
\x1b[32m  first 7390 1/35 group: ours 65 theirs 79\x1b[0m
test diff::second::tests::run425_s_word_frame_is_widened_whole ... ok
'''

SIM = '''
impl Sim {
    fn research(&self, who: usize) -> bool {
        // .pop in a comment is no reader
        self.leaders[who].pop > 40 && self.leaders[who].escrow[2] == 0
    }

    fn born(&mut self, who: usize) {
        self.leaders[who].pop += 1;
        self.leaders[who].pop = 0;
        if self.leaders[who].pop == 3 {}
    }
}

#[cfg(test)]
mod tests {
    fn a_test_reads_it() {
        assert_eq!(sim.units[0].cur_time, 4);
        assert_eq!(sim.leaders[1].pop, 4);
    }
}
'''



SIBLINGS = '''running 1 test
  standing 17171 1/-1 leader:leader_flags: ours 0 theirs 2
  standing 17171 1/67 order:move.pause: ours 0 theirs 1
  f17244 1/67 order:move.pause: ours 1 theirs 0
  f17244: 3 keys
  near 17245: 2 rows
test diff::third_pair::run661_s_word_frame_is_widened_whole ... ok
'''

COVERAGE = '''
const UNREAD: &[(&str, &str)] = &[
    // a comment
    (
        "GAME/FRAME/BUILDDATA/WALLDATA/OBJECT",
        "down down_who flags mylos near_o size",
    ),
    ("GAME/FRAME/CITIES", "increment length size"),
];
'''


class Standing(unittest.TestCase):
    def setUp(self):
        # Never the machine's export: a test that wants one writes its own.
        import pdb_layout
        absent = patch.dict(os.environ, {'RON_DECOMP': '/nonexistent/decomp'})
        absent.start()
        self.addCleanup(absent.stop)
        pdb_layout.clear()
        self.addCleanup(pdb_layout.clear)

    def test_the_unread_keys_are_listed_by_record(self):
        # Loop 3547: a building's `mylos` is pinned unread, so the Turks'
        # tower printed 19 against ours' 15 on every block of two widenings
        # and parted no key; the walk found it only through a packet. The
        # table is printed beside the standing keys so quiet is not read as
        # agreement.
        rows = standing.unread_table(COVERAGE)
        self.assertEqual(rows, [('GAME/FRAME/BUILDDATA/WALLDATA/OBJECT', 'down down_who flags mylos near_o size'),
                                ('GAME/FRAME/CITIES', 'increment length size')])
        lines = standing.unread_lines(rows)
        self.assertEqual(len(lines), 3)
        self.assertIn('cannot part', lines[0])
        self.assertIn('mylos', lines[1])
        self.assertEqual(len(standing.unread_lines(rows, 'cities')), 2)

    def test_the_print_is_read_through_its_colour(self):
        rows = standing.firsts(PRINT.split('\n'))
        self.assertEqual(len(rows), 7)
        self.assertEqual(rows[0], (7377, 1, -1, 'leader:pop', 'ours 41 theirs 42'))
        self.assertEqual(rows[-1][:4], (7390, 1, 35, 'group'))

    def test_a_golden_widening_s_print_is_read_too(self):
        # Parked 1250: a chapter's widening prints `  ch43 f<frame> …`, not
        # `first <frame> …`, and item 1235 converted it with `sed` by hand.
        rows = standing.firsts(['  ch43 f1356 0/7 order:kind: ours 3 theirs 5',
                                '  ch7b f900 1/-1 leader:pop: ours 4 theirs 5'])
        self.assertEqual(rows, [(1356, 0, 7, 'order:kind', 'ours 3 theirs 5'),
                                (900, 1, -1, 'leader:pop', 'ours 4 theirs 5')])

    def test_a_key_names_its_field(self):
        for what, field in (('leader:pop_cap', 'pop_cap'), ('order:move.dest', 'dest'),
                            ('g.cur_time[0]', 'cur_time'), ('leader:escrow[2]', 'escrow'),
                            ('group', 'group'), ('leader:SITE[0].reg', 'reg'),
                            ('group:70.speed', 'speed'), ('order:kind', 'kind')):
            self.assertEqual(standing.field_of(what), field, what)

    def with_sim(self, fn):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / 'ai.rs').write_text(SIM)
            (Path(tmp) / 'ai_tests.rs').write_text('fn t() { x.shade; x.pop; }\n')
            saved = standing.SIM
            standing.SIM = Path(tmp)
            try:
                return fn(Path(tmp))
            finally:
                standing.SIM = saved

    def test_a_reader_is_outside_the_tests_and_is_no_writer(self):
        found = self.with_sim(lambda root: standing.readers('pop', root))
        self.assertEqual([n for _, n in found], [5, 11])
        self.assertEqual(self.with_sim(lambda root: standing.readers('cur_time', root)), [])
        self.assertEqual(self.with_sim(lambda root: standing.readers('shade', root)), [])

    def test_the_first_block_s_keys_are_the_standing_ones(self):
        def run(root):
            saved = standing.readers
            standing.readers = lambda field, r=root: saved(field, r)
            try:
                return standing.report(standing.firsts(PRINT.split('\n')))
            finally:
                standing.readers = saved
        lines, read = self.with_sim(run)
        self.assertEqual(read, 2)
        self.assertIn('block 7377: 5 keys stand on it, 4 fields', lines[0])
        text = '\n'.join(lines)
        self.assertLess(text.index('  pop: 2 readers'), text.index('  escrow: 1 readers'))
        self.assertIn('nothing in sim reads, by that name: cur_time, shade', text)
        self.assertNotIn('dest', text)

    def test_another_block_is_asked_for(self):
        lines, _ = standing.report(standing.firsts(PRINT.split('\n')), block=7390)
        self.assertIn('block 7390: 1 keys first part on it', lines[0])

    def test_an_input_with_no_print_says_so(self):
        lines, read = standing.report(standing.firsts(['test … ok']))
        self.assertEqual(read, 0)
        self.assertIn('RON_FIRSTS=1', lines[0])


    def test_a_sibling_widening_s_standing_and_f_lines_are_read(self):
        # Parked 1471: `widen_on_siblings` prints `standing <block> …` and
        # `f<block> …`; two landings fed them through `sed` to read them here.
        rows = standing.firsts(SIBLINGS.splitlines(True))
        self.assertEqual([(f, who, o, what) for f, who, o, what, _ in rows],
                         [(17171, 1, -1, 'leader:leader_flags'), (17171, 1, 67, 'order:move.pause'),
                          (17244, 1, 67, 'order:move.pause')])

    # Loop 3768: item 3763 found the one writer of `leader:treaties` it
    # needed (`Unit::set_attacking`'s, `0x00e3a424`) only by grepping the
    # address, never the field's name. A synthetic export, never the real one.
    TYPES = """struct /rise.pdb/Leaders  // size 0x4554c
  +0x0     Leader[10]                               list

struct /rise.pdb/LeaderData  // size 0x6ee4
  +0x0     int                                      leader_flags
  +0x94    int[8]                                   treaties

struct /rise.pdb/MoveOrder  // size 0x5c
  +0x10    int                                      dest
"""
    CONTACT = ['  first 620 1/-1 leader:treaties[2]: ours 1 theirs 3',
               '  first 620 0/-1 leader:treaties[1]: ours 1 theirs 3',
               '  first 620 1/35 order:move.dest: ours 0 theirs 1',
               '  first 620 1/35 g.cur_time[0]: ours 3 theirs 9']

    def contact(self, decomp):
        import pdb_layout
        saved = standing.readers
        standing.readers = lambda field: [] if field == 'cur_time' else [('crates/sim/src/x.rs', 1)]
        try:
            with patch.dict(os.environ, {'RON_DECOMP': str(decomp)}):
                pdb_layout.clear()
                return '\n'.join(standing.report(standing.firsts(self.CONTACT))[0])
        finally:
            standing.readers = saved

    def test_the_export_s_offset_is_read_by_struct_and_field(self):
        import pdb_layout
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / 'types.txt').write_text(self.TYPES)
            with patch.dict(os.environ, {'RON_DECOMP': tmp}):
                pdb_layout.clear()
                self.assertEqual(pdb_layout.offsets('LeaderData', 'treaties'), 0x94)
                self.assertEqual(pdb_layout.offsets('MoveOrder', 'dest'), 0x10)
                self.assertIsNone(pdb_layout.offsets('LeaderData', 'dest'))
                self.assertEqual(pdb_layout.record_structs('order:move.dest'), ('MoveOrder',))
                self.assertEqual(pdb_layout.record_structs('leader:treaties[2]'), ('LeaderData',))
                self.assertEqual(pdb_layout.record_structs('group'), ())
            with patch.dict(os.environ, {'RON_DECOMP': str(Path(tmp) / 'absent')}):
                pdb_layout.clear()
                self.assertIsNone(pdb_layout.offsets('LeaderData', 'treaties'))

    def test_a_standing_field_prints_its_offset_its_grep_spellings_and_its_slot(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / 'types.txt').write_text(self.TYPES)
            text = self.contact(tmp)
        line = next(l for l in text.split('\n') if l.startswith('  treaties:'))
        # who=1: 0x00e3a390 + 1 * 0x6eec + 0x94; leader 0's is 3763's 0x00e3a424.
        self.assertIn('LeaderData @+0x94', line)
        self.assertIn('`field_0x94`', line)
        self.assertIn('`+ 0x94)`', line)
        self.assertIn('`0x00e41310`', line)
        import pdb_layout
        self.assertEqual(pdb_layout.slot_address('leader:treaties[1]', 0, 0x94), 0x00e3a424)
        dest = next(l for l in text.split('\n') if l.startswith('  dest:'))
        self.assertIn('MoveOrder @+0x10', dest)
        self.assertNotIn('`0x00', dest, 'no global covers an order')
        quiet = next(l for l in text.split('\n') if 'nothing in sim reads' in l)
        self.assertNotIn('@+', quiet, 'GuyData is not in this export')

    def test_with_no_export_the_lines_are_unchanged(self):
        with tempfile.TemporaryDirectory() as tmp:
            text = self.contact(Path(tmp) / 'absent')
        self.assertNotIn('@+0x', text)
        line = next(l for l in text.split('\n') if l.startswith('  treaties:'))
        self.assertTrue(line.endswith('ours 1 theirs 3 and 1 more'), line)


if __name__ == '__main__':
    unittest.main()
