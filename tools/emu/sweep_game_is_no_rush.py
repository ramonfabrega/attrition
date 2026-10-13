#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_is_no_rush.py — `Game::is_no_rush@005946c0` under unicorn.

    uv run tools/emu/sweep_game_is_no_rush.py <install>/riseofnations.exe

reads the game pointer in the global cell at 0x00c061ec; returns whether the byte at +0x32 is non-zero. The rows run every value of the byte.
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x005946c0
OBJ = 0x10000000


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_write(0x00c061ec, struct.pack("<I", OBJ))
    for b in range(256):
        uc.mem_write(OBJ, bytes([0xA5]) * 0x1000)
        uc.mem_write(OBJ + 0x32, bytes([b]))
        print(f"{b} -> {signed(m.call(ENTRY, ()))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
