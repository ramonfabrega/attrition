#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_move_order_is_pathed.py — `MoveOrder::is_pathed@00488980` under unicorn.

    uv run tools/emu/sweep_move_order_is_pathed.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the order's embedded base; with `p = [ecx-0x54]`, `q = [p+4]`, returns `signed byte [q + ecx - 0x50] & 1`. The rows run every value of the byte.
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00488980
OBJ = 0x10000000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1766):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for b in range(256):
        uc.mem_write(OBJ, bytes([0xA5]) * 0x1000)
        uc.mem_write(OBJ + 0x100 - 0x54, struct.pack("<I", OBJ + 0x800))
        uc.mem_write(OBJ + 0x804, struct.pack("<I", 0x40))
        uc.mem_write(OBJ + 0x100 - 0x50 + 0x40, bytes([b]))
        print(f"{b} -> {signed(m.call(ENTRY, (), ecx=OBJ + 0x100))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
