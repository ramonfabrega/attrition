"""`tools/archive_compress.py`: the archives are compressed one tree at a
time, and a file the compressor's race left reading empty is found and
mended (DECISIONS 156 §6; the pass of 2026-11-23, third: four trace logs).

(a) the compressor is run once a tree, never on two trees together;
(b) a file whose flag is off over a valid header is found by the scan;
(c) `repair` gives it back byte for byte, and refuses a file not in that state;
(d) linked pairs compressed a tree at a time come out whole.

(b) to (d) make the state for real and need `afsctool`; without it they skip.

(e) a run whose own output is a file under a tree it would compress is
refused before anything is compressed (the pass of 2026-11-25, second).
"""
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import archive_compress as ac  # noqa: E402

HAVE = any(shutil.which(e) for e in ac.AFSCTOOL)


def body(k):
    return b''.join(b'RONT trace text %d %d\n' % (k, i) for i in range(40000))


class OneTreeAtATime(unittest.TestCase):
    def test_the_compressor_is_run_once_a_tree(self):
        calls = []
        if not HAVE:
            self.skipTest('no afsctool')
        ac.compress([Path('/x/ron-data'), Path('/x/ron-golden')], run=lambda argv, **_: calls.append(argv))
        self.assertEqual([c[-1] for c in calls], ['/x/ron-data', '/x/ron-golden'])
        for c in calls:
            self.assertEqual(sum(a.startswith('/x/') for a in c), 1, 'one tree a run')


class ItsOwnOutput(unittest.TestCase):
    # The pass of 2026-11-25, second, sent this tool's output and its
    # acts' log to its directory under `~/ron-data`. The compressor took
    # both files while they were open, every later write to them failed,
    # and the two scripts ended with exit 120 and exit 1 and no line saying
    # why.
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.tmp = Path(self.dir.name).resolve()
        (self.tmp / 'tree').mkdir()
        (self.tmp / 'out').mkdir()

    def tearDown(self):
        self.dir.cleanup()

    def test_a_file_under_a_tree_is_named_and_one_outside_is_not(self):
        with open(self.tmp / 'tree/run.log', 'w') as inside, open(self.tmp / 'out/run.log', 'w') as outside:
            self.assertEqual(ac.output_under([self.tmp / 'tree'], fds=(outside.fileno(), inside.fileno())),
                             self.tmp / 'tree/run.log')
            self.assertIsNone(ac.output_under([self.tmp / 'tree'], fds=(outside.fileno(),)))
        read, write = os.pipe()
        try:
            self.assertIsNone(ac.output_under([self.tmp / 'tree'], fds=(write,)))
        finally:
            os.close(read)
            os.close(write)

    def test_a_run_writing_into_its_own_tree_is_refused_before_it_compresses(self):
        tool = ROOT / 'tools/archive_compress.py'
        with open(self.tmp / 'tree/run.log', 'w') as log:
            done = subprocess.run([sys.executable, str(tool), str(self.tmp / 'tree')],
                                  stdout=log, stderr=subprocess.PIPE, text=True, timeout=60)
        self.assertEqual(done.returncode, 2)
        self.assertIn('refused', done.stderr)
        self.assertIn('tree/run.log', done.stderr)
        self.assertEqual((self.tmp / 'tree/run.log').read_text(), '')


@unittest.skipUnless(HAVE, 'no afsctool')
class TheDamagedState(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix='attrition-archive-compress-'))
        self.addCleanup(shutil.rmtree, self.tmp, True)

    def linked(self, k):
        a, b = self.tmp / 'data', self.tmp / 'golden'
        a.mkdir(exist_ok=True), b.mkdir(exist_ok=True)
        (b / f'take{k}.log').write_bytes(body(k))
        os.link(b / f'take{k}.log', a / f'filed{k}.log')
        return b / f'take{k}.log'

    def test_a_flag_off_over_a_valid_header_is_found_and_mended(self):
        p = self.linked(0)
        ac.compress([self.tmp / 'golden'], quiet=True)
        st = os.lstat(p)
        self.assertTrue(st.st_flags & ac.UF_COMPRESSED, 'the compressor took it')
        self.assertEqual(ac.scan([self.tmp]), {})
        os.chflags(p, st.st_flags & ~ac.UF_COMPRESSED)   # the race's leaving
        self.assertEqual(os.lstat(p).st_size, 0)
        found = ac.scan([self.tmp])
        self.assertEqual(len(found), 1)
        self.assertEqual(sorted(Path(n).name for n in next(iter(found.values()))), ['filed0.log', 'take0.log'])
        self.assertEqual(ac.repair(p), len(body(0)))
        self.assertEqual(p.read_bytes(), body(0))
        self.assertEqual((self.tmp / 'data/filed0.log').read_bytes(), body(0))
        self.assertEqual(ac.scan([self.tmp]), {})

    def test_repair_refuses_a_whole_file_and_an_empty_one(self):
        p = self.linked(1)
        with self.assertRaises(ac.Refused):
            ac.repair(p)
        e = self.tmp / 'empty.log'
        e.write_bytes(b'')
        with self.assertRaises(ac.Refused):
            ac.repair(e)
        self.assertEqual(ac.scan([self.tmp]), {})

    def test_linked_pairs_come_out_whole_a_tree_at_a_time(self):
        ps = [self.linked(k) for k in range(8)]
        ac.compress([self.tmp / 'data', self.tmp / 'golden'], quiet=True)
        self.assertEqual(ac.scan([self.tmp]), {})
        for k, p in enumerate(ps):
            self.assertEqual(p.read_bytes(), body(k))


if __name__ == '__main__':
    unittest.main()
