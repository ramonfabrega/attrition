#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_is_tournament.py — `Game::is_tournament@0043ec90` under unicorn (item 1819).

    uv run tools/emu/sweep_game_is_tournament.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the game; true when the byte at +0x820 has 0x4 and the byte at +0x20 has 0x80 — every byte against the other's clear and set cases.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0043ec90
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    rows = [(a, b) for a in range(256) for b in (0, 1, 0x7F, 0x80, 0xFF)]
    rows += [(a, b) for b in range(256) for a in (0, 4, 0xFB, 0xFF)]
    for a, b in rows:
        uc.mem_write(OBJ + 0x820, bytes([a]))
        uc.mem_write(OBJ + 0x20, bytes([b]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{a} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
