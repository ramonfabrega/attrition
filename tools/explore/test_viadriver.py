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

Since 2026-11-16 (about 13:00 -05:00) this host SIGKILLs a shell script
exec'd as a bundle's main executable: a direct exec exits 137, `open -n -a`
exits 0 and the script never runs, an ad-hoc `codesign -s -` does not help,
and the system log is silent. A compiled Mach-O in the same bundle runs both
ways, and the real RonDriver.app is one. So the script still holds the logic
but lives at `Contents/Resources/fixture.sh`, and `Contents/MacOS/RonDriver`
is a C stub compiled in `setUp` (`xcrun clang`) that execs `/bin/sh` with the
script and its own argv[1..], so the script sees the same `$1 $2 ...` as
before. No compiler is a skip that says so, never a quiet green.
"""
import os
import plistlib
import re
import shutil
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

# The bundle's executable: exec /bin/sh on the script, argv[1..] passed on.
STUB = r'''#include <stdlib.h>
#include <unistd.h>
int main(int argc, char **argv) {
    char **args = calloc((size_t)argc + 2, sizeof *args);
    if (!args) return 127;
    args[0] = "/bin/sh";
    args[1] = SCRIPT_PATH;
    for (int i = 1; i < argc; i++) args[i + 1] = argv[i];
    execv("/bin/sh", args);
    return 127;
}
'''


def _clang():
    # `xcrun clang`, not the path `xcrun --find clang` prints: run bare, that
    # binary has no SDK and finds no `stdlib.h`; through xcrun it does.
    if shutil.which('xcrun'):
        found = subprocess.run(['xcrun', '--find', 'clang'], capture_output=True, text=True)
        if found.returncode == 0 and found.stdout.strip():
            return ['xcrun', 'clang']
    bare = shutil.which('clang')
    return [bare] if bare else None


def _c_string(text):
    return '"' + text.replace('\\', '\\\\').replace('"', '\\"') + '"'


class Viadriver(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        clang = _clang()
        if clang is None:
            self.tmp.cleanup()
            self.skipTest('no C compiler (`xcrun --find clang` and `which clang` both '
                          'failed): the fixture bundle needs a compiled executable, since '
                          'this host kills a script bundle executable (2026-11-16)')
        app = self.root / 'Fixture.app'
        macos = app / 'Contents' / 'MacOS'
        macos.mkdir(parents=True)
        resources = app / 'Contents' / 'Resources'
        resources.mkdir()
        script = resources / 'fixture.sh'
        script.write_text(EXECUTABLE)
        script.chmod(0o755)
        source = self.root / 'stub.c'
        source.write_text(STUB)
        # viadriver.sh looks for `Contents/MacOS/RonDriver` by name.
        binary = macos / 'RonDriver'
        built = subprocess.run(
            [*clang, '-O2',f'-DSCRIPT_PATH={_c_string(str(script))}',
             '-o', str(binary), str(source)],
            capture_output=True, text=True)
        if built.returncode != 0:
            self.tmp.cleanup()
            self.fail(f'the fixture stub did not compile:\n{built.stderr}')
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
        # The stub execs /bin/sh, so a running instance's command line names
        # `Contents/Resources/fixture.sh`: match anything under the bundle.
        running = str(self.app / 'Contents') + '/'
        subprocess.run(['pkill', '-f', running], capture_output=True)
        # Wait for the killed instances to be gone, so a test's log holds
        # only its own launches.
        deadline = time.time() + 10
        while time.time() < deadline and subprocess.run(
                ['pgrep', '-f', running],
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
        # Loop 3638 (the fifty-seventh pass): the wait is on the CONTENT, not
        # on the file. The fixture's `echo "args: $*" >> "$log"` creates the
        # log and then writes it, and twice in one tranche a loaded host read
        # it in between — `'args: ' not found in ''` — and cost a lane gate's
        # whole re-run; the deadline is twice the sibling's for the same load.
        # Loop 3656 (the fifty-eighth pass): the twenty seconds flaked twice
        # more in one lane's five gates, `'args: ' not found in ''` on the
        # FIRST launch's log each time and green six of six alone — so the
        # open question is whether a loaded host writes the log late or not
        # at all. The wait is sixty seconds now, and the failure says when
        # each line arrived, so the next flake answers that instead of
        # costing a lane gate's re-run.
        started = time.time()
        deadline = started + 60
        arrived = {}
        while time.time() < deadline and len(arrived) < 2:
            for name, log in (('first', first), ('second', second)):
                if name not in arrived and self._args_line(log):
                    arrived[name] = round(time.time() - started, 1)
            if len(arrived) < 2:
                time.sleep(0.2)
        report = (f'args lines arrived (seconds after the second launch): {arrived}; '
                  f'past the old 20 s: {[n for n, at in arrived.items() if at > 20]}')
        self.assertIn('args: ', first.read_text() if first.exists() else '', report)
        text = second.read_text() if second.exists() else ''
        self.assertIn('args: ', text,
                      'the second launch wrote no log: `open` without `-n` activated the '
                      'running instance and dropped the arguments (parked 810); ' + report)
        self.assertIn(str(second), text)

    @staticmethod
    def _args_line(log):
        return 'args: ' in (log.read_text() if log.exists() else '')

    def test_two_launches_in_one_second_have_two_logs(self):
        # Item 1568: the log was named by the second, so two capture lanes
        # started together shared one, and each waiter read both receipts.
        # `date` is pinned so the two launches share a second every time.
        bin_ = self.root / 'bin'; bin_.mkdir()
        (bin_ / 'date').write_text('#!/bin/sh\necho 20261007-002635\n'); (bin_ / 'date').chmod(0o755)
        path = f"{bin_}:{os.environ['PATH']}"
        first, second = self.launch(PATH=path), self.launch(PATH=path)
        self.assertIn('20261007-002635', first.name)
        self.assertNotEqual(first, second)

    def test_the_capture_lane_reaches_the_runner(self):
        # Parked 1139, item 1568: the lane is in the launch's argv — the
        # log's `args` line — and in the spawned program's environment.
        log = self.launch(RON_CAPTURE_LANE='2')
        deadline = time.time() + 10
        while time.time() < deadline and 'lane: ' not in (log.read_text() if log.exists() else ''):
            time.sleep(0.2)
        text = log.read_text() if log.exists() else ''
        self.assertIn('lane: 2', text)
        self.assertIn('/usr/bin/env RON_CAPTURE_LANE=2 /bin/zsh', text)


if __name__ == '__main__':
    unittest.main()
