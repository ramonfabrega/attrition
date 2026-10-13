#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_get_wdata.py — `World::get_wdata@0046d220` under unicorn (item 1856).

    uv run tools/emu/sweep_world_get_wdata.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): pointers to two `WCoord` words (x, y); `[this+0x134] + 28 * (y * [this] + x)`, 32-bit wrapping. Row `<xs> <x> <y> <base> -> <result>`.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046d220
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000
EDGES = (0, 1, 2, -1, 0x70, 0x100, 0x7FFFFFFF, -0x80000000, 0xFF, -7, 12345, 0x12345678, -0x70, 0x1000)


def setup():
    m = Machine(Image(sys.argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x20000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x40000)
    return m, uc

def main(argv):
    m, uc = setup()
    n = 0
    for xs in (1, 2, 6, 9, 100, 0x10000, -3):
        for x in (0, 1, 5, -1, 0x7FFFFFFF):
            for y in (0, 1, 7, -2, 0x12345):
                for base in (0, 0x20000000, -0x100):
                    uc.mem_write(OBJ, struct.pack("<i", xs))
                    uc.mem_write(OBJ + 0x134, struct.pack("<i", base))
                    uc.mem_write(ARG, struct.pack("<ii", x, y))
                    print(f"{xs} {x} {y} {base} -> {signed(m.call(ENTRY, (ARG, ARG + 4), ecx=OBJ))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
