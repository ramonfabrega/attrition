#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_is_infinite.py — `Game::is_infinite@00594640` under unicorn (item 1814).

    uv run tools/emu/sweep_game_is_infinite.py <install>/riseofnations.exe

`__cdecl`-shaped (reads the `GameAccess::game` cell at 0xc061ec); true when the byte at game+0x2d is 8 — every byte value, against `b == 8`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00594640
OBJ, ARGS = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def words(seed=1814, n=150):
    rng = random.Random(seed)
    return list(EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(n)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARGS, 0x1000)
    for b in range(256):
        uc.mem_write(0xC061EC, struct.pack("<I", OBJ))
        uc.mem_write(OBJ + 0x2D, bytes([b]))
        r = signed(m.call(ENTRY, ()))
        print(f"{b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
