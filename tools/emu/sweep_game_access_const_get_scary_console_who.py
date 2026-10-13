#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_access_const_get_scary_console_who.py — `GameAccessConst::get_scary_console_who@005834a0` under unicorn.

    uv run tools/emu/sweep_game_access_const_get_scary_console_who.py <install>/riseofnations.exe

reads the pointer in the global cell at 0x00c06210 and returns the word at +0x298 of what it points at; the rows seed the record.
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x005834a0
CELL = 0x00c06210
REC = 0x10000000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1766):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(REC, 0x2000)
    uc.mem_write(CELL, struct.pack("<I", REC))
    for w in ints()[:80]:
        # distinct words all over the record, so an offset that is not 0x298 reads a different row
        uc.mem_write(REC, b"".join(struct.pack("<I", (w + 0x01010101 * i) & 0xFFFFFFFF) for i in range(0x800)))
        print(f"{w} -> {signed(m.call(ENTRY, ()))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
