#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_game_access_const_get_scary_cliffs.py — `GameAccessConst::get_scary_cliffs@00583370` under unicorn.

    uv run tools/emu/sweep_game_access_const_get_scary_cliffs.py <install>/riseofnations.exe

`__cdecl`, no arguments; returns the address of a global (0x00e88eb0) whatever `ecx` holds. One row per input, `<ecx> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00583370
CELL = 0x00e88eb0
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1760):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    for w in ints()[:20]:
        # the result must not depend on the cell contents: seed `ecx` and the stack with noise
        print(f"{w} -> {m.call(ENTRY, (), ecx=w & 0xFFFFFFFF) & 0xFFFFFFFF}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
