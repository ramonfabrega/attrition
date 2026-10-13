#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_data_get_whose.py — `WorldData::get_whose@006b4d80` under unicorn (item 1824).

    uv run tools/emu/sweep_world_data_get_whose.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world, two stack arguments (`ret 8`) pointers to `WCoord` words (x, then y): `wdata[y * xs + x].who`, a signed byte at `+0xf` of a 0x1c-byte record (`xs` at +0, `wdata` at `+0x134`). Rows `<xs> <x> <y> <who> <decoy>`; every other tile carries `decoy`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006b4d80
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000


def rows():
    rng = random.Random(1824)
    out = []
    for xs in (4, 8, 13):
        for x, y in ((0, 0), (xs - 1, 0), (0, 3), (xs - 1, 3), (xs // 2, 2)):
            for w in (0, 1, 7, 127, -1, -128, -57):
                for d in (0, 5, -3):
                    out.append((xs, x, y, w, d))
    for _ in range(300):
        xs = rng.randint(1, 16)
        out.append((xs, rng.randrange(xs), rng.randrange(4), rng.randint(-128, 127), rng.randint(-128, 127)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x1000)
    for n, row in enumerate(rows()):
        xs, x, y, w, d = row
        TD = OBJ + 0x10000
        uc.mem_write(OBJ, struct.pack("<i", xs))
        uc.mem_write(OBJ + 0x134, struct.pack("<I", TD))
        for i in range(xs * 4):
            uc.mem_write(TD + i * 0x1C, b"\0" * 0x1C)
            uc.mem_write(TD + i * 0x1C + 0xF, struct.pack("<b", d))
        uc.mem_write(TD + (y * xs + x) * 0x1C + 0xF, struct.pack("<b", w))
        uc.mem_write(ARG, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARG, ARG + 4), ecx=OBJ))
        print(f"{xs} {x} {y} {w} {d} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
