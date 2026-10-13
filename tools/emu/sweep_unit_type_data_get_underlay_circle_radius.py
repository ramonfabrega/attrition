#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_type_data_get_underlay_circle_radius.py — `UnitTypeData::get_underlay_circle_radius@0061daf0` under unicorn (item 1851).

    uv run tools/emu/sweep_unit_type_data_get_underlay_circle_radius.py <install>/riseofnations.exe

`__thiscall`, no arguments: the int at `+0x234` times `unit_block_radius` at `+0x10` of the `GameAccessConst::constantsc` cell `0xc061e4` (32-bit wrapping). Row `<r> <block>`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0061daf0
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000
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


def setup():
    m = Machine(Image(sys.argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x20000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x40000)
    return m, uc


def main(argv):
    m, uc = setup()
    C = OBJ + 0x4000
    uc.mem_write(0x00C061E4, struct.pack("<I", C))
    for r in VALS:
        for b in VALS:
            uc.mem_write(OBJ + 0x234, struct.pack("<i", r))
            uc.mem_write(C + 0x10, struct.pack("<i", b))
            print(f"{r} {b} -> {signed(m.call(ENTRY, (), ecx=OBJ))}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
