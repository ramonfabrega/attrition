#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_is_hero.py — `UnitData::is_hero@0046ce60` under unicorn.

    uv run tools/emu/sweep_unit_is_hero.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the unit (its type pointer at +0x18); returns the word at type+0x2b8 masked with 0x20.
One row per input, `<word> -> <result>`: every single bit, every all-but-one, all bits, none, seeded random.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046ce60
UNIT, TYPE = 0x10000000, 0x10010000


def rows():
    rng = random.Random(1694)
    edges = [0, 0xFFFFFFFF] + [1 << b for b in range(32)] + [0xFFFFFFFF ^ (1 << b) for b in range(32)]
    out = [(f,) for f in edges] + [(rng.getrandbits(32),) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(TYPE + 0x2b8, struct.pack("<I", row[0]))
        uc.mem_write(UNIT + 0x18, struct.pack("<I", TYPE))
        r = signed(m.call(ENTRY, (), ecx=UNIT))
        print(f"{row[0]} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
