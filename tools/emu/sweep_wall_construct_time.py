#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_construct_time.py — `WallData::construct_time@00642d90` under unicorn.

    uv run tools/emu/sweep_wall_construct_time.py <install>/riseofnations.exe

`__thiscall(int)`, `ecx` = the wall; returns `constr_time` (ulong at +0x50), the argument ignored.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00642d90
UNIT, TYPE = 0x10000000, 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1701)
    out = [(v & 0xFFFFFFFF,) for v in INT_EDGES]
    out += [(rng.getrandbits(32),) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(UNIT + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(UNIT + 0x50, struct.pack("<I", row[0]))
        r = m.call(ENTRY, (0,), ecx=UNIT)
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
