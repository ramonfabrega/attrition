"""`startcapture.sh` is held to what `longtrace.sh` is (parked 1080, the
nineteenth pass).

The third map's start capture (item 1066, run381) was the first taken
through `viadriver.sh` by a worker, and the script had been left behind by
three passes of rules written for its sibling: it printed no banner, so
`waitrun.sh` called its success a dead runner and exited 2; it restored the
INIs and never the lobby, so the map it wrote into the profile outlived it
(parked 987 was `longtrace.sh`'s half); and a take with no window left the
INIs staged (parked 937, the same).

The waiter is run here on authored logs with no runner alive, so it answers
at once and sleeps for nothing.
"""
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'tools/gamelog/startcapture.sh'
WAITER = ROOT / 'tools/gamelog/waitrun.sh'


def code():
    text = SCRIPT.read_text()
    return [(i + 1, l) for i, l in enumerate(text.split('\n'))
            if not l.lstrip().startswith('#')]


class TheLobbyIsPutBack(unittest.TestCase):
    def test_the_lobby_is_saved_before_the_first_write_to_it(self):
        lines = code()
        save = next((i for i, l in lines if re.match(r'\s*save_lobby\b', l)), None)
        write = next(i for i, l in lines if 'mapstyle.py' in l)
        self.assertIsNotNone(save, 'startcapture.sh never saves the lobby')
        self.assertLess(save, write)

    def test_every_restore_of_the_inis_restores_the_lobby(self):
        lines = code()
        restores = [n for n, (i, l) in enumerate(lines) if 'window.py" restore' in l]
        self.assertGreaterEqual(len(restores), 2,
                                'a take with no window restores nothing')
        for n in restores:
            self.assertRegex(lines[n + 1][1], r'^\s*restore_lobby\b',
                             f'line {lines[n][0]} restores the INIs and not the lobby')

    def test_a_take_with_no_window_is_not_left_to_set_e(self):
        body = SCRIPT.read_text()
        self.assertRegex(body, r'if ! zsh "\$W/tools/gamelog/waitwin\.sh"')


class TheWaiterReadsItsBanner(unittest.TestCase):
    def wait(self, text):
        with tempfile.TemporaryDirectory() as d:
            log = Path(d) / 'viadriver.log'
            log.write_text(text)
            # A runner pattern nothing matches: the verdict is the log's.
            return subprocess.run(
                ['zsh', str(WAITER), str(log), '1'],
                env={'PATH': '/usr/bin:/bin', 'WAITRUN_RUNNER': 'no-such-runner-1080'},
                capture_output=True, text=True, timeout=30)

    def banners(self):
        body = SCRIPT.read_text()
        return re.findall(r'echo "(=== capture[^"]*)"', body)

    def test_the_script_prints_both_banners(self):
        said = self.banners()
        self.assertTrue(any(b.startswith('=== captured: ') for b in said), said)
        self.assertTrue(any(b.startswith('=== capture failed: ') for b in said), said)

    def test_a_finished_start_capture_is_a_success(self):
        done = self.wait('launched pid 4\nsettled\n=== captured: run381 (greatsahara-start) ===\n')
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn('run381', done.stdout)

    def test_a_take_with_no_window_is_a_failure(self):
        done = self.wait('launched pid 4\n=== capture failed: run381, no window ===\n')
        self.assertEqual(done.returncode, 1, done.stderr)

    def test_a_log_with_no_banner_is_still_a_dead_runner(self):
        done = self.wait('launched pid 4\n')
        self.assertEqual(done.returncode, 2, done.stderr)

    def test_the_waiter_knows_the_script_as_a_runner(self):
        body = WAITER.read_text()
        default = re.search(r"runner_pattern=\$\{WAITRUN_RUNNER:-'([^']*)'\}", body)
        self.assertIsNotNone(default)
        self.assertIn('gamelog/startcapture.sh', default.group(1))


if __name__ == '__main__':
    unittest.main()
