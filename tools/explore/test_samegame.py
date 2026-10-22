"""`tools/gamelog/samegame.py --from F` — the driven clause (Loop 3435, the
forty-ninth pass).

A take that sends an input its `cover=0` twin does not share — a click, an
`@` line the twin dropped — parts the twin at that input by construction,
so "0 differing" is the wrong question for it (item 3427's three driven
takes, item 3436's six, item 3437's three pool takes all part their twins at
their first unshared input). The question that is still worth asking is
whether the two games were the same up to that input: `--from F` asserts
that no frame before F differs, prints the first differing frame as before,
and exits 0 when the first difference is at F or later — or there is none.
A difference before F is still exit 1, with the frame named.

These run the script on two authored dumps whose blocks agree on frames
1..4 and differ from frame 5 on.
"""
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'tools/gamelog/samegame.py'


def dump(first_differ, frames=8):
    out = []
    for n in range(1, frames + 1):
        out.append(f'BEGIN FRAME {n}')
        out.append('  BEGIN UNITDATA')
        out.append(f'    x {100 if n < first_differ else 200}')
        out.append('  END')
    return '\n'.join(out) + '\n'


class SamegameFrom(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.a = Path(self.tmp.name) / 'a.txt'
        self.b = Path(self.tmp.name) / 'b.txt'
        self.a.write_text(dump(99))
        self.b.write_text(dump(5))

    def tearDown(self):
        self.tmp.cleanup()

    def run_it(self, *extra):
        return subprocess.run(['python3', str(SCRIPT), str(self.a), str(self.b), *extra],
                              capture_output=True, text=True, env=dict(os.environ, RON_GAMELOG_DIR=self.tmp.name))

    def test_a_bare_run_is_red_on_the_difference(self):
        done = self.run_it()
        self.assertEqual(done.returncode, 1, done.stdout + done.stderr)
        self.assertIn('differ: 3; first: [5, 6, 7]', done.stdout)

    def test_from_the_first_unshared_input_the_difference_is_expected(self):
        done = self.run_it('--from', '5')
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        self.assertIn('differ: 3; first: [5, 6, 7]', done.stdout)
        self.assertIn('same game before frame 5', done.stdout)

    def test_a_difference_before_the_input_is_still_red(self):
        done = self.run_it('--from', '6')
        self.assertEqual(done.returncode, 1, done.stdout + done.stderr)
        self.assertIn('differs before frame 6: first 5', done.stdout)

    def test_from_takes_the_equals_spelling_too(self):
        done = self.run_it('--from=5')
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)


if __name__ == '__main__':
    unittest.main()
