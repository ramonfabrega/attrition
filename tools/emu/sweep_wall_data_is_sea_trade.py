#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_data_is_sea_trade.py — `WallData::is_sea_trade@004722f0` under unicorn (item 1840).

    uv run tools/emu/sweep_wall_data_is_sea_trade.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the wall: pushes `(0x1b0, 0)`; if virtual slot `+0xb8` is `ObjectData::is@00653790` it calls the type record's (`*(this+0x18)`) vtable slot `+0x60` with them, else slot `+0xb8` itself. Both callees here are stubs answering `v + 1000*arg0 + arg1` (`ret 8`), so the row shows which one ran and what it was passed. Rows `<mode> <sv> <tv>`: mode 0 = slot `+0xb8` holds `ObjectData::is` (type stub `tv`), mode 1 = slot `+0xb8` is a stub `sv` (and the type stub `tv` must not run).
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x004722f0
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000
TVT = 0x32000000
TYPE = OBJ + 0x8000
WORLD_CELL = 0x00C06188
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


def rows():
    out = []
    for mode in (0, 1):
        for sv in (0, 1, -1, 77):
            for tv in (0, 1, -1, 500, 12345):
                out.append((mode, sv, tv))
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
        mode, sv, tv = row
        uc.mem_write(OBJ, struct.pack("<I", VT))
        uc.mem_write(OBJ + 0x18, struct.pack("<I", TYPE))
        uc.mem_write(TYPE, struct.pack("<I", TVT))
        uc.mem_write(TVT + 0x60, struct.pack("<I", argstub(uc, n * 0x40, tv)))
        slot = 0x00653790 if mode == 0 else argstub(uc, n * 0x40 + 0x20, sv)
        uc.mem_write(VT + 0xB8, struct.pack("<I", slot))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{mode} {sv} {tv} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
