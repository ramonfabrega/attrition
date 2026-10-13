#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_good_inc_time.py — `Good::inc_time@0066d850` under unicorn.

    uv run tools/emu/sweep_good_inc_time.py <install>/riseofnations.exe

`__thiscall`, a bare `ret`; no argument, no write. Rows: `<fill byte> -> <bytes changed in 0x1000 of the object>` and `eax`-independent.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0066d850
OBJ = 0x10000000
TYPE = 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_map(TYPE, 0x2000)
    for fill in range(0, 256, 5):
        buf = bytes([fill]) * 0x1000
        uc.mem_write(OBJ, buf)
        m.call(ENTRY, (), ecx=OBJ)
        after = bytes(uc.mem_read(OBJ, 0x1000))
        print(f"{fill} -> {sum(1 for x, y in zip(buf, after) if x != y)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
