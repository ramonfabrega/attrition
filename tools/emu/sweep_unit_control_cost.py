#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_control_cost.py — `UnitData::control_cost@00609d40` under unicorn.

    uv run tools/emu/sweep_unit_control_cost.py <install>/riseofnations.exe

`__thiscall(int)`, `ecx` = the unit; 0 when the argument is 0 and `unit_masks` (+0x68) `& 1`, else the int at type+0x2f0. Row: `<masks> <arg> <pop>`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00609d40
UNIT, TYPE = 0x10000000, 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1701)
    out = [(a, b, c) for a in (0, 1, 2, 3, 0xFFFFFFFF, 0xFFFFFFFE) for b in (0, 1, -1, 7) for c in (0, 1, 2, -1, 0x7FFFFFFF)]
    out += [(rng.getrandbits(32), rng.choice((0, 0, 1, rng.randint(-5, 5))), rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(UNIT + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(UNIT + 0x68, struct.pack("<I", row[0]))
        uc.mem_write(TYPE + 0x2f0, struct.pack("<i", row[2]))
        r = signed(m.call(ENTRY, (row[1],), ecx=UNIT))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
