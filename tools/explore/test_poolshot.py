"""`tools/explore/poolshot.py` — the click-free pool's window-only photograph
(Loop 3469, 3451; the fiftieth pass).

Three lanes rebuilt the same shooter in their job's tmp (items 3444, 3460,
3463), and each paid for one of its traps once. These pin the three that a
pure function can hold, with the `screencapture` call stubbed: nothing here
lists a real window or takes a real photograph.

(a) the window is the one whose owner's command line names the run's own
    directory — as Wine writes it, with backslashes and a drive letter, and
    never a sibling run whose name merely starts the same (att-3455's game
    was on screen beside 3460's);
(b) a request with no window id — the shape of a desktop grab — is refused,
    by the Python and by the shim viadriver runs, and nothing is captured;
(c) the frame wait reads the gamelog's CR-terminated `BEGIN FRAME N` lines
    (3460's second recon keyed on `911$` and never matched), and does not
    count a line the game has not finished writing.
"""
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/explore'))

import poolshot  # noqa: E402

RUN = Path('/Users/someone/ron-golden/run1090')


def win(wid, pid, w=1920, h=1080, layer=0, owner='wine64-preloader'):
    return poolshot.Window(wid, pid, layer, w, h, owner)


class WindowMatcher(unittest.TestCase):
    def test_window_matcher_picks_the_run_dirs_window_with_backslashes(self):
        cmdlines = {
            100: r'Z:\Users\someone\ron-golden\run1091\riseofnations_trace.exe -automation',
            200: r'Z:\Users\someone\ron-golden\run1090\riseofnations_trace.exe -automation',
            300: r'Z:\Users\someone\ron-golden\run10900\riseofnations_trace.exe',
            400: '/Applications/Zed.app/Contents/MacOS/zed /Users/someone/notes',
        }
        windows = [win(11, 100), win(22, 200, 1, 1), win(23, 200), win(24, 200, 400, 300, layer=3),
                   win(33, 300), win(44, 400)]
        got = poolshot.pick_window(windows, cmdlines, RUN)
        self.assertIsNotNone(got)
        self.assertEqual((got.id, got.pid), (23, 200))
        # The same directory with forward slashes (a posix argv) matches too,
        # and so does a lower-cased drive path; a prefix sibling never does.
        self.assertTrue(poolshot.cmdline_names_dir('wine /Users/someone/ron-golden/run1090/x.exe', RUN))
        self.assertTrue(poolshot.cmdline_names_dir(r'z:\users\someone\ron-golden\run1090\x.exe', RUN))
        self.assertFalse(poolshot.cmdline_names_dir(r'Z:\Users\someone\ron-golden\run10900\x.exe', RUN))
        self.assertTrue(poolshot.cmdline_names_dir(r'Z:\Users\someone\ron-golden\run1090', RUN))
        # No process names the run: no window, and no guess.
        self.assertIsNone(poolshot.pick_window(windows, {k: v for k, v in cmdlines.items() if k != 200}, RUN))


class DesktopRefused(unittest.TestCase):
    def test_desktop_shaped_request_is_refused(self):
        calls = []
        for bad in (None, 0, -1, '', 'desktop'):
            with self.assertRaises(poolshot.Refused):
                poolshot.shoot(bad, Path('/tmp/never.png'), runner=calls.append)
        self.assertEqual(calls, [], 'a refused request reached the runner')
        argv = poolshot.grab_argv(4914, Path('/tmp/x.png'))
        self.assertEqual(argv[-3:], [str(poolshot.GRAB), '4914', '/tmp/x.png'])
        self.assertEqual(Path(argv[1]).name, 'viadriver.sh')
        poolshot.shoot(4914, Path('/tmp/x.png'), runner=calls.append)
        self.assertEqual(calls, [argv])
        # The shim viadriver runs refuses on its own, before any capture: the
        # stub binary would leave a mark if it ran.
        with tempfile.TemporaryDirectory() as tmp:
            mark = Path(tmp) / 'ran'
            stub = Path(tmp) / 'screencapture'
            stub.write_text(f'#!/bin/sh\necho "$@" > {mark}\n')
            stub.chmod(0o755)
            env = dict(os.environ, POOLSHOT_SCREENCAPTURE=str(stub))
            for args in ([], ['', 'out.png'], ['0', 'out.png'], ['abc', 'out.png'], ['4914']):
                done = subprocess.run(['zsh', str(poolshot.GRAB), *args], env=env, capture_output=True)
                self.assertEqual(done.returncode, 64, args)
            self.assertFalse(mark.exists(), 'the shim ran a capture it should have refused')
            done = subprocess.run(['zsh', str(poolshot.GRAB), '4914', 'out.png'], env=env, capture_output=True)
            self.assertEqual(done.returncode, 0)
            self.assertEqual(mark.read_text().split(), ['-x', '-o', '-l', '4914', 'out.png'])


class FrameWait(unittest.TestCase):
    def test_frame_wait_parses_cr_terminated_lines(self):
        watch = poolshot.FrameWatch()
        self.assertEqual(watch.feed(b'BEGIN FRAME 909\r\n  x 1\r\nBEGIN FRAME 910\r'), 910)
        self.assertEqual(watch.last, 910)
        # A line the game is still writing is not a frame yet: `91` must not
        # stand for 91, nor `911` for the frame the next byte might make 9110.
        watch.feed(b'\nBEGIN FRAME 91')
        self.assertEqual(watch.last, 910)
        watch.feed(b'1\r')
        self.assertEqual(watch.last, 911)
        self.assertTrue(watch.reached(911))
        self.assertFalse(watch.reached(912))
        # CR alone, LF alone and CRLF all end a line.
        self.assertEqual(poolshot.FrameWatch().feed(b'BEGIN FRAME 5\rBEGIN FRAME 6\nBEGIN FRAME 7\r\n'), 7)
        self.assertIsNone(poolshot.FrameWatch().feed(b'  BEGIN UNITDATA\r\nEND FRAME 3\r'))


if __name__ == '__main__':
    unittest.main()
