#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_is_plane.py — `UnitData::is_plane@0046ce40` under unicorn.

    uv run tools/emu/sweep_unit_is_plane.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the unit (type pointer at +0x18); answers `type+0x218 == 2 && !(type+0x2b4 & 0x20)`.
One row per input, `<domain> <flags> -> <result>`: domains -1..4 against flag words with and without 0x20.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046ce40
UNIT, TYPE = 0x10000000, 0x10010000


def rows():
    rng = random.Random(1691)
    out = [(d, f) for d in range(-1, 5) for f in (0, 0x20, 0xFFFFFFFF, 0xFFFFFFDF, 0x1, 0x10, 0x40)]
    out += [(rng.randint(-1, 4), rng.getrandbits(32)) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(UNIT + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(TYPE + 0x218, struct.pack("<i", row[0]))
        uc.mem_write(TYPE + 0x2B4, struct.pack("<I", row[1]))
        r = signed(m.call(ENTRY, (), ecx=UNIT))
        print(f"{row[0]} {row[1]} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
