"""`runqueue.sh` answers a TERM while its capture runs (parked 937).

The runner ran each capture as a foreground pipeline, and zsh holds a trap
until the foreground job ends: a TERM sent to a runner whose take had died in
`waitwin.sh` waited behind the very thing it was sent to stop (run314's first
take, item 923). The capture runs in the background now and the runner
`wait`s on it, which a signal interrupts.

The capture here is a stub, handed over through `RUNQUEUE_CAPTURE`, that
writes its pid and sleeps a minute. With the override set the runner's
cleanup kills its own descendants and nothing by name, so a gate that runs
this beside a real capture on the other lane leaves that capture alone.

**The runner reads the scenario it was started on** (parked 984, the
eighteenth pass). The stanza loop read `captures.txt` through an open
descriptor, so a stanza appended while a capture ran was taken in the same
run: run349 was, unplanned, and a stanza written ahead of its lane check
could launch before the check was done. The runner copies the file first.

**A capture puts the lobby back** (parked 987). `mapstyle.py` writes the
profile and `check.ini` on every capture and `longtrace.sh` restored them
only under `PROFILE` or `CHECKINI`, so the last map outlived its capture:
item 972 restored the held-out map's by hand.
"""
import re
import os
import signal
import subprocess
import tempfile
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'tools/gamelog/runqueue.sh'

STANZA = '''run: 99999937
tag: fixture-{pid}
item: 937
why: a stub that sleeps
frames: 10
mapstyle: 14
'''


def alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


class Runqueue(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.pidfile = self.root / 'capture.pid'
        self.capture = self.root / 'capture.sh'
        self.capture.write_text(f'#!/bin/zsh\necho $$ > {self.pidfile}\nsleep 60\n')
        self.scenario = self.root / 'captures.txt'
        self.scenario.write_text(STANZA.format(pid=os.getpid()))
        self.stub = None

    def tearDown(self):
        if self.stub and alive(self.stub):
            os.kill(self.stub, signal.SIGKILL)
        self.tmp.cleanup()

    def run_stub(self, body):
        self.capture.write_text(f'#!/bin/zsh\n{body}\n')
        env = dict(os.environ, RUNQUEUE_CAPTURE=str(self.capture),
                   RON_TMP=str(self.root / 'runs'))
        return subprocess.run(['zsh', str(SCRIPT), str(self.scenario)],
                              capture_output=True, text=True, env=env, timeout=30)

    def test_a_failed_capture_is_still_summarised_as_failed(self):
        # The background subshell must hand back the capture's status, not
        # tee's: the trap `set -o pipefail` was put there to close.
        done = self.run_stub('echo taking; exit 7')
        self.assertIn('CAPTURE FAILED (rc=7)', done.stdout, done.stdout + done.stderr)

    def test_a_finished_capture_is_summarised_as_captured(self):
        done = self.run_stub('echo taking; exit 0')
        self.assertIn('captured, checks ok', done.stdout, done.stdout + done.stderr)

    def test_a_stanza_appended_mid_run_waits_for_the_next_run(self):
        ran = self.root / 'ran.txt'
        # The stanza is flushed by its blank line, with the file still open
        # behind it: a stanza flushed by EOF has nothing left to read.
        self.scenario.write_text(STANZA.format(pid=os.getpid()) + '\n# the file goes on\n')
        late = ('\nrun: 99999938\ntag: late\nitem: 937\nwhy: appended mid-run\n'
                'frames: 10\nmapstyle: 14\n')
        done = self.run_stub(
            f'echo "run$1" >> {ran}\n'
            f'if [ ! -f {self.root}/appended ]; then\n'
            f'  touch {self.root}/appended\n'
            f'  printf {late!r} >> {self.scenario}\n'
            f'fi\nexit 0')
        self.assertEqual(ran.read_text().split(), ['run99999937'], done.stdout + done.stderr)
        self.assertIn('run: 99999938', self.scenario.read_text())

    def test_a_term_ends_the_runner_and_its_capture_at_once(self):
        env = dict(os.environ, RUNQUEUE_CAPTURE=str(self.capture),
                   RON_TMP=str(self.root / 'runs'))
        runner = subprocess.Popen(['zsh', str(SCRIPT), str(self.scenario)],
                                  stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                  text=True, env=env)
        try:
            deadline = time.time() + 10
            while time.time() < deadline and not self.pidfile.exists():
                time.sleep(0.1)
            self.assertTrue(self.pidfile.exists(), 'the stub capture never started')
            time.sleep(0.2)
            self.stub = int(self.pidfile.read_text())
            runner.send_signal(signal.SIGTERM)
            try:
                runner.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.fail('the runner was still there 5 s after its TERM: the trap '
                          'waits behind the foreground pipeline (parked 937)')
            deadline = time.time() + 5
            while time.time() < deadline and alive(self.stub):
                time.sleep(0.1)
            self.assertFalse(alive(self.stub), 'the capture outlived its runner')
        finally:
            if runner.poll() is None:
                runner.kill()
                runner.wait()
            runner.stdout.close()


class TheLobbyIsPutBack(unittest.TestCase):
    def code(self):
        text = (ROOT / 'tools/gamelog/longtrace.sh').read_text()
        return [(i + 1, l) for i, l in enumerate(text.split('\n'))
                if not l.lstrip().startswith('#')]

    def test_the_lobby_is_saved_before_the_first_write_to_it(self):
        code = self.code()
        save = next((i for i, l in code if re.match(r'\s*save_lobby\b', l)), None)
        write = next(i for i, l in code if 'mapstyle.py' in l)
        self.assertIsNotNone(save, 'longtrace.sh never saves the lobby')
        self.assertLess(save, write)

    def test_every_restore_of_the_inis_restores_the_lobby(self):
        code = self.code()
        restores = [n for n, (i, l) in enumerate(code) if 'window.py" restore' in l]
        self.assertGreaterEqual(len(restores), 2)
        for n in restores:
            self.assertRegex(code[n + 1][1], r'^\s*restore_lobby\b',
                             f'line {code[n][0]} restores the INIs and not the lobby')

    def test_the_save_is_not_conditional(self):
        body = (ROOT / 'tools/gamelog/longtrace.sh').read_text()
        fn = re.search(r'^save_lobby \(\) \{\n(.*?)^\}', body, re.S | re.M)
        self.assertIsNotNone(fn)
        self.assertNotIn('if ', fn.group(1))
        self.assertIn('Player.dat', fn.group(1))
        self.assertIn('check.ini', fn.group(1))


if __name__ == '__main__':
    unittest.main()
