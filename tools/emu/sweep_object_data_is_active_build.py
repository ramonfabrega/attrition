#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_object_data_is_active_build.py — `ObjectData::is_active_build@0046fb40` under unicorn (item 1834).

    uv run tools/emu/sweep_object_data_is_active_build.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object: the flag dword at `+8` (`f`); if bit 0 is set, virtual slot `+0x20` (`b`), then if non-zero slot `+0x4c` (`c`): 1 only when `f & 1`, `b != 0` and `c != 0`. Rows are the 96-point grid of the three.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046fb40
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
    for f in (0, 1, 2, 0xFF, -1, 0x100):
        for b in (0, 1, -1, 0x100):
            for c in (0, 1, -1, 5):
                out.append((f, b, c))
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
        f, b, c = row
        uc.mem_write(OBJ, struct.pack("<I", VT))
        uc.mem_write(OBJ + 8, struct.pack("<i", f))
        uc.mem_write(VT + 0x20, struct.pack("<I", stub(uc, n * 0x40, b)))
        uc.mem_write(VT + 0x4C, struct.pack("<I", stub(uc, n * 0x40 + 0x10, c)))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{f} {b} {c} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
