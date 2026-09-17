#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Authored framing fixtures for the bounded path-deletion witness."""
import struct
import unittest
from replay_path_capsule import PathCapsule, ENTRY


def fixture():
    raw = bytearray(2296)
    self_, path, esp, stop = 0x100000, 0x200000, 0x300000, 0x400000
    regs = [1, 2, 3, esp, 4, 5, self_, 6, 0x202]
    after = list(regs); after[3] += 4
    struct.pack_into('<27I', raw, 0, 0x31504350, 1, ENTRY, stop, 20, self_, path, 3, 0x500000, *regs, *after)
    struct.pack_into('<3I', raw, 212, path, 4, 3)
    struct.pack_into('<3I', raw, 224, path, 4, 1)
    for i in range(3):
        struct.pack_into('<4I', raw, 244+i*16, i, i+1, i+2, i & 1)
    raw[1268:1316] = raw[244:292]
    struct.pack_into('<I', raw, 2292, stop)
    return raw


class PathCapsuleTest(unittest.TestCase):
    def test_exact_permissions_and_complete_rows(self):
        c = PathCapsule(fixture())
        self.assertEqual(c.remaining, 1)
        self.assertEqual(len(c.row().split()), 14)
        self.assertEqual([(r.name, len(r.data)) for r in c.regions if r.writable], [('length', 4), ('scratch', 16)])
        self.assertEqual(len(c.expected['path']), 48)

    def test_truncated_and_oversized(self):
        for raw in (fixture()[:-1], fixture()+b'\0'):
            with self.assertRaisesRegex(ValueError, 'truncated or oversized'):
                PathCapsule(raw)

    def test_invalid_count(self):
        for count in (0, 65):
            raw=fixture(); struct.pack_into('<I', raw, 28, count)
            with self.assertRaisesRegex(ValueError, 'invalid capture domain'):
                PathCapsule(raw)

    def test_suspended_search_rejected(self):
        raw=fixture(); raw[236]=1
        with self.assertRaisesRegex(ValueError, 'suspended search'):
            PathCapsule(raw)

    def test_equal_length_is_not_a_mutation(self):
        raw=fixture(); struct.pack_into('<I', raw, 232, 3)
        with self.assertRaisesRegex(ValueError, 'no observed path deletion'):
            PathCapsule(raw)

    def test_invalid_pointer(self):
        raw=fixture(); raw[212]^=1
        with self.assertRaisesRegex(ValueError, 'broken path pointer'):
            PathCapsule(raw)

    def test_inconsistent_count(self):
        raw=fixture(); raw[220]^=1
        with self.assertRaisesRegex(ValueError, 'inconsistent input length'):
            PathCapsule(raw)

    def test_exit_stack(self):
        raw=fixture(); raw[84]^=1
        with self.assertRaisesRegex(ValueError, 'stack effect'):
            PathCapsule(raw)


if __name__ == '__main__':
    unittest.main()
