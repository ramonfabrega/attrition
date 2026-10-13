#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_is_caravan.py — `UnitData::is_caravan@0046ce90` under unicorn (item 1834).

    uv run tools/emu/sweep_unit_data_is_caravan.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the unit, a tail jump through slot `+0x130` of its type record's (`+0x18`) vtable, laid as a stub (`mov eax, imm; ret`) answering the row's word: the function answers that word unchanged. The rows are ten edge words and a hundred random ones.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046ce90
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
    out = [(v,) for v in VALS]
    rng = random.Random(1834)
    for _ in range(100):
        out.append((rng.getrandbits(32) - 0x80000000,))
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
        uc.mem_write(TVT + 0x130, struct.pack("<I", stub(uc, n * 0x40, t)))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{t} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
