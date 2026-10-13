#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_is_turbo_resources.py — `Game::is_turbo_resources@0043ecb0` under unicorn (item 1819).

    uv run tools/emu/sweep_game_is_turbo_resources.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the game; true when the flag byte at +0x20 has 0x2 or the rules byte at +0x2a is 9 — every byte against the other's clear and set cases.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0043ecb0
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    rows = [(f, g) for f in range(256) for g in (0, 8, 9, 10, 0xFF)]
    rows += [(f, g) for g in range(256) for f in (0, 2, 0xFD, 0xFF)]
    for f, g in rows:
        uc.mem_write(OBJ + 0x20, bytes([f]))
        uc.mem_write(OBJ + 0x2A, bytes([g]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{f} {g} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
