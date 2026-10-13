#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_type_data_get_stance_type.py — `ObjectTypeData::get_stance_type@00476aa0` under unicorn (item 1814, tools/sweep_gen.py).

    uv run tools/emu/sweep_object_type_data_get_stance_type.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object (never read).
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00476aa0
OBJ, TYPE = 0x10000000, 0x10100000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    return [(OBJ + d,) for d in (0, 4, 0x100, 0x1FFFC, 0x8000)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        pass
        r = signed(m.call(ENTRY, (), ecx=row[0]))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
