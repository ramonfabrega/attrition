"""The capture stanzas merge by union, and a run stands once (parked 1161,
the twentieth pass).

`tools/gamelog/captures.txt` is append-only by stanza, one capture each, and
two lanes append at the same line: `ccc update` refused on it for items
1147 and 1156, beside two sections appended to one document, and each lane
fell back to a raw merge that kept both sides. `docs/RUNS.md` has merged by
union since the eighth pass (parked 428); the stanza file does now. What a
union can get wrong is the one thing it cannot see — both sides writing the
same run — so a run number that heads two stanzas fails here.

A specification is *amended in place*, so it does not merge by union: two
lanes amending one paragraph would keep both and say nothing. Two sections
appended at one anchor stay a refusal, and the lane keeps both by hand.
"""
import re
import unittest
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CAPTURES = ROOT / 'tools/gamelog/captures.txt'
ATTRIBUTES = ROOT / '.gitattributes'


def runs_twice(text):
    """The run numbers that head more than one stanza, in file order."""
    heads = re.findall(r'^run:[ \t]*(\S+)[ \t]*$', text, re.M)
    return [run for run, n in Counter(heads).items() if n > 1]


def union_merged(text):
    """The paths `.gitattributes` merges by union."""
    out = []
    for line in text.split('\n'):
        if line.lstrip().startswith('#') or not line.strip():
            continue
        path, *attrs = line.split()
        if 'merge=union' in attrs:
            out.append(path)
    return out


class TheStanzasMergeByUnion(unittest.TestCase):
    def test_the_stanza_file_merges_by_union(self):
        self.assertIn('tools/gamelog/captures.txt', union_merged(ATTRIBUTES.read_text()))

    def test_the_ledger_still_does(self):
        self.assertIn('docs/RUNS.md', union_merged(ATTRIBUTES.read_text()))

    def test_no_specification_merges_by_union(self):
        for path in union_merged(ATTRIBUTES.read_text()):
            self.assertIn(path, ('docs/RUNS.md', 'tools/gamelog/captures.txt'))

    def test_a_run_written_by_both_sides_is_seen(self):
        both = 'run: 461\ntag: a\nframes: 920\n\nrun: 462\ntag: b\n\nrun: 461\ntag: c\n'
        self.assertEqual(runs_twice(both), ['461'])
        self.assertEqual(runs_twice('run: 461\ntag: a\n\nrun: 462\ntag: b\n'), [])

    def test_no_run_heads_two_stanzas(self):
        text = CAPTURES.read_text()
        self.assertGreater(len(re.findall(r'^run:', text, re.M)), 100)
        self.assertEqual(runs_twice(text), [])


if __name__ == '__main__':
    unittest.main()
