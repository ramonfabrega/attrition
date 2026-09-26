"""rngcmp.py's own guards, run as a subprocess; all trace records are authored.

The record format is the tracer's: 32 bytes, eight little-endian u32, kind 2
being the per-frame `game_random` word (frame, word) that the identity check
compares. A 32-byte header precedes the records and is skipped.
"""
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

RNGCMP = Path(__file__).resolve().parents[1] / 'gamelog' / 'rngcmp.py'


def trace(path, words):
    with path.open('wb') as f:
        f.write(bytes(32))
        for frame, word in words:
            f.write(struct.pack('<8I', 2, frame, word, 0, 0, 0, 0, 0))


class RngcmpTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def run_cmp(self, left, right):
        return subprocess.run([sys.executable, str(RNGCMP), str(left), str(right)],
                              capture_output=True, text=True)

    def test_absolute_paths_are_taken_as_given(self):
        a, b = self.root / 'a.log', self.root / 'b.log'
        trace(a, [(1, 111), (2, 222)])
        trace(b, [(1, 111), (2, 222)])
        done = self.run_cmp(a, b)
        self.assertIn('frames in common: 2', done.stdout)
        self.assertIn('differing frames: 0', done.stdout)
        self.assertEqual(done.returncode, 0)

    def test_a_differing_word_is_named_and_loud(self):
        a, b = self.root / 'a.log', self.root / 'b.log'
        trace(a, [(1, 111), (2, 222), (3, 333)])
        trace(b, [(1, 111), (2, 999), (3, 333)])
        done = self.run_cmp(a, b)
        self.assertIn('differing frames: 1 first: [2]', done.stdout)
        # The trace numbers frames one ahead of the harness (parked 845).
        self.assertIn('harness word: 1', done.stdout)
        self.assertEqual(done.returncode, 1)

    def test_nothing_in_common_is_not_agreement(self):
        # The failure this catches: two traces of disjoint frame ranges once
        # read as "differing frames: 0" and were taken for the same game.
        a, b = self.root / 'a.log', self.root / 'b.log'
        trace(a, [(1, 111), (2, 222)])
        trace(b, [(90, 111), (91, 222)])
        done = self.run_cmp(a, b)
        self.assertIn('frames in common: 0', done.stdout)
        self.assertIn('differing frames: 0', done.stdout)
        self.assertEqual(done.returncode, 1)


if __name__ == '__main__':
    unittest.main()
