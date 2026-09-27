"""`waitwin.sh` ends when the game does (parked 937).

The waiter had one way out, a window, so a game that died before its window
held the capture script, the queue runner and the lane until four pids were
killed by hand (run314's first take, item 923). These run it against a
fixture process under a name of its own and a focus script that never
answers: the game that dies is exit 3, the launch that starts none exit 4,
the game that lives windowless exit 5. Before the change each of the three
ran to the test's deadline.
"""
import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'tools/gamelog/waitwin.sh'
DEADLINE = 15


class Waitwin(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        # A name of its own: the gate may run beside a real capture, and the
        # stock pattern would find that game.
        self.name = f'waitwin-fixture-{os.getpid()}.exe'
        self.game = self.root / self.name
        self.game.write_text('#!/bin/sh\nsleep "$1"\n')
        self.game.chmod(0o755)
        self.focus = self.root / 'focus.sh'
        self.focus.write_text('#!/bin/zsh\nexit 0\n')
        self.children = []

    def tearDown(self):
        for c in self.children:
            c.kill()
            c.wait()
        self.tmp.cleanup()

    def start_game(self, seconds):
        c = subprocess.Popen(['/bin/sh', str(self.game), str(seconds)])
        self.children.append(c)
        return c

    def wait(self, **env):
        base = dict(os.environ,
                    WAITWIN_PATTERN=self.name.replace('.', r'\.'),
                    WAITWIN_FOCUS=str(self.focus),
                    WAITWIN_POLL='0.3')
        base.update(env)
        began = time.time()
        try:
            done = subprocess.run(['zsh', str(SCRIPT), str(self.root / 'shot.png')],
                                  capture_output=True, text=True, env=base,
                                  timeout=DEADLINE)
        except subprocess.TimeoutExpired:
            self.fail(f'waitwin.sh was still waiting after {DEADLINE} s: '
                      'a wait with no way out but a window (parked 937)')
        return done, time.time() - began

    def test_a_game_that_dies_before_its_window_ends_the_wait(self):
        self.start_game(2)
        done, took = self.wait()
        self.assertEqual(done.returncode, 3, done.stderr)
        self.assertIn('died before its window', done.stderr)
        self.assertLess(took, 10)

    def test_a_launch_that_starts_no_game_ends_the_wait(self):
        done, _ = self.wait(WAITWIN_START_MAX='1')
        self.assertEqual(done.returncode, 4, done.stderr)
        self.assertIn('no game process', done.stderr)

    def test_a_live_game_with_no_window_ends_the_wait_and_says_which(self):
        self.start_game(30)
        done, _ = self.wait(WAITWIN_MAX='2')
        self.assertEqual(done.returncode, 5, done.stderr)
        self.assertIn('no window answered', done.stderr)


if __name__ == '__main__':
    unittest.main()
