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
