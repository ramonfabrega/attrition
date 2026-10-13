#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_good_data_get_gpiece.py — `GoodData::get_gpiece@0066e5b0` under unicorn (item 1824).

    uv run tools/emu/sweep_good_data_get_gpiece.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the good, `+0x18` its type record, whose dword at `+4` plus the `first_good_piece` (`+0xd58`) of the structure the global cell `0xc06214` points to: a wrapping 32-bit sum.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0066e5b0
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000


def rows():
    rng = random.Random(1824)
    out = []
    edges = [0, 1, -1, 7, 0x7FFFFFFF, -0x80000000, 100000, -100000]
    for k in edges:
        for b in edges:
            out.append((k, b))
    for _ in range(200):
        out.append((rng.getrandbits(32) - 0x80000000, rng.getrandbits(32) - 0x80000000))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x1000)
    for n, row in enumerate(rows()):
        k, b = row
        TYPE, GP = OBJ + 0x1000, OBJ + 0x4000
        uc.mem_write(OBJ + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(TYPE + 4, struct.pack("<i", k))
        uc.mem_write(0x00C06214, struct.pack("<I", GP))
        uc.mem_write(GP + 0xD58, struct.pack("<i", b))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{k} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
