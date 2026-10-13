#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_order_get_type.py — `BuildOrder::get_type@00482b50` under unicorn.

    uv run tools/emu/sweep_build_order_get_type.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the order (never read); answers the constant `BUILD_AT`, returned in `eax`.
Rows: the `this` pointer, over a few addresses, to show it is not read.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00482b50
OBJ = 0x10000000
ARG = 0x20000000


def rows():
    return [(OBJ + d,) for d in (0, 4, 0x100, 0x1FFFC, 0x8000)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    for row in rows():
        r = signed(m.call(ENTRY, (), ecx=row[0]))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
