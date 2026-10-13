#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_get_goods.py — `WorldData::get_goods@0046ef10` under unicorn (item 1834).

    uv run tools/emu/sweep_world_data_get_goods.py <install>/riseofnations.exe

`__thiscall` with no use of `this`; two stack pointers (`ret 8`) to `WCoord` words (x, then y); reads the `GameAccess::world` cell at 0xc06188: the unsigned byte at `+0xd` of `wdata[y * xs + x]` (0x1c-byte records, `xs` at +0, `wdata` at `+0x134`). Rows `<xs> <x> <y> <goods> <decoy>`; every other tile carries `decoy`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046ef10
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
    for xs in (4, 8, 13):
        for x, y in ((0, 0), (xs - 1, 0), (0, 3), (xs - 1, 3), (xs // 2, 2)):
            for g in (0, 1, 7, 127, 128, 255, 200):
                for d in (0, 5, 255):
                    out.append((xs, x, y, g, d))
    rng = random.Random(1834)
    for _ in range(300):
        xs = rng.randint(1, 16)
        out.append((xs, rng.randrange(xs), rng.randrange(4), rng.randint(0, 255), rng.randint(0, 255)))
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
        xs, x, y, g, d = row
        WD = OBJ + 0x10000
        uc.mem_write(0x00C06188, struct.pack("<I", OBJ))
        uc.mem_write(OBJ, struct.pack("<i", xs))
        uc.mem_write(OBJ + 0x134, struct.pack("<I", WD))
        for i in range(xs * 4):
            uc.mem_write(WD + i * 0x1C, b"\0" * 0x1C)
            uc.mem_write(WD + i * 0x1C + 0xD, struct.pack("<B", d))
        uc.mem_write(WD + (y * xs + x) * 0x1C + 0xD, struct.pack("<B", g))
        uc.mem_write(ARG, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARG, ARG + 4), ecx=OBJ))
        print(f"{xs} {x} {y} {g} {d} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
