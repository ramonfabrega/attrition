#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_supply_deplete.py — `Supply::deplete@0073b5d0` under unicorn.

    uv run tools/emu/sweep_supply_deplete.py <install>/riseofnations.exe

`__thiscall`, `retl $4`; one stack argument, no write. Rows: `<arg> -> <bytes changed in 0x1000 of the object>`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0073b5d0
OBJ = 0x10000000
TYPE = 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)

def ints(seed=1739):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_map(TYPE, 0x2000)
    for a in ints()[:80]:
        buf = struct.pack("<i", a ^ 0x5A5A5A5A) * 0x400
        uc.mem_write(OBJ, buf)
        m.call(ENTRY, (a,), ecx=OBJ)
        after = bytes(uc.mem_read(OBJ, 0x1000))
        print(f"{a} -> {sum(1 for x, y in zip(buf, after) if x != y)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
