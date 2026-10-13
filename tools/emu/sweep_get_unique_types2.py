#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_get_unique_types2.py — `get_unique_types2@00669100` under unicorn.

    uv run tools/emu/sweep_get_unique_types2.py <install>/riseofnations.exe

returns the byte held in the global cell at 0x00ed88cc (`mov al, [cell]`; only `al` is the answer, so the rows read the low byte).
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0000669100
OBJ = 0x10000000
CELL = 0x00ed88cc
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1782):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    for b in range(256):
        uc.mem_write(CELL, bytes([b]))
        print(f"{b} -> {signed(m.call(ENTRY, ())) & 0xFF}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
