#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_is_captain.py — `UnitData::is_captain@0046ceb0` under unicorn.

    uv run tools/emu/sweep_unit_is_captain.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the unit; answers `o_up < 0`, `o_up` the signed short at unit+0x8e.
One row per input, `<o_up> -> <result>`: the short's edges and a seeded band.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046ceb0
UNIT, TYPE = 0x10000000, 0x10010000


def rows():
    rng = random.Random(1691)
    out = [(v,) for v in (0, 1, -1, 2, -2, 0x7FFF, -0x8000, 0x100, -0x100, 0x4000, -0x4000)]
    out += [(rng.randint(-0x8000, 0x7FFF),) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(UNIT + 0x8E, struct.pack("<h", row[0]))
        r = signed(m.call(ENTRY, (), ecx=UNIT))
        print(f"{row[0]} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
