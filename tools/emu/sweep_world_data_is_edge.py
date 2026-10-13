#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_is_edge.py — `WorldData::is_edge@0047a970` under unicorn (item 1840).

    uv run tools/emu/sweep_world_data_is_edge.py <install>/riseofnations.exe

`__thiscall` with no use of `this`; two stack pointers (`ret 8`) to `WCoord` words (x, then y); reads the `GameAccess::world` cell at 0xc06188 (`xs` at `+0`, `ys` at `+4`): 1 when `x == 0`, `y == 0`, `x == xs - 1` or `y == ys - 1`. Rows `<xs> <ys> <x> <y>`; x and y may lie outside the map.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0047a970
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000
TVT = 0x32000000
TYPE = OBJ + 0x8000
WORLD_CELL = 0x00C06188
VALS = (0, 1, 2, -1, 0x100, 0x7FFFFFFF, -0x80000000, 0xFF, -7, 12345)


def stub(uc, off, v, ret=0):
    """`mov eax, v; ret` (or `ret N`) at CODE+off; the address is fresh per row, as unicorn serves a stale block for rewritten code."""
    tail = b"\xc3" if not ret else b"\xc2" + struct.pack("<H", ret)
    uc.mem_write(CODE + off, b"\xb8" + struct.pack("<i", v) + tail)
    return CODE + off


def argstub(uc, off, v):
    """`eax = v + 1000*[esp+4] + [esp+8]; ret 8`: the answer carries the two stack arguments the caller pushed."""
    code = b"\x8b\x44\x24\x04" + b"\x69\xc0" + struct.pack("<i", 1000) + b"\x03\x44\x24\x08" + b"\x05" + struct.pack("<i", v) + b"\xc2\x08\x00"
    uc.mem_write(CODE + off, code)
    return CODE + off


def rows():
    out = []
    for xs, ys in ((1, 1), (2, 3), (8, 6), (64, 64)):
        for x in (-1, 0, 1, xs - 2, xs - 1, xs, xs // 2):
            for y in (-1, 0, 1, ys - 2, ys - 1, ys, ys // 2):
                out.append((xs, ys, x, y))
    rng = random.Random(1840)
    for _ in range(200):
        xs, ys = rng.randint(1, 20), rng.randint(1, 20)
        out.append((xs, ys, rng.randint(-2, xs + 1), rng.randint(-2, ys + 1)))
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
        uc.mem_write(WORLD_CELL, struct.pack("<I", OBJ))
        uc.mem_write(OBJ, struct.pack("<ii", xs, ys))
        uc.mem_write(ARG, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARG, ARG + 4), ecx=OBJ))
        print(f"{xs} {ys} {x} {y} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
