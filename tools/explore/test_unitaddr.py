"""`tools/recomp/unitaddr.py`: a unit's address in a packet from its position
(parked 1953, the thirty-first pass: three lanes scanned a packet for the
masked pair by hand)."""
import struct
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/recomp'))
import unitaddr  # noqa: E402


class Scan(unittest.TestCase):
    def test_a_unit_is_found_by_its_masked_position(self):
        x, y = 37719, 44523
        blob = bytes(0x40) + struct.pack('<II', x ^ unitaddr.MASK, y ^ unitaddr.MASK) + bytes(0x20)
        self.assertEqual(unitaddr.find([(0x1000, blob)], x, y), [0x1000 + 0x40 - unitaddr.AT])
        self.assertEqual(unitaddr.find([(0x1000, blob)], x, y + 1), [])

    def test_every_hit_is_listed_across_ranges(self):
        x, y = 5, 6
        pair = struct.pack('<II', x ^ unitaddr.MASK, y ^ unitaddr.MASK)
        self.assertEqual(unitaddr.find([(0x1000, bytes(0x10) + pair), (0x2000, bytes(0x30) + pair)], x, y),
                         [0x1000, 0x2020])

    def test_the_command_line_is_a_packet_and_a_position(self):
        self.assertEqual(unitaddr.parse(['p.bin', '37719', '44523']), ('p.bin', 37719, 44523))
        with self.assertRaises(SystemExit):
            unitaddr.parse(['p.bin', '37719'])


if __name__ == '__main__':
    unittest.main()
