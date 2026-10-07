"""`viadriver.sh` hands every launch its arguments, a running bundle or not (parked 810).

`open -a` on a bundle that is already running activates the running instance
and discards `--args`: a second lane's capture wrote no log, never reached
`winelaunch.sh`'s lane lock, and the commander sequenced the two by message
(item 800's run251 beside 803's run249). `open -n -a` starts a new instance,
which carries its arguments to the lock, where a second game is refused or
waits (`test_lane_lock.py`). This launches a fixture bundle twice, a second
apart, and reads both logs; before `-n`, the second was empty.

The fixture is a bundle whose executable is a shell script that appends its
argv to the log it is handed, runs the program it is handed as RonDriver
does, and sleeps long enough to still be running when the second launch
arrives. `LSBackgroundOnly` keeps it off the Dock.
"""
import os
import plistlib
import re
import subprocess
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'tools/gamelog/viadriver.sh'

EXECUTABLE = '''#!/bin/sh
log=$2
echo "args: $*" >> "$log"
shift 2
"$@" >> "$log" 2>&1
sleep 6
'''


class Viadriver(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        app = self.root / 'Fixture.app'
        macos = app / 'Contents' / 'MacOS'
        macos.mkdir(parents=True)
        # viadriver.sh looks for `Contents/MacOS/RonDriver` by name.
        binary = macos / 'RonDriver'
        binary.write_text(EXECUTABLE)
        binary.chmod(0o755)
        with (app / 'Contents' / 'Info.plist').open('wb') as f:
            plistlib.dump({
                'CFBundleExecutable': 'RonDriver',
                'CFBundleIdentifier': 'test.viadriver.fixture',
                'CFBundleName': 'RonDriver',
                'CFBundlePackageType': 'APPL',
                'LSBackgroundOnly': True,
            }, f)
        self.app = app
        self.noop = self.root / 'noop.sh'
        # What RonDriver spawns: here it says which lane it was handed.
        self.noop.write_text('#!/bin/sh\necho "lane: ${RON_CAPTURE_LANE:-unset}"\n')
        self.noop.chmod(0o755)

    def tearDown(self):
        subprocess.run(['pkill', '-f', str(self.app / 'Contents/MacOS/RonDriver')],
                       capture_output=True)
        # Wait for the killed instances to be gone, so a test's log holds
        # only its own launches.
        deadline = time.time() + 10
        while time.time() < deadline and subprocess.run(
                ['pgrep', '-f', str(self.app / 'Contents/MacOS/RonDriver')],
                capture_output=True).returncode == 0:
            time.sleep(0.2)
        self.tmp.cleanup()

    def launch(self, **extra):
        env = dict(os.environ, RONDRIVER_APP=str(self.app), **extra)
        env.pop('RON_CAPTURE_LANE', None) if 'RON_CAPTURE_LANE' not in extra else None
        done = subprocess.run(['zsh', str(SCRIPT), str(self.noop)],
                              capture_output=True, text=True, env=env)
        self.assertEqual(done.returncode, 0, done.stderr)
        log = re.search(r'^log: (.+)$', done.stdout, re.M)
        self.assertIsNotNone(log, done.stdout)
        return Path(log.group(1))

    def test_a_second_launch_while_the_first_runs_still_carries_its_arguments(self):
        first = self.launch()
        time.sleep(1)
        second = self.launch()
        deadline = time.time() + 10
        while time.time() < deadline and not (first.exists() and second.exists()):
            time.sleep(0.2)
        self.assertIn('args: ', first.read_text() if first.exists() else '')
        text = second.read_text() if second.exists() else ''
        self.assertIn('args: ', text,
                      'the second launch wrote no log: `open` without `-n` activated the '
                      'running instance and dropped the arguments (parked 810)')
        self.assertIn(str(second), text)

    def test_two_launches_in_one_second_have_two_logs(self):
        # Item 1567: the log was named by the second, so two capture lanes
        # started together shared one, and each waiter read both receipts.
        first, second = self.launch(), self.launch()
        self.assertNotEqual(first, second)

    def test_the_capture_lane_reaches_the_runner(self):
        # Parked 1139, item 1567: LaunchServices hands the bundle launchd's
        # environment, so `RON_CAPTURE_LANE=2` reached no runner without
        # `open --env`, and a lane-2 capture would have run on lane 1.
        log = self.launch(RON_CAPTURE_LANE='2')
        deadline = time.time() + 10
        while time.time() < deadline and 'lane: ' not in (log.read_text() if log.exists() else ''):
            time.sleep(0.2)
        self.assertIn('lane: 2', log.read_text() if log.exists() else '')


if __name__ == '__main__':
    unittest.main()
