#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Order-capsule framing guards using authored bytes, without a game install."""
import struct
import unittest
from replay_order_capsule import OrderCapsule, ENTRY


def fixture():
    raw = bytearray(240)
    self_, head, node, stack, stop = 0x100000, 0x200000, 0x200100, 0x300000, 0x400000
    regs = [1, 2, 3, stack, 4, 5, self_, 6, 0x202]
    after_regs = list(regs); after_regs[3] += 4
    struct.pack_into('<27I', raw, 0, 0x314f4352, 1, ENTRY, stop, 0, self_, head, node, 0x500000, *regs, *after_regs)
    struct.pack_into('<I', raw, 172+16, head)
    raw[192:212] = raw[172:192]
    struct.pack_into('<2I', raw, 212, node, node)
    struct.pack_into('<I', raw, 236, stop)
    return raw


class OrderCapsuleTest(unittest.TestCase):
    def test_exact_write_permissions_and_equal_value_refresh(self):
        c = OrderCapsule(fixture())
        self.assertEqual(c.changed_bytes, 0)
        self.assertEqual([(r.address, len(r.data)) for r in c.regions if r.writable],
                         [(c.self_+0xcc, 4), (c.self_+0xd0, 1), (c.self_+0xd4, 4)])
        self.assertEqual(sum(len(r.data) for r in c.regions), 98)

    def test_truncated_and_oversized(self):
        for raw in (fixture()[:-1], fixture()+b'\0'):
            with self.assertRaisesRegex(ValueError, 'truncated or oversized'):
                OrderCapsule(raw)

    def test_identity(self):
        raw = fixture(); raw[0] ^= 1
        with self.assertRaisesRegex(ValueError, 'unsupported'):
            OrderCapsule(raw)

    def test_ecx(self):
        raw = fixture(); raw[60] ^= 1
        with self.assertRaisesRegex(ValueError, 'self differs'):
            OrderCapsule(raw)

    def test_pointer_graph(self):
        raw = fixture(); raw[188] ^= 1
        with self.assertRaisesRegex(ValueError, 'broken pointer graph'):
            OrderCapsule(raw)

    def test_return_slot(self):
        raw = fixture(); raw[236] ^= 1
        with self.assertRaisesRegex(ValueError, 'stop differs'):
            OrderCapsule(raw)

    def test_exit_stack(self):
        raw = fixture(); raw[84] ^= 1
        with self.assertRaisesRegex(ValueError, 'stack effect'):
            OrderCapsule(raw)

    def test_frame(self):
        raw = fixture(); struct.pack_into('<I', raw, 16, 36)
        with self.assertRaisesRegex(ValueError, 'outside capture window'):
            OrderCapsule(raw)


if __name__ == '__main__':
    unittest.main()
