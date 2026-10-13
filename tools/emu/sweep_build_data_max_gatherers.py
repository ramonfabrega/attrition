#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_data_max_gatherers.py — `BuildData::max_gatherers@00630590` under unicorn (item 1793, tools/sweep_gen.py).

    uv run tools/emu/sweep_build_data_max_gatherers.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; reads the int at this+0x80.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00630590
OBJ, TYPE = 0x10000000, 0x10100000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1793)
    out = [(v,) for v in INT_EDGES]
    out += [(rng.randint(-0x80000000, 0x7FFFFFFF),) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(OBJ + 0x80, struct.pack("<i", row[0]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
