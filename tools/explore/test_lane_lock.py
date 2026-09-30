"""The launch line's lane lock, and the wait on its release (parked 758).

`tools/gamelog/winelaunch.sh`'s `ron_wine` refuses a second game while the
lock's pid is alive, and takes a stale lock over silently. Nothing waited on
a live one: item 742 chained its capture behind a hand-rolled `kill -0` loop
on another lane's pid. `RON_LANE_WAIT=<seconds>` is the wait, and these run
the function against `/usr/bin/true` with an orphaned `sleep` as the holder.

**The lane is taken before anything it shares is written** (parked 974, the
eighteenth pass). `longtrace.sh` rewrote the profile and removed
`Logs/gamelog.txt` and `rontrace.log` before `ron_wine` looked at the lock,
so a second lane launching into a running capture cleared that game's log
and was refused only afterwards. `ron_lane_take` is the lock taken by the
capture *script*, on its own pid, before its first write; `ron_wine` then
puts the game's pid beside it, and the lane is held while either lives —
which also covers the minute after the game exits in which the script is
still archiving the log. `TakeFirst` fails on the older launch line, and
`EveryScriptTakesFirst` on a capture script that writes before it takes.
"""
import os
import re
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


class TakeFirst(LaneLock):
    def shell(self, body, **extra):
        env = dict(os.environ, RON_WINE_BIN='/usr/bin/true', RON_WINEPREFIX=str(self.root),
                   RON_LANE_LOCK=str(self.lock), RON_LANE_HOLDER='test')
        env.pop('RON_LANE_WAIT', None)
        env.pop('RON_LANE_FORCE', None)
        env.update(extra)
        return subprocess.run(['zsh', '-c', f'source {SCRIPT}; {body}'],
                              env=env, capture_output=True, text=True, timeout=60)

    def test_a_take_refuses_a_live_lock_and_writes_nothing(self):
        pid = self.hold(20)
        before = self.lock.read_text()
        run = self.shell('ron_lane_take; echo "rc=$?"')
        self.assertIn('rc=75', run.stdout, run.stderr)
        self.assertIn(f'pid {pid}', run.stderr)
        self.assertEqual(self.lock.read_text(), before)

    def test_the_taker_launches_its_own_game(self):
        run = self.shell(f'ron_lane_take && ron_wine {self.log}; echo "rc=$?"')
        self.assertIn('rc=0', run.stdout, run.stderr)

    def test_a_taken_lane_refuses_another_script(self):
        holder = subprocess.Popen(
            ['zsh', '-c', f'source {SCRIPT}; ron_lane_take; echo taken; sleep 20'],
            env=dict(os.environ, RON_WINE_BIN='/usr/bin/true', RON_WINEPREFIX=str(self.root),
                     RON_LANE_LOCK=str(self.lock), RON_LANE_HOLDER='the first lane'),
            stdout=subprocess.PIPE, text=True)
        self.holders.append(holder.pid)
        try:
            self.assertEqual(holder.stdout.readline().strip(), 'taken')
            rc, err, took = self.launch()
            self.assertEqual(rc, 75)
            self.assertIn('the first lane', err)
            self.assertLess(took, 5)
        finally:
            holder.kill()
            holder.wait()
        rc, _, _ = self.launch()
        self.assertEqual(rc, 0)

    def test_a_script_still_archiving_holds_the_lane(self):
        # The game is dead and its script is not: the log is being moved.
        game = self.hold(20)
        os.kill(game, 9)
        for _ in range(50):
            if not alive(game):
                break
            time.sleep(0.1)
        script = self.hold(20)
        self.lock.write_text(f'{game}\nthe first lane since now\n{script}\n')
        rc, err, _ = self.launch()
        self.assertEqual(rc, 75)
        self.assertIn(f'pid {script}', err)


