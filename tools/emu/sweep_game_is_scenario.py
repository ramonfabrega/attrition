#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_is_scenario.py — `Game::is_scenario@0043ec70` under unicorn (item 1814).

    uv run tools/emu/sweep_game_is_scenario.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the game; two flag bytes at +0x821 and +0x822 — true when byte 0 has 0x10 or byte 1 has 0x02, over every byte against the other's clear and set cases.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0043ec70
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
    rows = [(a, b) for a in range(256) for b in (0, 1, 2, 3, 0xFD, 0xFF)]
    rows += [(a, b) for b in range(256) for a in (0, 1, 0x10, 0xEF, 0xFF)]
    for a, b in rows:
        uc.mem_write(OBJ + 0x821, bytes([a, b]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{a} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
