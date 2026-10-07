"""The click-free capture lanes, and their pool (parked 1139, items 1568, 1569).

The lane's prefix, its install copy and its profile are each a singleton a
capture writes, so a second lane is a second of each, chosen by
`RON_CAPTURE_LANE` (`tools/gamelog/lanes.sh`). What fails without the
second lane: `winelaunch.sh` launched every game into `~/wine-ron`;
`live_session.require_closed` refused while **any** game ran, so a lane-1
capture could never start beside a lane-2 one; and the runner would
stage a lane-2 capture into whatever profile it was handed. Never
launches a game: the closed-game check reads a canned `ps` and `lsof`.
"""
import os
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/explore'))
import live_session  # noqa: E402
import unattended_capture as runner  # noqa: E402
sys.path.insert(0, str(ROOT / 'tools/gamelog'))
import lanes  # noqa: E402

HOME = Path.home()


def shell(script, **env):
    full = {k: v for k, v in os.environ.items()
            if k not in ('RON_CAPTURE_LANE', 'RON_WINEPREFIX', 'RON_LANE_LOCK', 'RON_INSTALL', 'RON_PROFILE',
                         'RON_LANE_POOL', 'RON_LANES_ROOT', 'RON_LANES_MAX', 'RON_LANE_TAKEN')}
    full.update(env)
    return subprocess.run(['zsh', '-c', script], capture_output=True, text=True, env=full)


class TheLaneTable(unittest.TestCase):
    def test_each_lane_is_its_own_prefix_and_lock(self):
        launch = f'source {ROOT}/tools/gamelog/winelaunch.sh; print -r -- $RON_WINEPREFIX; print -r -- $RON_LANE_LOCK'
        one = shell(launch).stdout.split('\n')
        three = shell(launch, RON_CAPTURE_LANE='3').stdout.split('\n')
        self.assertEqual(one[:2], [f'{HOME}/wine-ron', f'{HOME}/wine-ron/.lane.lock'])
        self.assertEqual(three[:2], [f'{HOME}/wine-ron-3', f'{HOME}/wine-ron-3/.lane.lock'])

    def test_a_hand_set_prefix_still_wins(self):
        # The lock tests set it; a lane must not move them onto a real prefix.
        out = shell(f'source {ROOT}/tools/gamelog/winelaunch.sh; print -r -- $RON_WINEPREFIX',
                    RON_CAPTURE_LANE='2', RON_WINEPREFIX='/tmp/x').stdout
        self.assertEqual(out.strip(), '/tmp/x')

    def test_a_lane_past_the_cap_refuses(self):
        done = shell(f'source {ROOT}/tools/gamelog/winelaunch.sh || exit $?; print launched',
                     RON_CAPTURE_LANE='4')
        self.assertEqual(done.returncode, 64)
        self.assertNotIn('launched', done.stdout)
        with self.assertRaises(ValueError):
            runner.lane_paths(dict(os.environ, RON_CAPTURE_LANE='4'))
        self.assertEqual(runner.lane_paths(dict(os.environ, RON_CAPTURE_LANE='4', RON_LANES_MAX='4'))['lane'], '4')

    def test_lane_n_ignores_a_lane_one_install_and_profile(self):
        env = dict(os.environ, RON_CAPTURE_LANE='2', RON_INSTALL='/lane1/game', RON_PROFILE='/lane1/profile')
        paths = runner.lane_paths(env)
        self.assertEqual(paths['lane'], '2')
        self.assertEqual(paths['install'], HOME / 'ron-capture-lane-2/game')
        self.assertEqual(paths['profile'],
                         HOME / 'ron-capture-lane-2/AppData/Roaming/Microsoft Games/Rise of Nations')
        self.assertEqual(paths['prefixes'], [f'{HOME}/wine-ron', f'{HOME}/wine-ron-2', f'{HOME}/wine-ron-3'])
        self.assertEqual(paths['max'], 3)
        one = runner.lane_paths(dict(os.environ, RON_INSTALL='/lane1/game'))
        self.assertEqual((one['lane'], one['install']), ('1', Path('/lane1/game')))

    def test_the_runner_refuses_another_lane_s_profile_before_writing(self):
        with tempfile.TemporaryDirectory() as tmp:
            tmp = Path(tmp)
            (tmp / 'install').mkdir(); (tmp / 'profile').mkdir()
            env = dict(os.environ, RON_CAPTURE_LANE='2')
            done = subprocess.run([sys.executable, str(ROOT / 'tools/explore/unattended_capture.py'),
                                   str(tmp / 'install'), str(tmp / 'out'), str(tmp / 'profile')],
                                  capture_output=True, text=True, env=env)
            self.assertEqual(done.returncode, 2, done.stderr)
            self.assertIn('capture lane 2 runs on its own install', done.stderr)
            self.assertFalse((tmp / 'out').exists())


