"""`tools/emu/hooks.py`'s stub region grows (Loop 2043, parked 2022).

One page held 256 stubs and never reused one, so a script that built a
vtable per row faulted at the 257th row with an unmapped fetch. The region
now maps a page as it is reached, up to `STUB_PAGES_MAX`. The harness maps
the executable, so this needs the install (`RON_INSTALL`) and skips without
it — saying so, never passing quietly.
"""
import os
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools' / 'emu'))

INSTALL = os.environ.get('RON_INSTALL')
EXE = Path(INSTALL, 'riseofnations.exe') if INSTALL else None
try:
    import unicorn  # noqa: F401  (the scripts declare it for `uv run`)
    UNICORN = True
except ImportError:
    UNICORN = False


@unittest.skipUnless(EXE and EXE.is_file() and UNICORN,
                     'needs RON_INSTALL and unicorn (`uv run --with unicorn`): the harness maps the executable')
class StubRegionGrows(unittest.TestCase):
    def test_the_three_hundredth_stub_is_mapped(self):
        import hooks
        h = hooks.Harness(str(EXE), {})
        addrs = [h.stub(f's{i}', 0, 0) for i in range(300)]
        mapped = list(h.m.uc.mem_regions())
        for a in (addrs[0], addrs[256], addrs[299]):
            self.assertTrue(any(lo <= a <= hi for lo, hi, _ in mapped), f'stub {a:#x} is not mapped')
        self.assertEqual(h.stub_pages, 2)

    def test_the_region_refuses_by_name_past_its_ceiling(self):
        import hooks
        h = hooks.Harness(str(EXE), {})
        h.stub_pages = hooks.STUB_PAGES_MAX
        h.next_stub = hooks.STUBS + hooks.STUB_PAGES_MAX * 0x1000
        with self.assertRaises(MemoryError) as cm:
            h.stub('over', 0, 0)
        self.assertIn('2043', str(cm.exception))


if __name__ == '__main__':
    unittest.main()
