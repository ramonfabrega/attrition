#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wcoord_traverse_y.py — `WCoord::traverse_y@0046eed0` under unicorn (item 1856).

    uv run tools/emu/sweep_wcoord_traverse_y.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): the out pointer, then `y`; stores `((y + ((y >> 31) & 3)) >> 2) + [this] * 4`. Row `<value> <y> -> <stored word>`.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046eed0
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
    for value in EDGES + (3, 100, -100):
        for y in EDGES + (3, 4, 5, 6, 7, 8, -3, -4, -5, -8):
            uc.mem_write(OBJ, struct.pack("<i", value))
            uc.mem_write(ARG, b"\0\0\0\0")
            m.call(ENTRY, (ARG, y), ecx=OBJ)
            out = struct.unpack("<i", uc.mem_read(ARG, 4))[0]
            print(f"{value} {y} -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
