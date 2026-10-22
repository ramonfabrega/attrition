"""The held key's `@hold <vk> <until-frame>` line (item 3502, parked 3442a, 3495a).

`tools/trace/hold.h` holds the import walk that finds the executable's own
`GetKeyState` slot by name, the span check and the answer; `hold_test.c`
builds them on the host. The source checks pin the wiring the host build
cannot reach: the verb is dispatched, the slot is the one 3495a read off the
shipped image, and the hook answers the high byte only inside a span.
"""
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

TRACE = Path(__file__).resolve().parents[2] / 'tools' / 'trace'


class HoldHook(unittest.TestCase):
    @unittest.skipUnless(shutil.which('cc'), 'needs a host C compiler')
    def test_the_import_the_span_and_the_answer(self):
        with tempfile.TemporaryDirectory() as d:
            exe = Path(d, 'hold_test')
            built = subprocess.run(['cc', '-Wall', '-Wextra', '-Werror', '-o', str(exe),
                                    str(TRACE / 'hold_test.c')], capture_output=True, text=True)
            self.assertEqual(built.returncode, 0, built.stderr)
            run = subprocess.run([str(exe)], capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stdout)
            self.assertEqual(run.stdout.strip(), 'ok')

    def test_the_wiring(self):
        src = (TRACE / 'tracer.c').read_text()
        self.assertIn('if (issue_verb(&t, "hold ")) { hold_line(frame, idx, t); return; }', src)
        self.assertIn('#define RVA_GETKEYSTATE_IAT 0x6c53e8u', src)
        self.assertIn('hold_import_slot((const u8 *)g_base, "USER32.dll", "GetKeyState")', src)
        self.assertIn('if (hold_held(g_holds, g_nholds, vk, g_frame)) {', src)
        self.assertIn('return (short)0x8000;', src)
        self.assertIn('return g_hold_real(vk);', src)
        # the slot is touched only by a line: no install at attach or at the first frame
        self.assertEqual(src.count('hold_install()'), 1)


if __name__ == '__main__':
    unittest.main()
