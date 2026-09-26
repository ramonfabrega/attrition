"""The launch line's lane lock, and the wait on its release (parked 758).

`tools/gamelog/winelaunch.sh`'s `ron_wine` refuses a second game while the
lock's pid is alive, and takes a stale lock over silently. Nothing waited on
a live one: item 742 chained its capture behind a hand-rolled `kill -0` loop
on another lane's pid. `RON_LANE_WAIT=<seconds>` is the wait, and these run
the function against `/usr/bin/true` with an orphaned `sleep` as the holder.
"""
import os
import subprocess
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'tools/gamelog/winelaunch.sh'


def alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


class LaneLock(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.lock = self.root / '.lane.lock'
        self.log = self.root / 'wine.log'
        self.holders = []

    def tearDown(self):
        for pid in self.holders:
            try:
                os.kill(pid, 9)
            except ProcessLookupError:
                pass
        self.tmp.cleanup()

    def hold(self, seconds):
        # An orphan, not a child: a child that has exited is a zombie until
        # it is waited for, and `kill -0` still says it is alive.
        # The sleep lets go of the pipe, or `run` waits for it instead of `sh`.
        run = subprocess.run(['sh', '-c', f'sleep {seconds} >/dev/null 2>&1 </dev/null & echo $!'],
                             capture_output=True, text=True, check=True)
        pid = int(run.stdout.strip())
        self.holders.append(pid)
        self.lock.write_text(f'{pid}\ntest since now\n')
        return pid

    def holder_pid(self):
        return self.lock.read_text().split('\n')[0]

    def launch(self, **extra):
        env = dict(os.environ, RON_WINE_BIN='/usr/bin/true', RON_WINEPREFIX=str(self.root),
                   RON_LANE_LOCK=str(self.lock), RON_LANE_HOLDER='test')
        env.pop('RON_LANE_WAIT', None)
        env.pop('RON_LANE_FORCE', None)
        env.update(extra)
        started = time.monotonic()
        run = subprocess.run(['zsh', '-c', f'source {SCRIPT}; ron_wine {self.log}; echo "rc=$?"'],
                             env=env, capture_output=True, text=True, timeout=60)
        rc = int(run.stdout.strip().rsplit('rc=', 1)[1])
        return rc, run.stderr, time.monotonic() - started

    def test_a_live_lock_refuses_at_once(self):
        pid = self.hold(20)
        rc, err, took = self.launch()
        self.assertEqual(rc, 75)
        self.assertIn(f'pid {pid}', err)
        self.assertLess(took, 5)
        self.assertEqual(self.holder_pid(), str(pid))

    def test_a_stale_lock_is_taken_over(self):
        pid = self.hold(20)
        os.kill(pid, 9)
        for _ in range(50):
            if not alive(pid):
                break
            time.sleep(0.1)
        self.assertFalse(alive(pid))
        rc, _, _ = self.launch()
        self.assertEqual(rc, 0)
        self.assertNotEqual(self.holder_pid(), str(pid))

    def test_the_wait_outlives_the_holder(self):
        pid = self.hold(2)
        rc, err, took = self.launch(RON_LANE_WAIT='15')
        self.assertEqual(rc, 0, err)
        self.assertIn('waiting', err)
        self.assertLess(took, 12)
        self.assertNotEqual(self.holder_pid(), str(pid))

    def test_the_wait_expires_on_a_live_holder(self):
        pid = self.hold(20)
        rc, err, took = self.launch(RON_LANE_WAIT='2')
        self.assertEqual(rc, 75)
        self.assertIn(f'pid {pid}', err)
        self.assertGreaterEqual(took, 2)
        self.assertEqual(self.holder_pid(), str(pid))


if __name__ == '__main__':
    unittest.main()