class Liveness(LaneLock):
    """A pid is a number the kernel hands out again (parked 1180, the
    twentieth pass). run426's lock named pid 15593; the game had exited and
    an unrelated `next-server` held the number, so `kill -0` said the lane
    was held and run428 was refused with no wine running. The lock carries
    each pid's start time now, and a pid that lives under another start
    time is dead for the lane. And a lock whose pids were both dead read as
    a busy lane to a worker who looked at the file, six times in one
    tranche: `ron_lane_state` says which it is."""

    def shell(self, body, **extra):
        env = dict(os.environ, RON_WINE_BIN='/usr/bin/true', RON_WINEPREFIX=str(self.root),
                   RON_LANE_LOCK=str(self.lock), RON_LANE_HOLDER='test')
        env.pop('RON_LANE_WAIT', None)
        env.pop('RON_LANE_FORCE', None)
        env.update(extra)
        return subprocess.run(['zsh', '-c', f'source {SCRIPT}; {body}'],
                              env=env, capture_output=True, text=True, timeout=60)

    def started(self, pid):
        return subprocess.run(['ps', '-o', 'lstart=', '-p', str(pid)],
                              capture_output=True, text=True).stdout.strip()

    def test_a_recycled_pid_is_a_stale_lock(self):
        pid = self.hold(20)
        self.lock.write_text(f'{pid}\nrun426 since then\nstamp {pid} Mon Jan  1 00:00:00 2001\n')
        rc, err, _ = self.launch()
        self.assertEqual(rc, 0, err)
        self.assertNotEqual(self.holder_pid(), str(pid))

    def test_a_live_pid_under_its_own_stamp_holds(self):
        pid = self.hold(20)
        self.lock.write_text(f'{pid}\nrun426 since then\nstamp {pid} {self.started(pid)}\n')
        rc, err, _ = self.launch()
        self.assertEqual(rc, 75)
        self.assertIn(f'pid {pid}', err)

    def test_a_launch_stamps_what_it_writes(self):
        holder = subprocess.Popen(
            ['zsh', '-c', f'source {SCRIPT}; ron_lane_take; echo taken; sleep 20'],
            env=dict(os.environ, RON_WINE_BIN='/usr/bin/true', RON_WINEPREFIX=str(self.root),
                     RON_LANE_LOCK=str(self.lock), RON_LANE_HOLDER='the first lane'),
            stdout=subprocess.PIPE, text=True)
        self.holders.append(holder.pid)
        try:
            self.assertEqual(holder.stdout.readline().strip(), 'taken')
            text = self.lock.read_text()
            self.assertIn(f'stamp {holder.pid} {self.started(holder.pid)}', text)
            # The first three lines are what they were: a reader by line
            # number (`sed -n 1p`, `2p`, `3p`) is not moved by the stamps.
            self.assertEqual(text.split('\n')[0], str(holder.pid))
            self.assertEqual(text.split('\n')[2], str(holder.pid))
        finally:
            holder.kill()
            holder.wait()

    def test_the_state_of_a_free_lane(self):
        run = self.shell('ron_lane_state')
        self.assertEqual(run.stdout.strip(), 'free')

    def test_a_take_for_another_pid_holds_the_lane_for_it(self):
        # Parked 1234 (three reaches): the click-free runner launches
        # through a child shell, so a take on the shell's own pid was dead
        # the moment the shell exited, the lane read `stale` while the
        # runner's `finally` was still restoring the profile, and another
        # lane's long trace launched into the restore. A take names the pid
        # the lane is held for; the runner's launches carry it in
        # `RON_LANE_TAKEN` and go; another shell's refuse; and the runner
        # releases what it took, so the lane reads `free` and not `stale`.
        run = subprocess.run(['sh', '-c', 'sleep 30 >/dev/null 2>&1 </dev/null & echo $!'],
                             capture_output=True, text=True, check=True)
        pid = int(run.stdout.strip())
        self.holders.append(pid)
        run = self.shell(f'ron_lane_take {pid}; echo "rc=$?"; ron_lane_state')
        self.assertIn('rc=0', run.stdout, run.stdout + run.stderr)
        self.assertIn('held by test since', run.stdout)
        self.assertIn(f'(pid {pid}, sleep)', run.stdout)
        self.assertEqual(self.lock.read_text().split('\n')[2], str(pid))
        rc, err, _ = self.launch()
        self.assertEqual(rc, 75, err)
        rc, err, _ = self.launch(RON_LANE_TAKEN=str(pid))
        self.assertEqual(rc, 0, err)
        self.assertEqual(self.lock.read_text().split('\n')[2], str(pid))
        run = self.shell(f'ron_lane_release {pid}; echo "rc=$?"; ron_lane_state')
        self.assertIn('rc=0', run.stdout, run.stdout + run.stderr)
        self.assertEqual(run.stdout.strip().split('\n')[-1], 'free')
        self.assertFalse(self.lock.exists())

    def test_a_release_by_another_pid_leaves_the_lock(self):
        pid = self.hold(20)
        run = self.shell(f'ron_lane_release {pid + 100000}; echo "rc=$?"; ron_lane_state')
        self.assertIn('rc=1\n', run.stdout, run.stdout + run.stderr)
        self.assertTrue(self.lock.exists())

    def test_the_state_of_a_dead_holder(self):
        pid = self.hold(20)
        os.kill(pid, 9)
        for _ in range(50):
            if not alive(pid):
                break
            time.sleep(0.1)
        run = self.shell('ron_lane_state')
        self.assertTrue(run.stdout.startswith('stale'), run.stdout + run.stderr)
        self.assertIn(str(pid), run.stdout)

    def test_the_state_of_a_recycled_pid(self):
        pid = self.hold(20)
        self.lock.write_text(f'{pid}\nrun426 since then\nstamp {pid} Mon Jan  1 00:00:00 2001\n')
        run = self.shell('ron_lane_state')
        self.assertTrue(run.stdout.startswith('stale'), run.stdout + run.stderr)
        self.assertIn('recycled', run.stdout)

    def test_the_state_of_a_held_lane(self):
        pid = self.hold(20)
        run = self.shell('ron_lane_state')
        self.assertTrue(run.stdout.startswith('held'), run.stdout + run.stderr)
        self.assertIn(f'pid {pid}', run.stdout)
        self.assertIn('test since now', run.stdout)


WRITES = re.compile(
    r'perm_probe |mapstyle\.py|profile\.py|checkini\.py|seedini\.py|setlog\.py'
    r'|window\.py" (?:stage|frames)|> "\$G/|rm -f "\$G')


class EveryScriptTakesFirst(unittest.TestCase):
    def scripts(self):
        found = []
        for d in ('tools/gamelog', 'tools/fuzz'):
            for f in sorted((ROOT / d).glob('*.sh')):
                if f.name != 'winelaunch.sh' and re.search(r'^\s*ron_wine ', f.read_text(), re.M):
                    found.append(f)
        return found

    def test_the_scripts_are_found(self):
        self.assertIn('longtrace.sh', [f.name for f in self.scripts()])
        self.assertGreaterEqual(len(self.scripts()), 8)

    def test_no_script_writes_before_it_takes(self):
        late = []
        for f in self.scripts():
            code = [(i + 1, l) for i, l in enumerate(f.read_text().split('\n'))
                    if not l.lstrip().startswith('#')]
            take = next((i for i, l in code if re.match(r'\s*ron_lane_take\b', l)), None)
            write = next((i for i, l in code if WRITES.search(l)), None)
            if take is None or (write is not None and write < take):
                late.append(f'{f.relative_to(ROOT)}: takes at {take}, first write at {write}')
        self.assertEqual(late, [])


if __name__ == '__main__':
    unittest.main()
