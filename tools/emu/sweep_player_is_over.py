#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_player_is_over.py — `Player::is_over@0043e220` under unicorn.

    uv run tools/emu/sweep_player_is_over.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the player; the 16-bit word at `+0x30` has bit 0x100 or any of 0xd0 set -> 1. Rows: the dword at `+0x30` (the high half a decoy) -> result.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0043e220
OBJ = 0x10000000
TYPE = 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)

def ints(seed=1735):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_map(TYPE, 0x2000)
    for w in ints():
        uc.mem_write(OBJ, struct.pack("<i", 0x5A5A5A5A) * 0x400)
        uc.mem_write(OBJ + 0x30, struct.pack("<i", w))
        print(f"{w} -> {signed(m.call(ENTRY, (), ecx=OBJ))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
