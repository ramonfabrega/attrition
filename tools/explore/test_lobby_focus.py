#!/usr/bin/env python3
"""No desktop access: mocked commands verify focus failures cannot click."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1] / 'gamelog'


class FocusTest(unittest.TestCase):
    def run_mock(self, answer='', status=0, title_only=False):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            scripts = {'osascript': '#!/bin/sh\nprintf "%s" "$MOCK_TITLE"\nexit "$MOCK_STATUS"\n',
                       'cliclick': '#!/bin/sh\necho clicked >> "$MOCK_CLICKS"\n',
                       'sleep': '#!/bin/sh\nexit 0\n'}
            for name, content in scripts.items():
                p = root/name; p.write_text(content); p.chmod(0o755)
            env = dict(os.environ, PATH=str(root)+':'+os.environ['PATH'], MOCK_TITLE=answer,
                       MOCK_STATUS=str(status), MOCK_CLICKS=str(root/'clicks'), RON_WINDOW='Rise of Nations')
            if title_only:
                command = ['zsh', str(TOOLS/'focus.sh'), '--title']
            else:
                command = ['zsh', '-c', 'source "$1/lobby.sh"; RON_TOOLS=$1; LOBBY[start]="1 2"; LOBBY_STARTS=2; lobby_start 0', 'test', str(TOOLS)]
            result = subprocess.run(command, env=env, text=True, capture_output=True)
            clicks = (root/'clicks').read_text().splitlines() if (root/'clicks').exists() else []
            return result, clicks

    def test_missing_window_stops_before_first_click(self):
        result, clicks = self.run_mock()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('no matching game window', result.stderr)
        self.assertEqual(clicks, [])

    def test_automation_error_stops_before_first_click(self):
        result, clicks = self.run_mock(status=23)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(clicks, [])

    def test_found_window_allows_both_start_presses(self):
        result, clicks = self.run_mock(answer='Rise of Nations')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(clicks), 2)

    def test_title_query_preserves_empty_success(self):
        result, clicks = self.run_mock(title_only=True)
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout.strip(), '')
        self.assertEqual(clicks, [])


if __name__ == '__main__':
    unittest.main()
