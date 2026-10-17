"""`tools/runwiring.py`: a run's wiring through the harness, place by place
(parked 1674 (g) and 1729, the twenty-ninth pass)."""
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import runwiring  # noqa: E402

TEXTS = {
    'docs/RUNS.md': '## run763 — the coverage pair (2026-10-13, item 1719)\n\n`--map 9`\n',
    'crates/rondata/src/diff/coverage_pair.rs': (
        'const CLOSING: &str =\n    "gamelog-run763-eastindies-closing-window-4677-4731.txt";\n'
        '#[test]\nfn run763_s_closing_frame_is_widened_whole() {\n    let w = 1;\n}\n'),
    'crates/rondata/src/diff/testkit.rs': (
        'pub(crate) const GROUND_INEXACT: &[(&str, u32)] = &[\n'
        '    ("run763_s_closing_frame_is_widened_whole", 514),\n];\n'
        'pub(crate) const WIDENINGS: &[&str] = &[\n    // run762 here\n];\n'),
}


class Wiring(unittest.TestCase):
    def test_every_line_is_listed_under_its_scope(self):
        found = runwiring.mentions('run763', TEXTS)
        self.assertEqual([(f.rsplit('/', 1)[-1], line, scope) for f, line, scope, _ in found],
                         [('coverage_pair.rs', 2, 'CLOSING'), ('coverage_pair.rs', 4, 'run763_s_closing_frame_is_widened_whole'),
                          ('testkit.rs', 2, 'GROUND_INEXACT'), ('RUNS.md', 1, '')])

    def test_the_places_are_checked_off(self):
        checked = dict(runwiring.places('run763', runwiring.mentions('run763', TEXTS)))
        self.assertEqual(checked['runs ledger'], ['docs/RUNS.md:1'])
        self.assertEqual(checked['archive file'], ['crates/rondata/src/diff/coverage_pair.rs:2'])
        self.assertEqual(checked['widening test'], ['crates/rondata/src/diff/coverage_pair.rs:4'])
        self.assertEqual(checked['GROUND_INEXACT'], ['crates/rondata/src/diff/testkit.rs:2'])
        self.assertEqual(checked['WIDENINGS'], [])
        self.assertEqual(checked['coverage driver'], [])

    def test_a_number_is_a_run(self):
        self.assertEqual(runwiring.mentions('763', TEXTS), runwiring.mentions('run763', TEXTS))
        self.assertEqual(runwiring.mentions('run76', TEXTS), [])


if __name__ == '__main__':
    unittest.main()

CHAPTER = {
    'crates/rondata/src/golden.rs': '        ("chapter90.cmd", &[]),\n',
    'crates/rondata/src/diff/golden/cases_80_89.rs': (
        'pub(super) fn chapter_ninety_holds_to_the_golden_word() {\n'
        '    let Some(w) = walk_script("ch90", "chapter90", 90, 14, 1200) else {\n        return;\n    };\n}\n'
        'pub(super) fn chapter_ninety_s_window_is_widened_whole() {\n    widen("chapter90");\n}\n'),
    'crates/rondata/src/diff/testkit.rs': (
        '/// `chapter_ninety_s_window_is_widened_whole`\'s window: run892, blocks 605..1200.\n'
        'pub(crate) const GOLDEN_WORD_CHAPTER_NINETY: i64 = 1199;\n'
        'pub(crate) const WIDENINGS: &[&str] = &[\n    "chapter_ninety_s_window_is_widened_whole",\n];\n'),
    'crates/rondata/src/diff/floors.rs': (
        '    const GOLDEN_WORDS: &[(&str, i64, (i64, i64))] = &[\n'
        '        ("ch90", GOLDEN_WORD_CHAPTER_NINETY, WIDENING_CHAPTER_NINETY),\n    ];\n'),
    'docs/RUNS.md': '## run892 — chapter ninety: the Informer\'s three kills (2026-10-17, item 2949)\n',
}


class Chapter(unittest.TestCase):
    # Loop 2914 (three reaches, the thirty-ninth pass): a chapter's wiring
    # is six files by hand, the staged-line count and the words array's
    # length each hand-counted and each wrong once.
    def test_a_number_s_word_is_the_harness_s(self):
        self.assertEqual(runwiring.number_word(90), 'ninety')
        self.assertEqual(runwiring.number_word(91), 'ninety_one')
        self.assertEqual(runwiring.number_word(7), 'seven')
        self.assertEqual(runwiring.number_word(115, '-'), 'one-hundred-fifteen')

    def test_a_chapter_s_places_are_checked_off_by_its_three_spellings(self):
        found = runwiring.chapter_mentions(90, CHAPTER)
        checked = dict(runwiring.chapter_places(90, found))
        self.assertEqual(checked['script list'], ['crates/rondata/src/golden.rs:1'])
        self.assertEqual(checked['walk'], ['crates/rondata/src/diff/golden/cases_80_89.rs:2'])
        self.assertEqual(checked['walk test'], ['crates/rondata/src/diff/golden/cases_80_89.rs:1'])
        self.assertEqual(checked['widening test'], ['crates/rondata/src/diff/golden/cases_80_89.rs:6'])
        self.assertEqual(checked['GOLDEN_WORD pin'], ['crates/rondata/src/diff/testkit.rs:2'])
        self.assertEqual(checked['WIDENINGS'], ['crates/rondata/src/diff/testkit.rs:4'])
        self.assertEqual(checked['GOLDEN_WORDS row'], ['crates/rondata/src/diff/floors.rs:2'])
        self.assertEqual(checked['coverage driver'], [])
        self.assertEqual(checked['runs ledger'], ['docs/RUNS.md:1'])
        self.assertEqual(dict(runwiring.chapter_places(91, runwiring.chapter_mentions(91, CHAPTER)))['walk'], [])

    def test_the_staged_count_is_read_off_the_script_and_the_walk(self):
        found = runwiring.chapter_mentions(90, CHAPTER)
        self.assertEqual(runwiring.staged_count(90, found, '# chapter ninety\nadd hoplite 1 100,100\n\n@attack 1/2\n'),
                         (2, 14))

