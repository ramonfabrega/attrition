"""`tools/trace/report.py`'s frame arguments (Loop 2512, the thirty-fifth pass).

`draws <frame>` re-reads the whole trace — 277 MB, about a minute — on
every call, and 2460's loop over two hundred frames ran out the foreground
limit. A window is one argument, `lo-hi`, read in one pass.
"""
import importlib.util
import os
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('report', ROOT / 'tools/trace/report.py')
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


class FrameArguments(unittest.TestCase):
    def test_a_window_is_one_argument(self):
        self.assertEqual(report.frame_args(['setup', '5', '7-9']), {-1, 5, 7, 8, 9})
        self.assertEqual(report.frame_args(['4859-4859']), {4859})
        with self.assertRaises(ValueError):
            report.frame_args(['9-7'])


def frame(f):
    return (2, f & 0xFFFFFFFF, 0, 0, 0, 0, 0, f & 0xFFFFFFFF)


def hit(va, f):
    return (0, va, 1, 0, 0, 0, 0, f & 0xFFFFFFFF)


def draw(ret, f, up=0, upup=0):
    # get(a,b): caller (a return address), Random*, seed, caller's caller, its caller
    return (3, ret, 0xE37A8C, 0, up, upup, 0, f & 0xFFFFFFFF)


def head(lo=-1, hi=-2):
    return dict(version=1, base=0x400000, text_rva=0x1000, text_size=0, nfuncs=3, win_lo=lo, win_hi=hi)


STARTS = [0x500000, 0x500100, 0x500200]


def start_of(va):
    return max((a for a in STARTS if a <= va), default=None)


class FramesAFunctionRan(unittest.TestCase):
    """Parked 3973: `coverage` lists a function's first entry, never every
    frame it ran, and a booking once named "run1017's `set_stance` frames"
    with no instrument behind them. `frames <function>` prints every frame
    the trace holds for one function, and says which of its sources is
    complete and which is a first entry only."""

    def test_a_window_records_every_frame_a_stub_entered(self):
        recs = [hit(0x500000, -1), frame(0), frame(1), hit(0x500000, 1),
                frame(2), hit(0x500000, 2), hit(0x500000, 2), frame(3)]
        got = report.function_frames(head(1, 2), recs, {0x500000}, start_of)
        self.assertEqual(got['hit'], [-1, 1, 2])
        self.assertEqual(got['window'], (1, 2))

    def test_without_a_window_a_stub_has_its_first_entry_only(self):
        recs = [frame(0), hit(0x500100, 0), frame(1), frame(2)]
        got = report.function_frames(head(), recs, {0x500100}, start_of)
        self.assertEqual(got['hit'], [0])
        self.assertIsNone(got['window'])

    def test_draws_made_from_a_function_date_every_frame_it_drew(self):
        recs = [frame(0), hit(0x500100, 0), draw(0x500120, 0), frame(1),
                draw(0x500130, 1), draw(0x500140, 1), draw(0x500210, 1, up=0x500150),
                frame(2), draw(0x500010, 2, upup=0x500101)]
        got = report.function_frames(head(), recs, {0x500100}, start_of)
        self.assertEqual(dict(got['site']), {0: 1, 1: 2})
        self.assertEqual(dict(got['chain']), {1: 1, 2: 1})

    def test_the_verb_says_what_a_frame_list_cannot_hold(self):
        import struct
        import subprocess
        import sys
        import tempfile
        recs = [frame(0), hit(0x500100, 0), draw(0x500120, 0), frame(1), draw(0x500130, 1), frame(2)]
        h = head()
        with tempfile.TemporaryDirectory() as d:
            log = Path(d) / 'rontrace.log'
            words = [0x544E4F52, 1, h['base'], h['text_rva'], 0, 3, 0xFFFFFFFF, 0xFFFFFFFE]
            log.write_bytes(b''.join(struct.pack('<8I', *r) for r in [words] + recs))
            idx = Path(d) / 'INDEX.tsv'
            idx.write_text(''.join(f'{a:08x}\tC::f{i}\tx\n' for i, a in enumerate(STARTS)))
            out = subprocess.run([sys.executable, str(ROOT / 'tools/trace/report.py'), str(log),
                                  '--index', str(idx), 'frames', 'C::f1'],
                                 capture_output=True, text=True, check=True).stdout
        self.assertIn('first entry only', out)
        self.assertIn('f0 ', out)
        self.assertIn('f1 ', out)


GOLDEN = Path.home() / 'ron-golden/run1542b/map-7/rontrace.log'
INDEX = Path.home() / 'ghidra-projects/decomp/INDEX.tsv'


class FramesOnACoverTrace(unittest.TestCase):
    """run1542b is a `cover=1` trace with no window: its coverage records
    are each function's first entry, and nothing more."""

    def setUp(self):
        if not GOLDEN.exists() or not INDEX.exists():
            self.skipTest(f'no cover=1 trace at {GOLDEN} (or no {INDEX}): '
                          'the run dir is outside this machine')
        # Indexing a whole `cover=1` trace peaks past 1 GiB of RSS, and the
        # gate's offline step runs under a 1024 MiB reservation (the
        # seventy-third pass's gate stopped on exactly this test at 1109 MiB),
        # so the live-trace cases are opt-in: `RON_COVER_TRACE_TESTS=1`.
        if os.environ.get('RON_COVER_TRACE_TESTS') != '1':
            self.skipTest('live cover=1 trace tests are opt-in (RON_COVER_TRACE_TESTS=1): '
                          'indexing the trace peaks past the offline step\'s 1024 MiB cap')

    def run_frames(self, name):
        import subprocess
        import sys
        return subprocess.run([sys.executable, str(ROOT / 'tools/trace/report.py'), str(GOLDEN),
                               'frames', name], capture_output=True, text=True, check=True).stdout

    def test_a_function_the_trace_enters_more_than_once(self):
        out = self.run_frames(LIVE_FUNCTION)
        rows = [line for line in out.splitlines() if line.startswith('f')]
        self.assertIn('first entry only, f615', out)
        self.assertGreater(len(rows), 1, out)

    def test_a_stub_only_function_says_its_later_frames_are_unrecorded(self):
        out = self.run_frames('Army::set_stance')
        self.assertIn('first entry only, f248', out)
        self.assertEqual([line for line in out.splitlines() if line.startswith('f')], ['f248 first entry'])

    def test_a_proxied_function_has_every_call(self):
        out = self.run_frames('PathFinder::calc_cost')
        self.assertIn('every call recorded', out)


LIVE_FUNCTION = 'Unit::do_attack'


if __name__ == '__main__':
    unittest.main()
