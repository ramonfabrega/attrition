#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_is_valid_int.py — `WorldData::is_valid(int,int)@00471170` under unicorn (item 1834).

    uv run tools/emu/sweep_world_data_is_valid_int.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world, two stack ints (`ret 8`): 1 when `0 <= x < xs` (`+0`) and `0 <= y < ys` (`+4`). Rows `<xs> <ys> <x> <y>`: a 400-point grid and a hundred random.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00471170
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
    for xs in (4, 10):
        for ys in (3, 10):
            for x in edges:
                for y in edges:
                    out.append((xs, ys, x, y))
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
        uc.mem_write(OBJ, struct.pack("<ii", xs, ys))
        r = signed(m.call(ENTRY, (x, y), ecx=OBJ))
        print(f"{xs} {ys} {x} {y} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
