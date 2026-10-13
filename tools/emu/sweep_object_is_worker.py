#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_is_worker.py — `ObjectData::is_worker@0046fa10` under unicorn.

    uv run tools/emu/sweep_object_is_worker.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; answers 1 when `type+4` is one of 0x32..0x35.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046fa10
UNIT, TYPE = 0x10000000, 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1701)
    out = [(i,) for i in range(0, 0x200)] + [(-1,), (0x7FFFFFFF,), (-0x80000000,)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(UNIT, 0x1000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(UNIT + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(TYPE + 0x4, struct.pack("<i", row[0]))
        r = signed(m.call(ENTRY, (), ecx=UNIT))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
