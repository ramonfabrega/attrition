#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_valid.py — `WorldData::valid@00461420` under unicorn (item 1814).

    uv run tools/emu/sweep_world_data_valid.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world data (`xs` +0, `ys` +4); two `WCoord` pointers on the stack — in range when both are non-negative and under their extent.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00461420
OBJ, ARGS = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def words(seed=1814, n=150):
    rng = random.Random(seed)
    return list(EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(n)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARGS, 0x1000)
    rng = random.Random(1814)
    vals = (-1, 0, 1, 5, 99, 100, 101, 0x7FFFFFFF, -0x80000000)
    rows = [(xs, ys, x, y) for xs in (0, 1, 100) for ys in (0, 1, 50) for x in vals for y in vals]
    rows += [(rng.randint(-5, 300), rng.randint(-5, 300), rng.randint(-5, 300), rng.randint(-5, 300)) for _ in range(600)]
    for xs, ys, x, y in rows:
        uc.mem_write(OBJ, struct.pack("<ii", xs, ys))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=OBJ))
        print(f"{xs} {ys} {x} {y} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
