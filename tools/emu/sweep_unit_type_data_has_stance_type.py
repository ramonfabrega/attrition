#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_type_data_has_stance_type.py — `UnitTypeData::has_stance_type@0061d3b0` under unicorn (item 1840).

    uv run tools/emu/sweep_unit_type_data_has_stance_type.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record, one stack int (`ret 4`): virtual slot `+0x138` (`v`), 1 when it equals the argument `a`. Rows `<v> <a>`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0061d3b0
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
    out = [(v, a) for v in VALS for a in VALS]
    rng = random.Random(1840)
    for _ in range(100):
        out.append((rng.randint(0, 40), rng.randint(0, 40)))
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
        v, a = row
        uc.mem_write(OBJ, struct.pack("<I", VT))
        uc.mem_write(VT + 0x138, struct.pack("<I", stub(uc, n * 0x20, v)))
        r = signed(m.call(ENTRY, (a,), ecx=OBJ))
        print(f"{v} {a} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
