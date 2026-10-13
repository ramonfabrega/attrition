#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_animal_update_los.py — `Animal::update_los@005d7d50` under unicorn.

    uv run tools/emu/sweep_animal_update_los.py <install>/riseofnations.exe

`__thiscall`, `ecx` = this; writes 0 to the byte `field_0x3c` and returns 0. Rows: `<offset> -> <byte there after>` over 0x3b..0x3f on an 0xa5 fill, and offset -1 for the return value.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x005d7d50
OBJ = 0x10000000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints():
    rng = random.Random(1724)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for off in (0x3b, 0x3c, 0x3d, 0x3e, 0x3f, -1):
        uc.mem_write(OBJ, b"\xa5" * 0x1000)
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        if off < 0:
            print(f"{off} -> {r}")
            continue
        (w,) = struct.unpack("<B", bytes(uc.mem_read(OBJ + off, 1)))
        print(f"{off} -> {w}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
