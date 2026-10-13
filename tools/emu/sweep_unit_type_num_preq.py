#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_type_num_preq.py — `UnitTypeData::num_preq@00470810` under unicorn.

    uv run tools/emu/sweep_unit_type_num_preq.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record; the body returns 2 and reads nothing, so the
rows fill the record with seeded words and ask each time.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00470810
TYPE = 0x10010000


def rows():
    rng = random.Random(1705)
    return [(0,), (0xFFFFFFFF,)] + [(rng.getrandbits(32),) for _ in range(60)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(TYPE, struct.pack("<I", row[0]) * 0x400)
        r = signed(m.call(ENTRY, (), ecx=TYPE))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
