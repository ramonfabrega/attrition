#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_group_action_begin.py — `Group::action_begin@00714100` under unicorn.

    uv run tools/emu/sweep_group_action_begin.py <install>/riseofnations.exe

`__thiscall`, `ecx` = this; writes 0 to the dword at `this+0x28`. Rows: `<offset> -> <dword after>` over 0x24/0x28/0x2c on an 0xa5 fill.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00714100
OBJ = 0x10000000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints():
    rng = random.Random(1724)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for off in (0x24, 0x28, 0x2c):
        uc.mem_write(OBJ, b"\xa5" * 0x1000)
        m.call(ENTRY, (), ecx=OBJ)
        (w,) = struct.unpack("<i", bytes(uc.mem_read(OBJ + off, 4)))
        print(f"{off} -> {w}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