class ThePool(unittest.TestCase):
    """No caller names a lane (item 1569): the runner takes the first free
    one, grows one under the cap, waits at it. Lanes here are directories
    under a temporary `RON_LANES_ROOT`; `ron_wine` is never called."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        env = patch.dict(os.environ, {'RON_LANES_ROOT': str(self.root), 'RON_LANES_MAX': '2',
                                      'RON_WINE_BIN': '/usr/bin/true', 'RON_INSTALL': str(self.root / 'game')})
        env.start(); self.addCleanup(env.stop)
        for k in ('RON_CAPTURE_LANE', 'RON_WINEPREFIX', 'RON_LANE_LOCK', 'RON_LANE_TAKEN', 'RON_PROFILE',
                  'RON_LANE_POOL'):
            os.environ.pop(k, None)
        self.make(1); self.make(2)

    def make(self, n, admitted=True):
        paths = runner.lane_paths(runner.lane_env(n))
        Path(paths['prefix']).mkdir(parents=True, exist_ok=True)
        paths['profile'].mkdir(parents=True, exist_ok=True)
        if admitted and n > 1:
            (paths['home'] / 'admitted').write_text('a test\n')
        return paths

    def log(self, n):
        path = Path(runner.lane_paths(runner.lane_env(n))['prefix']) / '.lane.log'
        return path.read_text() if path.exists() else ''

    def test_two_runners_take_two_lanes_and_the_takes_are_logged(self):
        with runner.pool_lane(0) as a, runner.pool_lane(0) as b:
            self.assertEqual((a['lane'], b['lane']), ('1', '2'))
            state = shell(f'source {ROOT}/tools/gamelog/winelaunch.sh; ron_lane_state',
                          RON_LANES_ROOT=str(self.root), RON_CAPTURE_LANE='2').stdout
            self.assertIn('held by pool unattended_capture', state)
        self.assertRegex(self.log(1), r' take 1 \d+ pool\n.* release 1 \d+\n')
        self.assertRegex(self.log(2), r' take 2 \d+ pool\n.* release 2 \d+\n')

    def test_an_unadmitted_lane_is_never_taken(self):
        (self.root / 'ron-capture-lane-2/admitted').unlink()
        (self.root / 'ron-capture-lane-2/admission-failed').write_text('a test\n')
        with runner.pool_lane(0) as a:
            self.assertEqual(a['lane'], '1')
            with self.assertRaises(BlockingIOError):
                with runner.pool_lane(0, build=lambda n: self.fail(f'grew lane {n}')):
                    pass

    def test_the_pool_grows_under_the_cap_and_tries_a_failed_build_once(self):
        built = []
        def build(n):
            built.append(n)
            self.make(n)
            return True
        with patch.dict(os.environ, RON_LANES_MAX='3'), \
             runner.pool_lane(0) as a, runner.pool_lane(0) as b, runner.pool_lane(0, build=build) as c:
            self.assertEqual((a['lane'], b['lane'], c['lane'], built), ('1', '2', '3', [3]))
        shutil.rmtree(self.root / 'ron-capture-lane-3')
        failed = []
        with patch.dict(os.environ, RON_LANES_MAX='3'), runner.pool_lane(0), runner.pool_lane(0):
            with self.assertRaises(BlockingIOError):
                with runner.pool_lane(0, build=lambda n: failed.append(n) or False):
                    pass
        self.assertEqual(failed, [3])

    def test_at_the_cap_the_pool_waits_and_says_how_long(self):
        with runner.pool_lane(0) as a:
            # A second process's take is what a second runner is; a thread
            # shares this pid, so the lane is held for a sleeping child.
            sleeper = subprocess.Popen(['sleep', '30']); self.addCleanup(sleeper.kill)
            env = dict(runner.lane_env(2), RON_LANES_ROOT=str(self.root))
            self.assertEqual(runner.lane(f'ron_lane_take {sleeper.pid}', env).returncode, 0)
            # Reaped, or `kill -0` still finds the zombie and the lane stays held.
            threading.Timer(0.6, lambda: (sleeper.kill(), sleeper.wait())).start()
            with runner.pool_lane(5, poll=0.2) as b:
                self.assertEqual(b['lane'], '2')
        # Lane 0 is the pool's: the wait was for any lane.
        self.assertRegex(self.log(1), r' waited 0 \d+ \d+\n')


class TheQueueLaneAndThePool(unittest.TestCase):
    """The queue lane finds the game's window by title (item 1569): it never
    runs beside a pool game, and the pool never beside it."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = self.tmp.name
        for d in ('wine-ron', 'wine-ron-2'):
            Path(self.root, d).mkdir()
        self.sleeper = subprocess.Popen(['sleep', '30']); self.addCleanup(self.sleeper.kill)

    def take(self, lane, pid, pool):
        env = {'RON_LANES_ROOT': self.root, 'RON_CAPTURE_LANE': str(lane), 'RON_WINE_BIN': '/usr/bin/true'}
        if pool:
            env['RON_LANE_POOL'] = '1'
        return shell(f'source {ROOT}/tools/gamelog/winelaunch.sh; ron_lane_take {pid}', **env)

    def test_the_queue_lane_refuses_while_a_pool_lane_is_held(self):
        self.assertEqual(self.take(2, self.sleeper.pid, pool=True).returncode, 0)
        done = self.take(1, os.getpid(), pool=False)
        self.assertEqual(done.returncode, 75, done.stderr)
        self.assertIn('a pool lane is held', done.stderr)

    def test_a_pool_take_refuses_while_the_queue_lane_holds_lane_one(self):
        self.assertEqual(self.take(1, self.sleeper.pid, pool=False).returncode, 0)
        done = self.take(2, os.getpid(), pool=True)
        self.assertEqual(done.returncode, 75, done.stderr)
        self.assertIn('the queue lane holds', done.stderr)

    def test_pool_lanes_take_beside_each_other(self):
        self.assertEqual(self.take(1, self.sleeper.pid, pool=True).returncode, 0)
        self.assertEqual(self.take(2, os.getpid(), pool=True).returncode, 0)


