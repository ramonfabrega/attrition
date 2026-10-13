#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_access_const_get_scary_scenario_editor.py — `GameAccessConst::get_scary_scenario_editor@00583380` under unicorn.

    uv run tools/emu/sweep_game_access_const_get_scary_scenario_editor.py <install>/riseofnations.exe

returns the word held in the global cell at 0x00e7f8dc; the rows seed the cell and ask.
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00583380
CELL = 0x00e7f8dc
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1766):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    for w in ints()[:80]:
        uc.mem_write(CELL, struct.pack("<I", w & 0xFFFFFFFF))
        print(f"{w} -> {signed(m.call(ENTRY, ()))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
