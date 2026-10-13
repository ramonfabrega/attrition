"""`tools/recomp/flags.py`: `step4.py`'s flags as one pure function (parked
1869, the thirtieth pass): `--poke ADDR=HEXBYTES` writes bytes into the
packet before the entry, `--hook ADDR` prints the registers each time an
address is reached. Two scratch edits answered 1845 and 1738 wrote the hook
by hand (`s4h.py`); the flags graduate them.
"""
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/recomp'))
import flags  # noqa: E402


class Flags(unittest.TestCase):
    def test_the_old_flags_parse_as_before(self):
        f, rest = flags.parse(['exe', 'snap.bin', '--entry', '00591ef0', '--this', 'c061ec',
                               '--watch', '00c06188+4', '--find', '0x1234', '--args', '1,0x10'])
        self.assertEqual(rest, ['exe', 'snap.bin'])
        self.assertEqual(f.entry, 0x591ef0)
        self.assertEqual(f.this, 0xc061ec)
        self.assertEqual(f.watch, [(0xc06188, 4)])
        self.assertEqual(f.find, {0x1234})
        self.assertEqual(f.args, [1, 16])

    def test_a_poke_is_an_address_and_bytes(self):
        f, _ = flags.parse(['e', 's', '--poke', '00c06200=2c010000', '--poke', '0x10=ff'])
        self.assertEqual(f.poke, [(0xc06200, bytes.fromhex('2c010000')), (0x10, b'\xff')])

    def test_a_hook_is_an_address_with_a_cap(self):
        f, _ = flags.parse(['e', 's', '--hook', '005b4a10', '--hook', '5b4a20:3'])
        self.assertEqual(f.hook, [(0x5b4a10, 20), (0x5b4a20, 3)])

    def test_a_hook_line_names_the_registers(self):
        line = flags.hook_line(0x5b4a10, 'Unit::think_carry', 0x12, 7,
                               {'eax': 1, 'ecx': 0x10, 'edx': 2, 'ebx': 3, 'esi': 4, 'edi': 5, 'esp': 0x7d0ffff0},
                               [0x11, 0x22])
        self.assertIn('hook 005b4a10 Unit::think_carry+0x12 #7', line)
        self.assertIn('ecx=00000010', line)
        self.assertIn('[esp+4]=00000011', line)


if __name__ == '__main__':
    unittest.main()
