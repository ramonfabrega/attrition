#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_has_repeat_air.py — `UnitData::has_repeat_air@0046cec0` under unicorn.

    uv run tools/emu/sweep_unit_has_repeat_air.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the record; `unit_masks` (`+0x68`) `& 0x200000`, returned as the masked value.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046cec0
OBJ = 0x10000000


def rows():
    rng = random.Random(1705)
    return [(v,) for v in (0, 1, 0x1000, 0x200000, 0x201000, 0xFFFFFFFF, 0xFFDFFFFF, 0xFFFFEFFF, 0xFFDFEFFF, 0x80000000, 0x7FFFFFFF)] + [(1 << b,) for b in range(32)] + [(0xFFFFFFFF ^ (1 << b),) for b in range(32)] + [(rng.getrandbits(32),) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x1000)
    for row in rows():
        uc.mem_write(OBJ + 0x68, struct.pack("<I", row[0]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
