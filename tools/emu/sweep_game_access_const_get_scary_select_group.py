#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_access_const_get_scary_select_group.py — `GameAccessConst::get_scary_select_group@00583450` under unicorn.

    uv run tools/emu/sweep_game_access_const_get_scary_select_group.py <install>/riseofnations.exe

reads the pointer in the global cell at 0x00c06210 and returns the word at 0x00e8d444 plus `0x9f0 *` its word at +0x2a0 (a 32-bit multiply); the rows seed the record and the base word.
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00583450
OBJ = 0x10000000
CELL = 0x00c06210
BASE = 0x00e8d444
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1787):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_write(CELL, struct.pack("<I", OBJ))
    for w in ints()[:80]:
        uc.mem_write(OBJ, b"".join(struct.pack("<I", (w + 0x01010101 * i) & 0xFFFFFFFF) for i in range(0x800)))
        uc.mem_write(BASE, struct.pack("<I", ~w & 0xFFFFFFFF))
        print(f"{w} -> {signed(m.call(ENTRY, ()))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
