#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_terrain_map_get_terrain_type.py — `TerrainMap::get_terrain_type@009b5ec0` under unicorn (item 1856).

    uv run tools/emu/sweep_terrain_map_get_terrain_type.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `x`, `y`; the byte at `[this+0x9c8] + [[0xc06188]] * y + x` (`GameAccess::world`'s first word is the map width). `eax` keeps `x`'s upper 24 bits (callers read `al`), so the row prints `al`. Row `<width> <x> <y> <byte> -> <al>`.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x009b5ec0
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
    W = ARG + 0x1000
    TT = OBJ + 0x10000
    for width in (1, 3, 6, 9, 40):
        uc.mem_write(0x00C06188, struct.pack("<I", W))
        uc.mem_write(W, struct.pack("<i", width))
        uc.mem_write(OBJ + 0x9C8, struct.pack("<I", TT))
        for x, y in ((0, 0), (width - 1, 0), (0, 3), (width - 1, 3), (width // 2, 2), (width, 1), (1, 5), (0, 10)):
            for byte in (0, 1, 2, 7, 0x80, 0xFF, 0x55):
                uc.mem_write(TT, b"\xaa" * 0x800)
                uc.mem_write(TT + width * y + x, bytes([byte]))
                print(f"{width} {x} {y} {byte} -> {m.call(ENTRY, (x, y), ecx=OBJ) & 0xFF}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
