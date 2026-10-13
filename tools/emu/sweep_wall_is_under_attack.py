#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_is_under_attack.py — `WallData::is_under_attack@00472420` under unicorn.

    uv run tools/emu/sweep_wall_is_under_attack.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the wall; `build_masks` (short at +0x60) `& 0x20`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00472420
UNIT, TYPE = 0x10000000, 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1701)
    edges = [0, -1, -0x8000, 0x7FFF] + [1 << b for b in range(15)] + [-0x8000 | (1 << b) for b in range(15)]
    edges += [0x7FFF ^ (1 << b) for b in range(15)]
    out = [(v,) for v in edges] + [(rng.randint(-0x8000, 0x7FFF),) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(UNIT + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(UNIT + 0x60, struct.pack("<h", row[0]))
        r = signed(m.call(ENTRY, (), ecx=UNIT))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
