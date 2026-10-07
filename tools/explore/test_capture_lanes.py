"""Two click-free capture lanes (parked 1139, item 1567).

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
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/explore'))
import live_session  # noqa: E402
import unattended_capture as runner  # noqa: E402

HOME = Path.home()


def shell(script, **env):
    full = {k: v for k, v in os.environ.items()
            if k not in ('RON_CAPTURE_LANE', 'RON_WINEPREFIX', 'RON_LANE_LOCK', 'RON_INSTALL', 'RON_PROFILE')}
    full.update(env)
    return subprocess.run(['zsh', '-c', script], capture_output=True, text=True, env=full)


class TheLaneTable(unittest.TestCase):
    def test_lane_two_is_its_own_prefix_and_lock(self):
        launch = f'source {ROOT}/tools/gamelog/winelaunch.sh; print -r -- $RON_WINEPREFIX; print -r -- $RON_LANE_LOCK'
        one = shell(launch).stdout.split('\n')
        two = shell(launch, RON_CAPTURE_LANE='2').stdout.split('\n')
        self.assertEqual(one[:2], [f'{HOME}/wine-ron', f'{HOME}/wine-ron/.lane.lock'])
        self.assertEqual(two[:2], [f'{HOME}/wine-ron-2', f'{HOME}/wine-ron-2/.lane.lock'])

    def test_a_hand_set_prefix_still_wins(self):
        # The lock tests set it; a lane must not move them onto a real prefix.
        out = shell(f'source {ROOT}/tools/gamelog/winelaunch.sh; print -r -- $RON_WINEPREFIX',
                    RON_CAPTURE_LANE='2', RON_WINEPREFIX='/tmp/x').stdout
        self.assertEqual(out.strip(), '/tmp/x')

    def test_an_unknown_lane_refuses(self):
        done = shell(f'source {ROOT}/tools/gamelog/winelaunch.sh || exit $?; print launched',
                     RON_CAPTURE_LANE='3')
        self.assertEqual(done.returncode, 64)
        self.assertNotIn('launched', done.stdout)
        with self.assertRaises(ValueError):
            runner.lane_paths(dict(os.environ, RON_CAPTURE_LANE='3'))

    def test_lane_two_ignores_a_lane_one_install_and_profile(self):
        env = dict(os.environ, RON_CAPTURE_LANE='2', RON_INSTALL='/lane1/game', RON_PROFILE='/lane1/profile')
        paths = runner.lane_paths(env)
        self.assertEqual(paths['lane'], '2')
        self.assertEqual(paths['install'], HOME / 'ron-capture-lane-2/game')
        self.assertEqual(paths['profile'],
                         HOME / 'ron-capture-lane-2/AppData/Roaming/Microsoft Games/Rise of Nations')
        self.assertEqual(paths['prefixes'], [f'{HOME}/wine-ron', f'{HOME}/wine-ron-2'])
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
