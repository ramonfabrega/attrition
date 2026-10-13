#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_get_type_count.py — `get_type_count@00669130` under unicorn.

    uv run tools/emu/sweep_get_type_count.py <install>/riseofnations.exe

`ecx` = column, `edx` = row; returns the byte at `0x00ed89ce + edx * 0x1ed + ecx` (low byte). The rows seed the table and read the low byte.
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00669130
OBJ = 0x10000000
TABLE = 0x00ed89ce
STRIDE = 0x1ed
ROWS = 6


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_write(TABLE, bytes(((k * 131 + (k >> 8) * 17 + 5) & 0xFF) for k in range(ROWS * STRIDE)))
    for row in range(ROWS):
        for col in list(range(0, STRIDE, 3)) + [STRIDE - 1]:
            print(f"{row} {col} -> {signed(m.call(ENTRY, (), ecx=col, edx=row)) & 0xFF}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
