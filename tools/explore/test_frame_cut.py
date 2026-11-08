"""`tools/gamelog/frame.py` cuts one frame's block, never the dump's tail (Loop 3977).

A capture with a stride dumps a `BEGIN FRAME` every fifty frames, so the
line after frame 651's block is `BEGIN FRAME 701`, not `BEGIN FRAME 652`.
The cut stopped only on `n + 1`, and on run1542 it wrote 2.0 M lines for
651 — forty frames' blocks — so `one.py --file` read the last record in
the file rather than the frame's (3975). The cut stops at the next
`BEGIN FRAME` of any number, as `danger.py` and the other block readers do.
"""
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FRAME = ROOT / 'tools/gamelog/frame.py'
RUN1542 = Path(os.environ.get('RON_GOLDEN', Path.home() / 'ron-golden')) / 'run1542/map-7/gamelog.txt'


def cut(dump, frame, out):
    """Run the tool as a capture session does; its stdout and the cut's lines."""
    done = subprocess.run([sys.executable, '-I', str(FRAME), str(dump), str(frame), str(out)],
                          capture_output=True, text=True, check=True)
    return done.stdout, Path(out).read_text().splitlines()


def frames_in(lines):
    return [s.strip() for s in lines if s.strip().startswith('BEGIN FRAME')]


class StridedDump(unittest.TestCase):
    """The dump's shape, authored: CRLF, a leading space, one block in fifty."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self.tmp.name)
        body = []
        for n in (601, 651, 701, 751):
            body += [f' BEGIN FRAME {n}', '  BEGIN UNITDATA', f'   uid {n}', '   myhits 130']
        self.dump = self.dir / 'gamelog.txt'
        self.dump.write_bytes(('\r\n'.join(body) + '\r\n').encode())

    def tearDown(self):
        self.tmp.cleanup()

    def test_a_strided_frame_stops_at_the_next_block(self):
        _, lines = cut(self.dump, 651, self.dir / 'f651.txt')
        self.assertEqual(frames_in(lines), ['BEGIN FRAME 651'])
        self.assertEqual([s.strip() for s in lines], ['BEGIN FRAME 651', 'BEGIN UNITDATA', 'uid 651', 'myhits 130'])

    def test_a_contiguous_frame_still_stops_at_n_plus_one(self):
        self.dump.write_bytes(b' BEGIN FRAME 4\r\n  uid 4\r\n BEGIN FRAME 5\r\n  uid 5\r\n')
        _, lines = cut(self.dump, 4, self.dir / 'f4.txt')
        self.assertEqual([s.strip() for s in lines], ['BEGIN FRAME 4', 'uid 4'])

    def test_the_last_block_runs_to_the_end(self):
        _, lines = cut(self.dump, 751, self.dir / 'f751.txt')
        self.assertEqual([s.strip() for s in lines], ['BEGIN FRAME 751', 'BEGIN UNITDATA', 'uid 751', 'myhits 130'])

    def test_a_frame_between_blocks_is_refused(self):
        with self.assertRaises(subprocess.CalledProcessError):
            cut(self.dump, 652, self.dir / 'f652.txt')

    def test_indentation_is_kept(self):
        _, lines = cut(self.dump, 651, self.dir / 'f651.txt')
        self.assertEqual(lines[1], '  BEGIN UNITDATA')


@unittest.skipUnless(RUN1542.is_file(), f'run1542 dump absent: {RUN1542}')
class Run1542(unittest.TestCase):
    """run1542's 651, the frame 3975 read: stride 50, 2,687,629 lines."""

    def test_651_is_one_frame(self):
        with tempfile.TemporaryDirectory() as d:
            out, lines = cut(RUN1542, 651, Path(d) / 'f651.txt')
        with open(RUN1542, errors='replace') as fh:
            total = sum(1 for _ in fh)
        print(f'\nrun1542 651: {len(lines)} lines cut of {total} ({out.strip()})', file=sys.stderr)
        self.assertEqual(frames_in(lines), ['BEGIN FRAME 651'])
        # The tail from 651 is 2.0 M lines; one frame's block is tens of thousands.
        self.assertLess(len(lines), 200_000)


if __name__ == '__main__':
    unittest.main()
