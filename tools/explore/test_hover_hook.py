"""The hover re-assert is opt-in (item 3494, parked 3463a).

`tools/trace/hover.h` holds the cfg key and the predicate; `hover_test.c`
builds them on the host and checks that the hook is off without
`hover=1` and on with it. The source checks pin the wiring the host build
cannot reach: the `describe_mouseover` site is skipped unless the key is
set, both where hooks are installed and where coverage stubs are excluded.
"""
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

TRACE = Path(__file__).resolve().parents[2] / 'tools' / 'trace'


class HoverHook(unittest.TestCase):
    @unittest.skipUnless(shutil.which('cc'), 'needs a host C compiler')
    def test_the_key_and_the_predicate(self):
        with tempfile.TemporaryDirectory() as d:
            exe = Path(d, 'hover_test')
            built = subprocess.run(['cc', '-Wall', '-Wextra', '-Werror', '-o', str(exe),
                                    str(TRACE / 'hover_test.c')], capture_output=True, text=True)
            self.assertEqual(built.returncode, 0, built.stderr)
            run = subprocess.run([str(exe)], capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stdout)
            self.assertEqual(run.stdout.strip(), 'ok')

    def test_the_site_is_gated_on_the_key(self):
        src = (TRACE / 'tracer.c').read_text()
        self.assertIn('{0x4b47e0, 9, K_HOVER, {0x56, 0x8b, 0xf1, 0x8d, 0x8e, 0x04, 0x01, 0x00, 0x00}}', src)
        self.assertIn('if (h->kind == K_HOVER && !g_hover_want) continue;', src)
        self.assertIn('(HOOKS[i].kind != K_HOVER || g_hover_want)', src)
        self.assertIn('static i32 g_hover_want,', src)  # zero: off by default
        self.assertIn('hover_cfg(p, &g_hover_want)', src)
        self.assertIn('g_hover_moved = 1;', (TRACE / 'click.h').read_text())


if __name__ == '__main__':
    unittest.main()
