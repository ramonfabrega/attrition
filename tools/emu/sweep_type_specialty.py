#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_specialty.py — `TypeData::is_specialty_type@00470570` under unicorn.

    uv run tools/emu/sweep_type_specialty.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record; `t == 0x68 || t == 0x83` on `type` (+0x4), signed.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00470570
TYPE = 0x10010000


def rows():
    return [(i,) for i in range(-2, 0x340)] + [(-1000,), (0x7FFFFFFF,), (-0x80000000,), (0x10000,)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(TYPE + 0x4, struct.pack("<i", row[0]))
        r = signed(m.call(ENTRY, (), ecx=TYPE))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
