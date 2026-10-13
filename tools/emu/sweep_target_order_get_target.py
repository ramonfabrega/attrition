#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_target_order_get_target.py — `TargetOrder::get_target@0072ff60` under unicorn (item 1851).

    uv run tools/emu/sweep_target_order_get_target.py <install>/riseofnations.exe

`__thiscall`, no arguments: `whom` at `+0xc`, `ox` at `+8`; the `GameAccess::objects` cell `0xc0618c` points at a table of 28-byte rows whose `+0x14` is an array of object pointers: `table[whom].array[ox]`. Row `<whom> <ox> <val>`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0072ff60
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
    T = OBJ + 0x1000
    uc.mem_write(0x00C0618C, struct.pack("<I", T))
    for whom in range(0, 4):
        for ox in (0, 1, 2, 5, 9):
            for val in (0, 1, -1, 0x12345678, -0x80000000):
                for w in range(0, 4):
                    uc.mem_write(T + 0x14 + w * 28, struct.pack("<I", ARG + 0x100 * (w + 1)))
                    uc.mem_write(ARG + 0x100 * (w + 1), struct.pack("<10i", *[0x01010101 + w] * 10))
                uc.mem_write(ARG + 0x100 * (whom + 1) + ox * 4, struct.pack("<i", val))
                uc.mem_write(OBJ + 8, struct.pack("<ii", ox, whom))
                print(f"{whom} {ox} {val} -> {signed(m.call(ENTRY, (), ecx=OBJ))}")
    return 0

if __name__ == "__main__":
    sys.exit(main(sys.argv))
