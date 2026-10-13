#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_data_hits.py — `WallData::hits@00642bb0` under unicorn.

    uv run tools/emu/sweep_wall_data_hits.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the wall data, one stack argument `a`; the int at `+0x20` for a non-zero argument, the one at `+0x54` for zero. Rows: `a v20 v54 -> result`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00642bb0
OBJ = 0x10000000
TYPE = 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)

def ints(seed=1738):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_map(TYPE, 0x2000)
    xs = ints()
    for k, a in enumerate((0, 1, -1, 2, 0x7FFFFFFF, -0x80000000) * 20):
        v20, v54 = xs[k], xs[(k * 7 + 3) % len(xs)]
        uc.mem_write(OBJ, bytes(0x1000))
        uc.mem_write(OBJ + 0x20, struct.pack("<i", v20))
        uc.mem_write(OBJ + 0x54, struct.pack("<i", v54))
        print(f"{a} {v20} {v54} -> {signed(m.call(ENTRY, (a,), ecx=OBJ))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