class TheReader(unittest.TestCase):
    """`tools/gamelog/lanes.py`: the peak, the waits and the rate a pass
    moves the cap on (item 1569)."""

    def test_peak_waits_and_rate(self):
        lines = '\n'.join([
            '100 x take 1 11 pool', '110 x take 2 22 pool', '120 x take 9 99 queue',
            '130 x waited 1 33 300', '140 x release 2 22',
            '150 x capture 1 11 frames=300 launched=2026-10-07T00:00:00-0500 exited=2026-10-07T00:00:20-0500 success=true out=a',
            '160 x capture 2 22 frames=4341 launched=2026-10-07T00:00:00-0500 exited=2026-10-07T00:01:13-0500 success=true out=b',
            '170 x take 1 44 pool',  # lane 1's runner died: the take closes it
            '180 x capture 3 55 frames=10 launched=- exited=- out=c', 'garbage'])
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / '.lane.log'; log.write_text(lines + '\n')
            s = lanes.summarize(lanes.read([log], since=105))
        self.assertEqual(s['peak'], 1)
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / '.lane.log'; log.write_text(lines + '\n')
            s = lanes.summarize(lanes.read([log]))
        self.assertEqual((s['peak'], s['peak_at']), (2, 110))
        self.assertEqual(s['waits'], [300.0])
        self.assertEqual([round(x['rate'], 1) for x in s['rates']], [15.0, 59.5])
        self.assertAlmostEqual(s['median_rate'], (15 + 4341 / 73) / 2)


PS = '  501 riseofnations_trace.exe\n  502 zsh\n'


def fake(lsof):
    def run(command, **kwargs):
        out = PS if command[0] == 'ps' else lsof
        return subprocess.CompletedProcess(command, 0, stdout=out, stderr='')
    return run


class TheClosedGameIsTheLane_s(unittest.TestCase):
    PREFIXES = ['/h/wine-ron', '/h/wine-ron-2']

    def check(self, lsof, prefix):
        with patch.object(live_session.subprocess, 'run', side_effect=fake(lsof)):
            live_session.require_closed(prefix, self.PREFIXES)

    def test_another_lane_s_game_is_let_be(self):
        self.check('p501\nn/h/wine-ron-2/drive_c/windows/syswow64/d3d11.dll\n', '/h/wine-ron')
        self.check('p501\nn/h/wine-ron/drive_c/windows/syswow64/d3d11.dll\n', '/h/wine-ron-2')

    def test_this_lane_s_game_refuses(self):
        # `/h/wine-ron` is a string prefix of `/h/wine-ron-2`; the slash decides.
        for lsof, prefix in [('n/h/wine-ron/drive_c/windows/syswow64/d3d11.dll\n', '/h/wine-ron'),
                             ('n/h/wine-ron-2/drive_c/windows/syswow64/dxgi.dll\n', '/h/wine-ron-2')]:
            with self.subTest(prefix=prefix), self.assertRaises(RuntimeError):
                self.check(lsof, prefix)

    def test_a_game_no_lane_accounts_for_refuses(self):
        with self.assertRaises(RuntimeError):
            self.check('n/Applications/Steam/riseofnations.exe\n', '/h/wine-ron')

    def test_without_a_prefix_any_game_refuses(self):
        with self.assertRaises(RuntimeError):
            self.check('n/h/wine-ron-2/drive_c/windows/syswow64/d3d11.dll\n', None)


if __name__ == '__main__':
    unittest.main()
