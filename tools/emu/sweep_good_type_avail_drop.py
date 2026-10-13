#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_good_type_avail_drop.py — `GoodTypeData::avail_drop@0066fd40` under unicorn.

    uv run tools/emu/sweep_good_type_avail_drop.py <install>/riseofnations.exe

`__thiscall`, `ecx` = this; the int `random_rare_drop` at `this+0x2f8`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0066fd40
OBJ, TYPE = 0x10000000, 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def rows():
    rng = random.Random(1723)
    out = [(v,) for v in INT_EDGES]
    out += [(rng.randint(-0x80000000, 0x7FFFFFFF),) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for row in rows():
        uc.mem_write(OBJ, bytes(0x1000))
        uc.mem_write(OBJ + 0x2f8, struct.pack("<i", row[0]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
