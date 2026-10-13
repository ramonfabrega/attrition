#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_is_tower.py — `BuildTypeData::is_tower@00472a30` under unicorn (item 1824).

    uv run tools/emu/sweep_build_type_data_is_tower.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record, whose vtable slot `+0x60` is laid as the original's `TypeData::is@004771c0` (so the answer is what the dispatch does with its constant): answers `type(+4) == 0x1b7`. The rows are the constant, its neighbours and the other four constants, and two hundred random words.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00472a30
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000


def rows():
    rng = random.Random(1824)
    out = []
    k = 0x1b7
    for t in (k, k - 1, k + 1, 0, -1, 0x7FFFFFFF, -0x80000000, 0x19e, 0x1a3, 0x1b0, 0x1b7, 0x1bb):
        out.append((t,))
    for _ in range(200):
        out.append((rng.choice((k, rng.randint(-2000, 2000), rng.getrandbits(32) - 0x80000000)),))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x1000)
    for n, row in enumerate(rows()):
        t = row[0]
        uc.mem_write(VT + 0x60, struct.pack("<I", 0x004771C0))
        uc.mem_write(OBJ, struct.pack("<I", VT))
        uc.mem_write(OBJ + 4, struct.pack("<i", t))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{t} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
