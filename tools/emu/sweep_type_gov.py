#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_gov.py — `TypeData::is_gov_type@004706a0` under unicorn.

    uv run tools/emu/sweep_type_gov.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record; `0x26e < t < 0x275` on `type` (+0x4), signed.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x004706a0
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
