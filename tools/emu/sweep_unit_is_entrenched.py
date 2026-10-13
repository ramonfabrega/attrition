#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_is_entrenched.py — `UnitData::is_entrenched@0060a810` under unicorn.

    uv run tools/emu/sweep_unit_is_entrenched.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the unit, one stack argument; `unit_masks` (+0x68) `& 0x2000000`, and with a non-zero argument only while `unit_masks2` (+0x6c) `& 0x20000` is clear. Row: `<masks> <masks2> <param>`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0060a810
UNIT = 0x10000000


def rows():
    rng = random.Random(1704)
    out = [(a, b, p) for a in (0, 0x2000000, 0xFFFFFFFF, 0xFDFFFFFF) for b in (0, 0x20000, 0xFFFFFFFF, 0xFFFDFFFF) for p in (0, 1, -1, 2, 0x7FFFFFFF)]
    out += [(rng.getrandbits(32) | (rng.getrandbits(1) << 25), rng.getrandbits(32), rng.choice((0, 1, rng.getrandbits(8)))) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    for row in rows():
        uc.mem_write(UNIT + 0x68, struct.pack("<II", row[0], row[1]))
        r = signed(m.call(ENTRY, (row[2],), ecx=UNIT))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
