#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_get_type_count2.py — `get_type_count2@00669110` under unicorn.

    uv run tools/emu/sweep_get_type_count2.py <install>/riseofnations.exe

returns the byte at `0x00ed9936 + ecx` (`mov al, [ecx + 0xed9936]`; the rows seed 256 bytes and read the low byte).
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0000669110
OBJ = 0x10000000
TABLE = 0x00ed9936
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1782):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    for i in range(256):
        uc.mem_write(TABLE + i, bytes([(i * 7 + 3) & 0xFF]))
    for i in range(256):
        print(f"{i} -> {signed(m.call(ENTRY, (), ecx=i)) & 0xFF}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
