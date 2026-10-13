#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_is_rare_collector.py — `UnitData::is_rare_collector@0046fae0` under unicorn (item 1834).

    uv run tools/emu/sweep_unit_data_is_rare_collector.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the unit, `+0x18` its type record, whose index at `+4` is tested against 0x3d, 0x3e and 400, then (vtable slot `+0x60`, laid as the original's `TypeData::is@004771c0`) against 0x13d: 1 when the index is one of the four. The rows are the four, their neighbours, the edges and a hundred random indices.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046fae0
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
    ids = (0x3D, 0x3E, 400, 0x13D, 0x3C, 0x3F, 399, 401, 0x13C, 0x13E, 0, -1, 0x7FFFFFFF)
    for t in ids:
        out.append((t,))
    rng = random.Random(1834)
    for _ in range(100):
        out.append((rng.choice((rng.randint(0, 800), rng.getrandbits(32) - 0x80000000)),))
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
        t = row[0]
        uc.mem_write(OBJ + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(TYPE, struct.pack("<I", TVT))
        uc.mem_write(TYPE + 4, struct.pack("<i", t))
        uc.mem_write(TVT + 0x60, struct.pack("<I", 0x004771C0))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{t} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
