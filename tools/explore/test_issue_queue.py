"""The `@` lines' `queued` byte (items 3448, 3504).

`tools/trace/queue.h` holds the choice; `queue_test.c` builds it on the
host and checks QUEUE_NEW for a plain verb, QUEUE_LAST for the shift-click
verbs and QUEUE_FIRST for `@repairins`. The source checks pin the wiring
the host build cannot reach: the verb parses to 16, the swarm issuer is
called with the chosen byte, and `@repair` itself still passes QUEUE_NEW.
"""
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

TRACE = Path(__file__).resolve().parents[2] / 'tools' / 'trace'


class IssueQueue(unittest.TestCase):
    @unittest.skipUnless(shutil.which('cc'), 'needs a host C compiler')
    def test_the_queue_byte(self):
        with tempfile.TemporaryDirectory() as d:
            exe = Path(d, 'queue_test')
            built = subprocess.run(['cc', '-Wall', '-Wextra', '-Werror', '-o', str(exe),
                                    str(TRACE / 'queue_test.c')], capture_output=True, text=True)
            self.assertEqual(built.returncode, 0, built.stderr)
            run = subprocess.run([str(exe)], capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stdout)
            self.assertEqual(run.stdout.strip(), 'ok')

    def test_the_verb_is_wired(self):
        src = (TRACE / 'tracer.c').read_text()
        self.assertIn('#include "queue.h"', src)
        self.assertIn('issue_verb(&t, "repairins ") ? (queue_first = 1, 16)', src)
        self.assertIn('issue_verb(&t, "repair ")   ? 16', src)
        self.assertIn('g_groupout, x, y, issue_queue(0, queue_first), 13);', src)
        self.assertIn('queue_add ? 1 : 2', src)  # the shift-click verbs, unchanged
        self.assertNotIn('g_groupout, x, y, 2, 13);', src)


if __name__ == '__main__':
    unittest.main()
