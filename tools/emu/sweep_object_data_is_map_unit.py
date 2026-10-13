#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_is_map_unit.py — `ObjectData::is_map_unit@0046ffa0` under unicorn (item 1824).

    uv run tools/emu/sweep_object_data_is_map_unit.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object, two virtual calls — slot `+8`, then (if non-zero) slot `+0xbc` — laid as stubs (`mov eax, imm; ret`) answering each row's two words: 1 only when both are non-zero, else 0.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046ffa0
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000


def rows():
    rng = random.Random(1824)
    out = []
    vals = (0, 1, 2, -1, 0x100, 0x7FFFFFFF, -0x80000000, 0xFF)
    for a in vals:
        for b in vals:
            out.append((a, b))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x1000)
    for n, row in enumerate(rows()):
        a, b = row
        for off, v, slot in ((n * 0x20, a, 0x8), (n * 0x20 + 0x10, b, 0xBC)):
            uc.mem_write(CODE + off, b"\xb8" + struct.pack("<i", v) + b"\xc3")
            uc.mem_write(VT + slot, struct.pack("<I", CODE + off))
        uc.mem_write(OBJ, struct.pack("<I", VT))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{a} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
