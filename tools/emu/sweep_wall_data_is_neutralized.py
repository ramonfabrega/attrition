#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_data_is_neutralized.py — `WallData::is_neutralized@0063ed30` under unicorn.

    uv run tools/emu/sweep_wall_data_is_neutralized.py <install>/riseofnations.exe

`__thiscall`, `ecx` = this; the constant 0.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0063ed30
OBJ = 0x10000000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints():
    rng = random.Random(1724)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for v in range(5):
        uc.mem_write(OBJ, bytes([v * 37 & 255]) * 0x1000)
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{v} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
