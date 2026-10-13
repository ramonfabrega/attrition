#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_groupmoveorder_is_fleeing.py — `GroupMoveOrder::is_fleeing@00485a40` under unicorn.

    uv run tools/emu/sweep_groupmoveorder_is_fleeing.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the order; the type record is the pointer at `this - 0x70`, its word at `+0x4` is an offset (0x40 here), and the flag byte sits at `this - 0x6c + offset`; returns the signed byte `& 1`. The rows run every value of the byte.
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0000485a40
OBJ = 0x10000000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1787):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for b in range(256):
        uc.mem_write(OBJ, bytes([0xA5]) * 0x1000)
        uc.mem_write(OBJ + 0x100 - 0x70, struct.pack("<I", OBJ + 0x800))
        uc.mem_write(OBJ + 0x804, struct.pack("<I", 0x40))
        uc.mem_write(OBJ + 0x100 - 0x6c + 0x40, bytes([b]))
        print(f"{b} -> {signed(m.call(ENTRY, (), ecx=OBJ + 0x100))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
