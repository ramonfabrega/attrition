"""`tools/recomp/snapshot.py` reads a packet whose footer never came (Loop 2461).

2423's run854 packet, taken by `golden_capture.sh`'s `RON_STATE_FRAME` on
the click-free lane, ends where its header's ranges end: the stream's
close is lost at `!quit`. Every reader refused a packet whose ranges were
whole, and the lane skipped the check by hand. A footer-less packet whose
ranges sum to what the header promised loads, and says so; one whose
ranges do not is still refused.
"""
import importlib.util
import struct
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('snapshot', ROOT / 'tools/recomp/snapshot.py')
snapshot = importlib.util.module_from_spec(spec)
spec.loader.exec_module(snapshot)


def stream(ranges, total=None, footer=True):
    h = [0] * 32
    h[0], h[1], h[2], h[3], h[4], h[5], h[6] = snapshot.MAGIC, 1, 1, 4859, 4859, 7, 9
    h[15], h[16], h[17], h[18], h[19] = 0, len(ranges), sum(len(b) for _, b in ranges) if total is None else total, 0, 0
    out = struct.pack('<32I', *h)
    for base, data in ranges:
        out += struct.pack('<3I', 0, base, len(data)) + data
    if footer:
        out += struct.pack('<7I', snapshot.FOOTER, 1, 0, h[16], h[17], 3, 0)
    return out


class AFooterlessPacket(unittest.TestCase):
    def test_whole_ranges_load_without_their_footer(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / 'frame-snapshot.bin'
            p.write_bytes(stream([(0x400000, b'abcd' * 4), (0x500000, b'\x01\x00\x00\x00')], footer=False))
            s = snapshot.Snapshot(str(p))
            self.assertIsNone(s.footer)
            self.assertEqual(s.u32(0x500000), 1)
            self.assertEqual(s.read(0x400004, 4), b'abcd')
            p.write_bytes(stream([(0x400000, b'abcd' * 4)], footer=True))
            self.assertIsNotNone(snapshot.Snapshot(str(p)).footer)

    def test_short_ranges_are_still_refused(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / 'frame-snapshot.bin'
            p.write_bytes(stream([(0x400000, b'abcd' * 4)], total=32, footer=False))
            with self.assertRaises(ValueError):
                snapshot.Snapshot(str(p))


if __name__ == '__main__':
    unittest.main()
