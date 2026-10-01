"""`tools/repin.py`: the moved-pin report rewrites its literal sites
(parked 1349 and 1411, the twenty-third pass).

The fixture is a source file with the pin shapes `rondata::diff` uses — a
one-line number, a multi-line tuple, an `Option<&str>` through
`.as_deref()`, a `vec!`, a constant's name — and a cargo log carrying
`Pins::hold()`'s report for each.
"""
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import repin  # noqa: E402

SOURCE = '''fn t() {
    let _pins = Pins::hold();
    pin_eq!(w.blocks, 257, "run471 whole: blocks 5371..5627");
    pin_eq!(
        by.iter().map(|(b, n)| (*b, *n)).collect::<Vec<_>>(),
        vec![(5376, 3), (5377, 1)],
        "the first partings"
    );
    pin_eq!(
        row(1, -1, "leader:SITE[2].reg").as_deref(),
        Some("5376: ours 1 theirs 0"),
        "who=1's third site's region"
    );
    pin_eq!(w.firsts.len(), WORD_KEYS, "every key parted on run471");
    pin!(w.missing.is_empty(), "run471 carries every key: {:?}", w.missing);
    pin_ne!(a, 5);
}
'''

LOG = '''running 3 tests
thread 'diff::sahara_toughest::tests::run471' panicked at crates/rondata/src/diff/testkit.rs:8280:
4 pins moved:
  crates/rondata/src/diff/fixture.rs:3: run471 whole: blocks 5371..5627
    got  258
    want 257
  crates/rondata/src/diff/fixture.rs:4: the first partings
    got  [(5376, 3), (5377, 2), (5380, 1)]
    want [(5376, 3), (5377, 1)]
  crates/rondata/src/diff/fixture.rs:9: who=1's third site's region
    got  Some("5376: ours 1 theirs 2")
    want Some("5376: ours 1 theirs 0")
  crates/rondata/src/diff/fixture.rs:14: every key parted on run471
    got  141
    want 140
thread 'other' panicked:
2 pins moved:
  crates/rondata/src/diff/fixture.rs:15: run471 carries every key: ["x"]
  crates/rondata/src/diff/fixture.rs:16: pin_ne
    both 5
'''


class Repin(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        self.file = self.root / 'crates/rondata/src/diff/fixture.rs'
        self.file.parent.mkdir(parents=True)
        self.file.write_text(SOURCE)

    def test_the_report_is_parsed_site_by_site(self):
        sites = repin.parse(LOG)
        self.assertEqual(len(sites), 6)
        self.assertEqual(sites[('crates/rondata/src/diff/fixture.rs', 3)],
                         [('run471 whole: blocks 5371..5627', '258', '257')])
        self.assertEqual(sites[('crates/rondata/src/diff/fixture.rs', 15)][0][1], None)

    def test_literal_wants_are_rewritten_and_the_rest_named(self):
        edits, by_hand = repin.plan(repin.parse(LOG), self.root)
        self.assertEqual(sum(len(v) for v in edits.values()), 3)
        self.assertEqual([(l, why.split(' ')[0]) for _, l, why, *_ in by_hand],
                         [(14, 'the'), (15, 'no'), (16, 'no')])
        repin.apply(edits, self.root)
        text = self.file.read_text()
        self.assertIn('pin_eq!(w.blocks, 258, "run471 whole', text)
        self.assertIn('vec![(5376, 3), (5377, 2), (5380, 1)],', text)
        self.assertIn('Some("5376: ours 1 theirs 2"),', text)
        self.assertIn('WORD_KEYS', text)
        # Nothing else moved: the file is the fixture with three wants swapped.
        self.assertEqual(len(text.split('\n')), len(SOURCE.split('\n')))

    def test_a_site_two_tests_moved_to_two_values_is_by_hand(self):
        log = LOG + '''
1 pin moved:
  crates/rondata/src/diff/fixture.rs:3: run471 whole: blocks 5371..5627
    got  259
    want 257
'''
        edits, by_hand = repin.plan(repin.parse(log), self.root)
        self.assertEqual(sum(len(v) for v in edits.values()), 2)
        self.assertIn('moved to 2 values', [why for _, _, why, *_ in by_hand][0])

    def test_nothing_is_written_without_the_flag(self):
        log = self.root / 'run.log'
        log.write_text(LOG)
        repin.main([str(log), '--root', str(self.root)])
        self.assertEqual(self.file.read_text(), SOURCE)
        repin.main([str(log), '--root', str(self.root), '--write'])
        self.assertNotEqual(self.file.read_text(), SOURCE)


if __name__ == '__main__':
    unittest.main()
