#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_good_type_data_get_enhancer.py — `GoodTypeData::get_enhancer@0066fcf0` under unicorn (item 1824).

    uv run tools/emu/sweep_good_type_data_get_enhancer.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the good type, a switch on the dword at `+4` (unsigned compare against 5): 0 → 0x1a7, 1 → 0x1a8, 4 → 0x1a9, 5 → 0x1aa, anything else (2, 3, above 5, negative) → -1.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0066fcf0
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000


def rows():
    rng = random.Random(1824)
    out = []
    for v in list(range(-4, 14)) + [0x7FFFFFFF, -0x80000000, 0x100, 0xFFFF, -0x100]:
        out.append((v,))
    for _ in range(200):
        out.append((rng.getrandbits(32) - 0x80000000,))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x1000)
    for n, row in enumerate(rows()):
        v = row[0]
        uc.mem_write(OBJ + 4, struct.pack("<i", v))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{v} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
