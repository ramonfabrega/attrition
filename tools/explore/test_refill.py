"""A refill's lobby triple is read off the tree, never off a rule (Loop 3769).

Item 3752's booking named `(14, 11, 5)` free by entry 72's rule (3), and the
triple already stood in `battery::SCORED_LOBBIES` as the second pair's lobby.
`tools/refill.py` says who holds a triple; these pin it on the live tree and,
on a synthetic tree, that a triple a comment cites is not an entry.

No live-tree test names a free triple by its digits (Loop 3778). Item 3771's
first gate went red on this file's own `(18, 6, 4)`-is-free pin, because the
item's work was to mint exactly that triple; the lane moved the pin to
`(18, 2, 4)`, which re-arms the trap for whichever booking names it next. A
booking only ever adds an occupant, so a held literal stays true; a free one
is the next booking's to break. The free case is derived from the occupied
set itself, and the held case is read out of it.
"""
import io
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import refill  # noqa: E402

BATTERY = '''
pub(crate) const SCORED_LOBBIES: &[(i32, i32, i32)] = &[
    (14, 11, 0),
    // The peace pair's (item 2320): the Lakota at Difficulty 3.
    (18, 19, 3),
    // Not the Enlightenment pair's `(14, 14, 5)`; that one is a pair's.
    (18, 14, 4),
];

pub(crate) const BATTERY: &[Lobby] = &[
    // `GreatLakesOldTier2` (map_style: 14, ai_tribe: 2, difficulty: 2) left.
    Lobby {
        name: "GreatLakesRomanTier2",
        map_style: 14,
        ai_tribe: 6,
        difficulty: 2,
        start: "a.txt",
    },
];
'''

PAIR = '''
pub(crate) const CHINESE_X: Pair = Pair {
    name: "GreatLakesChineseX",
    map_style: 14,
    ai_tribe: 14,
    difficulty: 5,
    starting_technology: 3,
};
// pub(crate) const DEAD: Pair = Pair { map_style: 9, ai_tribe: 9, difficulty: 9 };
'''

WORD = '''
pub(crate) const ROMAN: Lobby = Lobby {
    name: "EastIndiesRomanTier2",
    map_style: 18,
    ai_tribe: 6,
    difficulty: 2,
};
'''


def synthetic():
    tmp = tempfile.TemporaryDirectory()
    diff = Path(tmp.name) / 'crates/rondata/src/diff'
    diff.mkdir(parents=True)
    (diff / 'battery.rs').write_text(BATTERY)
    (diff / 'chinese_x_pair.rs').write_text(PAIR)
    (diff / 'roman_word.rs').write_text(WORD)
    return tmp, Path(tmp.name)


def free_triple(occ):
    """The first triple over the occupied set's own axes that nothing holds.

    The axes are the maps, nations and difficulties the tree already spells,
    so the triple is a lobby a booking could name, and it is whichever one
    no booking has named yet.
    """
    maps, nations, levels = (sorted({k[i] for k in occ}) for i in range(3))
    for t in ((m, n, d) for m in maps for n in nations for d in levels):
        if t not in occ:
            return t
    raise AssertionError('every triple over the occupied axes is held')


def kinds(occupants):
    return sorted((o.kind, o.where) for o in occupants)


class Synthetic(unittest.TestCase):
    def setUp(self):
        self.tmp, self.root = synthetic()
        self.occ = refill.occupants(self.root)

    def tearDown(self):
        self.tmp.cleanup()

    def test_comment_triples_are_not_entries(self):
        self.assertNotIn((14, 14, 5), {k for k, v in self.occ.items()
                                       if any(o.kind == 'scored' for o in v)})
        self.assertNotIn((14, 2, 2), self.occ)
        self.assertNotIn((9, 9, 9), self.occ)
        scored = sorted(k for k, v in self.occ.items()
                        if any(o.kind == 'scored' for o in v))
        self.assertEqual(scored, [(14, 11, 0), (18, 14, 4), (18, 19, 3)])

    def test_scored_entry_carries_its_comment(self):
        (o,) = self.occ[(18, 19, 3)]
        self.assertIn('peace pair', o.note)

    def test_literals_by_kind(self):
        self.assertEqual(kinds(self.occ[(14, 14, 5)]),
                         [('pair', 'chinese_x_pair.rs CHINESE_X "GreatLakesChineseX"')])
        self.assertEqual(kinds(self.occ[(14, 6, 2)]),
                         [('battery', 'battery.rs "GreatLakesRomanTier2"')])
        self.assertEqual(kinds(self.occ[(18, 6, 2)]),
                         [('word', 'roman_word.rs ROMAN "EastIndiesRomanTier2"')])

    def test_exit_codes(self):
        with redirect_stdout(io.StringIO()) as out:
            self.assertEqual(refill.main(['--root', str(self.root), '7', '7', '7']), 0)
        self.assertIn('free', out.getvalue())
        with redirect_stdout(io.StringIO()) as out:
            self.assertEqual(refill.main(['--root', str(self.root), '14', '14', '5']), 1)
        self.assertIn('chinese_x_pair.rs', out.getvalue())

    def test_nation(self):
        with redirect_stdout(io.StringIO()) as out:
            refill.main(['--root', str(self.root), '--nation', '6'])
        text = out.getvalue()
        self.assertIn('(14, 6, 2)', text)
        self.assertIn('(18, 6, 2)', text)
        self.assertNotIn('(14, 11, 0)', text)


class LiveTree(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.occ = refill.occupants(ROOT)

    def test_14_11_5_is_scored_and_british_modern(self):
        occ = self.occ[(14, 11, 5)]
        self.assertTrue(any(o.kind == 'scored' for o in occ))
        self.assertTrue(any(o.kind == 'pair' and o.where.startswith('british_modern_pair.rs')
                            for o in occ))

    def test_18_6_2_is_scored(self):
        self.assertTrue(any(o.kind == 'scored' for o in self.occ[(18, 6, 2)]))

    def test_18_6_4_is_scored_and_roman_modern(self):
        # Free at item 3771's base; the Roman Modern pair minted it.
        occ = self.occ[(18, 6, 4)]
        self.assertTrue(any(o.kind == 'scored' for o in occ))
        self.assertTrue(any(o.kind == 'pair' and o.where.startswith('roman_modern_pair.rs')
                            for o in occ))

    def test_a_derived_free_triple_reads_free(self):
        # Derived, never a literal: a booking that mints it moves it on.
        t = free_triple(self.occ)
        self.assertFalse(t in self.occ, f'{t} is held: {kinds(self.occ.get(t, []))}')
        with redirect_stdout(io.StringIO()) as out:
            self.assertEqual(refill.main(['--root', str(ROOT), *map(str, t)]), 0)
        self.assertEqual(out.getvalue().strip(), f'{t} free')

    def test_held_triples_read_held_with_their_occupants(self):
        # Read out of the occupied set: the first, the last, and the first scored.
        first_scored = min(k for k, v in self.occ.items()
                           if any(o.kind == 'scored' for o in v))
        for t in sorted({min(self.occ), max(self.occ), first_scored}):
            with self.subTest(triple=t), redirect_stdout(io.StringIO()) as out:
                self.assertEqual(refill.main(['--root', str(ROOT), *map(str, t)]), 1)
                text = out.getvalue()
                self.assertTrue(text.startswith(f'{t}\n'), text)
                for o in self.occ[t]:
                    self.assertIn(o.where, text)

    def test_every_pair_module_spells_its_lobby(self):
        self.assertEqual(refill.unparsed(ROOT), [])


if __name__ == '__main__':
    unittest.main()
