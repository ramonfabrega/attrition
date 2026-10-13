#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_area.py — `BuildTypeData::area@006389b0` under unicorn (item 1793, tools/sweep_gen.py).

    uv run tools/emu/sweep_build_type_data_area.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; reads ints at this+0x234 and this+0x238; returns their product (32-bit wrap).
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006389b0
OBJ, TYPE = 0x10000000, 0x10100000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1793)
    out = [(a, b) for a in INT_EDGES[:8] for b in INT_EDGES[:8]]
    out += [(rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(OBJ + 0x234, struct.pack("<ii", row[0], row[1]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
