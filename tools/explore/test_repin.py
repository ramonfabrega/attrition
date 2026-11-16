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

    def test_a_want_that_names_a_constant_s_field_rewrites_the_constant(self):
        # Parked 1767, the thirtieth pass: a pair's word is `MONGOL.count`,
        # a literal in the `Pair` const, and three re-pins were hand edits.
        self.file.write_text('''pub(crate) const MONGOL: Pair = Pair {
    name: "EastIndiesMongolAllTech",
    length: 5056,
    count: 2838,
    sequence: 2838,
};

fn t() {
    let _pins = Pins::hold();
    pin_eq!(w.count, MONGOL.count, "the draw count's first parting");
    pin_eq!(w.sequence, MONGOL.sequence, "the draw sequence's first parting");
}
''')
        log = '''1 pins moved:
  crates/rondata/src/diff/fixture.rs:10: the draw count's first parting
    got  2900
    want 2838
'''
        edits, by_hand = repin.plan(repin.parse(log), root=self.root)
        self.assertEqual(by_hand, [])
        repin.apply(edits, root=self.root)
        text = self.file.read_text()
        self.assertIn('    count: 2900,', text)
        self.assertIn('    sequence: 2838,', text)
        self.assertIn('pin_eq!(w.count, MONGOL.count,', text)

    def test_a_want_that_is_a_const_array_rewrites_the_const(self):
        # Loop 2071 (2032, 2099, 2117, 2235: four chapters re-pinned by
        # hand): a golden widening pins its rows as a `const WANT: &[&str]`
        # the test copies into a local and sorts, so the `want` at the site
        # is a local's name and the literal is the const's. The local is
        # followed to its `let … = CONST.iter()` and the const's array is
        # rewritten; a want that names the const itself is rewritten too.
        self.file.write_text('''const WANT_CH9: &[&str] = &[
    "1000 1/2006 city:city_flags[0x4]",
    "1004 0/3 order:flags", // the move
];
const WANT_POOL: &[&str] = &[];

fn t() {
    let _pins = Pins::hold();
    let mut want: Vec<String> = WANT_CH9.iter().map(|r| r.to_string()).collect();
    want.sort();
    pin_eq!(got, want, "ch9: what parts under the word moved");
    pin_eq!(pool, WANT_POOL, "ch9: the pool");
}
''')
        log = '''2 pins moved:
  crates/rondata/src/diff/fixture.rs:11: ch9: what parts under the word moved
    got  ["1000 1/2006 city:city_flags[0x4]", "1004 0/3 order:flags", "1022 1/5 mirror"]
    want ["1000 1/2006 city:city_flags[0x4]", "1004 0/3 order:flags"]
  crates/rondata/src/diff/fixture.rs:12: ch9: the pool
    got  ["7 0/9 near"]
    want []
'''
        edits, by_hand = repin.plan(repin.parse(log), self.root)
        self.assertEqual(by_hand, [])
        self.assertEqual(sum(len(v) for v in edits.values()), 2)
        repin.apply(edits, self.root)
        text = self.file.read_text()
        self.assertIn('const WANT_CH9: &[&str] = &["1000 1/2006 city:city_flags[0x4]", "1004 0/3 order:flags", "1022 1/5 mirror"];', text)
        self.assertIn('const WANT_POOL: &[&str] = &["7 0/9 near"];', text)
        self.assertIn('pin_eq!(got, want, "ch9', text, 'the site itself is untouched')

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

    def test_a_want_beside_a_comment_is_rewritten_and_the_comment_kept(self):
        # Parked 1462, the fifth reach: `repin.py` left 50 of 131 sites by
        # hand when a `want` carried a comment between the arguments, or
        # followed a comment line — a comma in the comment split the
        # arguments, and the comment's text made the want "not a literal".
        source = """fn t() {
    let _pins = Pins::hold();
    pin_eq!(
        w.count,
        // the word: 3142 -> 3395 (1625), stands from the gap, not the block
        3142,
        "the coverage pair's word"
    );
    pin_eq!(
        w.sequence, // 2323 (1620)
        2273, "the sequence"
    );
}
"""
        log = """1 pin moved:
  crates/rondata/src/diff/fixture.rs:3: the coverage pair's word
    got  3395
    want 3142
1 pin moved:
  crates/rondata/src/diff/fixture.rs:9: the sequence
    got  2323
    want 2273
"""
        self.file.write_text(source)
        edits, by_hand = repin.plan(repin.parse(log), self.root)
        self.assertEqual(by_hand, [])
        self.assertEqual(sum(len(v) for v in edits.values()), 2)
        repin.apply(edits, self.root)
        text = self.file.read_text()
        self.assertIn('// the word: 3142 -> 3395 (1625), stands from the gap, not the block\n        3395,', text)
        self.assertIn('w.sequence, // 2323 (1620)\n        2323, "the sequence"', text)

    # Item 4258's account (journal 2026-11-14, "For the Loop"): `--write`
    # "drops a trailing comment that sits between a pin's `want` and the
    # next argument", on 22 of 66 sites, and wrote a `[]` that does not
    # type-check. The lane's evidence was the removed lines of its re-pin
    # commit grepped for `//` (2626de89c: 22 removed, 20 added); the 20
    # trailing comments stood on the added lines. The two it lost were the
    # comment lines *inside* a list `want` the re-pin replaced whole.

    def test_a_trailing_comment_after_the_want_is_kept_byte_for_byte(self):
        # The shapes of `harness.rs`'s floors, verbatim from 6d7f7bffd.
        source = """fn t() {
    let _pins = Pins::hold();
    pin_eq!(
        (under, firsts.len()), // Item 1444: the ruins guard removes stale site fields (batch replay).
        (36, 2, 38), // Item 2997: (125, 2, 127) → (38, 2, 40), the pool. Item 1620: −2, `treaties[0]`/`[1]`.
        "the floor"
    );
    pin_eq!(
        standing_n, // Item 1444: the ruins guard removes stale site fields (batch replay).
        37, // Item 1620: −2, `treaties[0]`/`[1]` compared as the word (`docs/AI.md` §173).
        "every row standing on 14651"
    );
}
"""
        log = """2 pins moved:
  crates/rondata/src/diff/fixture.rs:3: the floor
    got  (25, 0, 25)
    want (36, 2, 38)
  crates/rondata/src/diff/fixture.rs:8: every row standing on 14651
    got  24
    want 37
"""
        self.file.write_text(source)
        edits, by_hand = repin.plan(repin.parse(log), self.root)
        self.assertEqual(by_hand, [])
        repin.apply(edits, self.root)
        want = source.replace('(36, 2, 38), //', '(25, 0, 25), //').replace('37, // Item', '24, // Item')
        self.assertEqual(self.file.read_text(), want)

    def test_a_comment_inside_a_replaced_want_is_kept(self):
        # `harness.rs` run178's rows (6d7f7bffd:6020): the list's own comment
        # lines went with the list. A comment inside a replaced literal is
        # kept, text unchanged, on its own line above the line the literal
        # opens on, at that line's indent; a const array's the same.
        source = """const WANT_CH9: &[&str] = &[
    "1000 1/2006 city:city_flags[0x4]",
    "1004 0/3 order:flags", // the move
];

fn t() {
    let _pins = Pins::hold();
    pin_eq!(
        mid,
        // Item 1106 closed who=0's `gather_stamp` on 14537.
        [
            // Item 1468 (group 17): every unit is born snapped to the 48-grid's centre, so the
            // trained squads' `orders_x`/`orders_y` stand where the original's do: six rows → two.
            (13556, 74),
            (13737, 75), // the second
        ],
        "the rows under the word"
    );
    let want: Vec<String> = WANT_CH9.iter().map(|r| r.to_string()).collect();
    pin_eq!(got, want, "ch9");
}
"""
        log = """2 pins moved:
  crates/rondata/src/diff/fixture.rs:8: the rows under the word
    got  [(13556, 74)]
    want [(13556, 74), (13737, 75)]
  crates/rondata/src/diff/fixture.rs:20: ch9
    got  ["1000 1/2006 city:city_flags[0x4]"]
    want ["1000 1/2006 city:city_flags[0x4]", "1004 0/3 order:flags"]
"""
        self.file.write_text(source)
        edits, by_hand = repin.plan(repin.parse(log), self.root)
        self.assertEqual(by_hand, [])
        repin.apply(edits, self.root)
        text = self.file.read_text()
        self.assertIn("""        // Item 1106 closed who=0's `gather_stamp` on 14537.
        // Item 1468 (group 17): every unit is born snapped to the 48-grid's centre, so the
        // trained squads' `orders_x`/`orders_y` stand where the original's do: six rows → two.
        // the second
        [(13556, 74)],
        "the rows under the word"
""", text)
        self.assertIn('// the move\nconst WANT_CH9: &[&str] = &["1000 1/2006 city:city_flags[0x4]"];\n', text)
        # Every comment line of the fixture is in the output, byte for byte.
        for line in source.split('\n'):
            if '//' in line:
                self.assertIn(line[line.index('//'):], text)

    def test_an_emptied_list_of_strings_takes_the_typed_empty_form(self):
        # Item 4258 (and item 4253 twice): an emptied list written `[]` does
        # not type-check against a `Vec<String>` — `String` is `PartialEq`
        # with `str`, `&str`, `String` and `Cow<str>`, so `[]`'s element
        # type is not inferred. The tree's own empty string lists are
        # `Vec::<String>::new()` (137 `pin_eq!` wants under crates/rondata/src
        # on b4f919fb6), while its `[]` (26) and `vec![]` (25) wants all
        # stand against numeric rows, where the one impl infers.
        source = """fn t() {
    let _pins = Pins::hold();
    pin_eq!(
        mid,
        [
            "13556 1/74 g.angle[0]: ours 1431655765 theirs 0",
            "13737 1/75 g.angle[0]: ours 1431655765 theirs 0"
        ],
        "the rows under the word"
    );
    pin_eq!(names, vec!["a"], "the names");
    pin_eq!(own, vec![(1, 2, 3, 4)], "the rows");
    pin_eq!(by, [(17140, 38)], "the blocks");
    pin_eq!(keyed, vec![("a", 1)], "a row with a string in it");
}
"""
        log = """5 pins moved:
  crates/rondata/src/diff/fixture.rs:3: the rows under the word
    got  []
    want ["13556 1/74 g.angle[0]: ours 1431655765 theirs 0", "13737 1/75 g.angle[0]: ours 1431655765 theirs 0"]
  crates/rondata/src/diff/fixture.rs:11: the names
    got  []
    want ["a"]
  crates/rondata/src/diff/fixture.rs:12: the rows
    got  []
    want [(1, 2, 3, 4)]
  crates/rondata/src/diff/fixture.rs:13: the blocks
    got  []
    want [(17140, 38)]
  crates/rondata/src/diff/fixture.rs:14: a row with a string in it
    got  []
    want [("a", 1)]
"""
        self.file.write_text(source)
        edits, by_hand = repin.plan(repin.parse(log), self.root)
        self.assertEqual([(l, why) for _, l, why, *_ in by_hand],
                         [(14, 'an emptied list of rows holding strings: its type is the site\'s to name')])
        repin.apply(edits, self.root)
        text = self.file.read_text()
        self.assertIn('        mid,\n        Vec::<String>::new(),\n        "the rows under the word"', text)
        self.assertIn('pin_eq!(names, Vec::<String>::new(), "the names");', text)
        self.assertIn('pin_eq!(own, vec![], "the rows");', text)
        self.assertIn('pin_eq!(by, [], "the blocks");', text)
        self.assertIn('pin_eq!(keyed, vec![("a", 1)], "a row', text)

    def test_the_pins_self_test_s_literals_are_never_rewritten(self):
        # Item 4159's second residue: a log of the whole suite carries the
        # pins self-test's own report (`testkit.rs`), and `--write` re-pinned
        # its two deliberate literals; they were put back by hand.
        kit = self.root / 'crates/rondata/src/diff/testkit.rs'
        kit.write_text("""mod tests {
    #[test]
    fn a_test_that_holds_several_pins_reports_every_one_that_moved() {
        let caught = std::panic::catch_unwind(|| {
            let _pins = Pins::hold();
            pin_eq!((1, 2, 3), (1, 2, 4), "the floor");
        });
        let bare = std::panic::catch_unwind(|| pin_eq!(1, 2, "no guard"));
    }

    #[test]
    fn a_real_pin() {
        let _pins = Pins::hold();
        pin_eq!(w, 3, "a real pin");
    }
}
""")
        log = """1 pins moved:
  crates/rondata/src/diff/testkit.rs:6: the floor
    got  (1, 2, 3)
    want (1, 2, 4)
  crates/rondata/src/diff/testkit.rs:8: no guard
    got  1
    want 2
  crates/rondata/src/diff/testkit.rs:14: a real pin
    got  4
    want 3
"""
        before = kit.read_text()
        edits, by_hand = repin.plan(repin.parse(log), self.root)
        self.assertEqual([(l, why) for _, l, why, *_ in by_hand],
                         [(6, "the pins self-test's deliberate literal"),
                          (8, "the pins self-test's deliberate literal")])
        repin.apply(edits, self.root)
        self.assertEqual(kit.read_text(), before.replace('pin_eq!(w, 3,', 'pin_eq!(w, 4,'))


if __name__ == '__main__':
    unittest.main()
