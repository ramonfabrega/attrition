"""`tools/standing.py`: a widening's standing keys, beside who in `sim`
reads them (parked 1162 and 1187, the twentieth pass).

The brief has asked since items 769 and 834 for "a standing floor row whose
field has a live reader" to be named, in prose; who=1's `pop_cap`, `pop`,
`escrow`, wealth and `trade_val` each stood on a widening's first block,
read as noise, until each was a word. The fixtures are those shapes.
"""
import sys
import tempfile
import unittest
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


class Standing(unittest.TestCase):
    def test_the_print_is_read_through_its_colour(self):
        rows = standing.firsts(PRINT.split('\n'))
        self.assertEqual(len(rows), 7)
        self.assertEqual(rows[0], (7377, 1, -1, 'leader:pop', 'ours 41 theirs 42'))
        self.assertEqual(rows[-1][:4], (7390, 1, 35, 'group'))

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


if __name__ == '__main__':
    unittest.main()
