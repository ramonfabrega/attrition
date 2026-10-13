#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_is_valid_tcoord.py — `WorldData::is_valid@0046f760` under unicorn (item 1834).

    uv run tools/emu/sweep_world_data_is_valid_tcoord.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world, two stack pointers (`ret 8`) to `TCoord` words: 1 when `0 <= x < tile_xs` (`+0x18`) and `0 <= y < tile_ys` (`+0x1c`). Rows `<tile_xs> <tile_ys> <x> <y>`: a 400-point grid and a hundred random.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046f760
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000
TVT = 0x32000000
TYPE = OBJ + 0x1000
VALS = (0, 1, 2, -1, 0x100, 0x7FFFFFFF, -0x80000000, 0xFF, -7, 12345)


def stub(uc, off, v, ret=0):
    """`mov eax, v; ret` (or `ret N`) at CODE+off; the address is fresh per row, as unicorn serves a stale block for rewritten code."""
    tail = b"\xc3" if not ret else b"\xc2" + struct.pack("<H", ret)
    uc.mem_write(CODE + off, b"\xb8" + struct.pack("<i", v) + tail)
    return CODE + off


def rows():
    out = []
    edges = (-5, -1, 0, 1, 3, 4, 5, 9, 10, 11)
    for lim_x in (4, 10):
        for lim_y in (3, 10):
            for x in edges:
                for y in edges:
                    out.append((lim_x, lim_y, x, y))
    rng = random.Random(1834)
    for _ in range(100):
        out.append((rng.randint(1, 20), rng.randint(1, 20), rng.randint(-3, 22), rng.randint(-3, 22)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x20000)
    uc.mem_map(TVT, 0x1000)
    for n, row in enumerate(rows()):
        xs, ys, x, y = row
        uc.mem_write(OBJ + 0x18, struct.pack("<ii", xs, ys))
        uc.mem_write(ARG, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARG, ARG + 4), ecx=OBJ))
        print(f"{xs} {ys} {x} {y} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
