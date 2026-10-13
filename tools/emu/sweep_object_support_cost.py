#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_support_cost.py — `ObjectData::support_cost@006452f0` under unicorn.

    uv run tools/emu/sweep_object_support_cost.py <install>/riseofnations.exe

`__thiscall`, `ecx` = this; `*(int*)(type+0x270+i*4) << 8` through the type pointer at `this+0x18`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006452f0
OBJ, TYPE = 0x10000000, 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def rows():
    rng = random.Random(1723)
    out = []
    for i in (0, 1, 2, 5, 7, 15, 31):
        for v in (0, 1, -1, 3, 100, 0x7FFFFF, -0x800000, 0x7FFFFFFF, -0x80000000):
            out.append((i, v))
    out += [(rng.randint(0, 40), rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(150)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_map(TYPE, 0x2000)
    for i, v in rows():
        uc.mem_write(TYPE, bytes(0x2000))
        uc.mem_write(OBJ + 0x18, struct.pack("<I", TYPE))
        # decoys on both neighbours: the read is exactly slot `i`
        for j in range(0, 48):
            uc.mem_write(TYPE + 0x270 + j * 4, struct.pack("<i", 0x1234 + j))
        uc.mem_write(TYPE + 0x270 + i * 4, struct.pack("<i", v))
        r = signed(m.call(ENTRY, (i,), ecx=OBJ))
        print(f"{i} {v} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
