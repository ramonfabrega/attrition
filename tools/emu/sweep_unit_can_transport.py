#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_can_transport.py — `UnitData::can_transport@0046f960` under unicorn.

    uv run tools/emu/sweep_unit_can_transport.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the unit; `unit_masks` (+0x68) `& 0x800000` unless `unit_masks2` (+0x6c) `& 0x2000`, or the type byte at +0x2b4 `& 0x10`. Row: `<masks> <masks2> <type byte>`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046f960
UNIT, TYPE = 0x10000000, 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1701)
    words = [0, 0xFFFFFFFF, 0x800000, 0xFFFFFFFF ^ 0x800000, 0x2000, 0xFFFFFFFF ^ 0x2000]
    out = [(a, b, c) for a in (0, 0x800000, 0xFFFFFFFF, 0x7FFFFF) for b in (0, 0x2000, 0xFFFFFFFF, 0xFFFFDFFF) for c in (0, 0x10, 0xEF, 0xFF, 1, 0x20)]
    out += [(rng.choice(words + [rng.getrandbits(32)]), rng.choice(words + [rng.getrandbits(32)]), rng.getrandbits(8)) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(UNIT + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(UNIT + 0x68, struct.pack("<II", row[0], row[1]))
        uc.mem_write(TYPE + 0x2b4, struct.pack("<B", row[2]))
        r = signed(m.call(ENTRY, (), ecx=UNIT))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
