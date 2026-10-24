"""`tools/gamelog/waitrun.sh` reads the log the verdict lands in (parked
656) — and a runner that died (the twenty-fifth pass, item 1503).

run659's first take printed a Python traceback from `autostart_receipt.py`
and no receipt, and the waiter's runner pattern (`pgrep -f`) matched the
other lane's runner, so the lane waited on a capture that was not its own.
These run the script on authored logs with a runner pattern nothing on the
box matches: a traceback with no receipt after it is exit 2 at once; a
traceback an earlier map printed, with a later receipt, is judged by the
receipt; the receipts keep their verdicts (the queue lane's banner left
with item 3485).
"""
import os
import signal
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'tools/gamelog/waitrun.sh'
TRACEBACK = ('Staged: /x/map-7\n'
             'Traceback (most recent call last):\n'
             '  File "unattended_capture.py", line 456, in <module>\n'
             "ValueError: extra lifecycle or fault records\n")
RECEIPT_OK = '{"map_requested": 7, "success": true}\n'
RECEIPT_BAD = '{"map_requested": 7, "success": false}\n'


class Waitrun(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.log = Path(self.tmp.name) / 'viadriver.log'

    def tearDown(self):
        self.tmp.cleanup()

    def run_it(self, text, runner=None):
        self.log.write_text(text)
        env = dict(os.environ, WAITRUN_RUNNER=runner or f'no-such-runner-{os.getpid()}')
        done = subprocess.run(['zsh', str(SCRIPT), str(self.log), '1'], env=env,
                              capture_output=True, text=True, timeout=30)
        return done.returncode, done.stdout, done.stderr

    def test_a_traceback_with_no_receipt_after_it_is_a_dead_runner(self):
        # The other lane's runner is alive — a fixture process whose command
        # line carries a name of its own — and the dead take is still
        # called at once; before the change the waiter sat until that
        # runner was gone.
        name = f'waitrun-fixture-runner-{os.getpid()}'
        runner = Path(self.tmp.name) / name
        runner.write_text('#!/bin/sh\nsleep 600\n')  # Loop 3546: lives until killed in finally
        runner.chmod(0o755)
        alive = subprocess.Popen([str(runner)], start_new_session=True)
        try:
            code, _, err = self.run_it(TRACEBACK, runner=name)
        finally:
            os.killpg(alive.pid, signal.SIGKILL)  # the sh and its sleep (Loop 3546)
            alive.wait()
        self.assertEqual(code, 2)
        self.assertIn('traceback and no receipt', err)
        self.assertIn('ValueError', err)

    def test_a_log_whose_every_staged_map_has_its_receipt_is_judged_while_another_runner_lives(self):
        # Loop 3063 (the fortieth tranche, item 3018): run913's receipt was
        # the log's last line at 17:03 and the waiter sat 81 minutes,
        # because the receipts are read only once no runner is alive and
        # `pgrep` saw run912's runner on the other pool lane. A log where
        # every `Staged: …/map-N` has its `"map_requested": N` receipt is
        # finished whatever else runs; one still owed a map keeps waiting.
        name = f'waitrun-fixture-runner-{os.getpid()}'
        runner = Path(self.tmp.name) / name
        runner.write_text('#!/bin/sh\nsleep 600\n')  # Loop 3546: lives until killed in finally
        runner.chmod(0o755)
        alive = subprocess.Popen([str(runner)], start_new_session=True)
        try:
            code, out, _ = self.run_it('Staged: /x/map-7\n' + RECEIPT_OK, runner=name)
            self.assertEqual(code, 0)
            self.assertIn('"map_requested": 7', out)
            self.assertEqual(self.run_it('Staged: /x/map-7\n' + RECEIPT_BAD, runner=name)[0], 1)
            self.log.write_text('Staged: /x/map-7\nStaged: /x/map-8\n' + RECEIPT_OK)
            env = dict(os.environ, WAITRUN_RUNNER=name)
            with self.assertRaises(subprocess.TimeoutExpired):
                subprocess.run(['zsh', str(SCRIPT), str(self.log), '1'], env=env,
                               capture_output=True, text=True, timeout=4)
        finally:
            os.killpg(alive.pid, signal.SIGKILL)  # the sh and its sleep (Loop 3546)
            alive.wait()

    def test_a_receipt_after_a_traceback_is_the_verdict(self):
        self.assertEqual(self.run_it(TRACEBACK + RECEIPT_OK)[0], 0)
        self.assertEqual(self.run_it(TRACEBACK + RECEIPT_BAD)[0], 1)

    def test_a_silent_log_with_no_runner_is_a_dead_runner(self):
        code, _, err = self.run_it('Staged: /x/map-7\n')
        self.assertEqual(code, 2)
        self.assertIn('holds no verdict', err)


class TheRunnerList(unittest.TestCase):
    def test_every_launcher_in_the_tree_is_on_the_default_runner_pattern(self):
        # Parked 1574: `WAITRUN_RUNNER`'s default is a list, and every new
        # launcher has missed it once (656, 1503, 1513, 1571) — the waiter
        # read a live capture as a dead runner and exited 2. A launcher is
        # a script that calls the launch line's `ron_wine` (or builds a
        # lane, `lane.sh`); this holds the list to the tree.
        line = next(l for l in SCRIPT.read_text().splitlines() if l.startswith('runner_pattern='))
        pattern = re.search(r"WAITRUN_RUNNER:-'([^']*)'", line).group(1)
        launchers = set()
        for path in list((ROOT / 'tools').rglob('*.sh')) + list((ROOT / 'tools').rglob('*.py')):
            if path.name.startswith('test_') or path.name == 'winelaunch.sh':
                continue
            if re.search(r'^[^#]*\bron_wine\b', path.read_text(), flags=re.M):
                launchers.add(path.name)
        launchers.add('lane.sh')
        unmatched = sorted(n for n in launchers if not re.search(pattern, 'tools/gamelog/' + n)
                           and not re.search(pattern, 'tools/explore/' + n)
                           and not re.search(pattern, 'tools/fuzz/' + n))
        self.assertEqual(unmatched, [], 'launchers the waiter would read as a dead runner')


if __name__ == '__main__':
    unittest.main()
