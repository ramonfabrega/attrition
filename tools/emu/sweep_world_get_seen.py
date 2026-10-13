#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_get_seen.py — `World::get_seen@006b4290` under unicorn (item 1840).

    uv run tools/emu/sweep_world_get_seen.py <install>/riseofnations.exe

`__thiscall` with no use of `this`; two stack pointers (`ret 8`) to `FCoord` words (x, then y); reads the `GameAccess::world` cell at 0xc06188: the byte at `array[xs * y + x]`, `xs` at `+0xc`, `array` at `+0x15c` (the other array, `+0x160`, carries a different fill). Rows `<xs> <x> <y> <val> <decoy>`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006b4290
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
    for xs in (4, 8, 13, 64):
        for x, y in ((0, 0), (xs - 1, 0), (0, 3), (xs - 1, 3), (xs // 2, 2)):
            for g in (0, 1, 7, 127, 128, 255, 200):
                for d in (0, 5, 255):
                    out.append((xs, x, y, g, d))
    rng = random.Random(1840)
    for _ in range(300):
        xs = rng.randint(1, 64)
        out.append((xs, rng.randrange(xs), rng.randrange(4), rng.randint(0, 255), rng.randint(0, 255)))
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
        xs, x, y, g, d = row
        A, B = OBJ + 0x10000, OBJ + 0x14000
        uc.mem_write(WORLD_CELL, struct.pack("<I", OBJ))
        uc.mem_write(OBJ + 0xC, struct.pack("<i", xs))
        uc.mem_write(OBJ + 0x15C, struct.pack("<I", A))
        uc.mem_write(OBJ + 0x160, struct.pack("<I", B))
        uc.mem_write(A, bytes([d]) * (xs * 4))
        uc.mem_write(B, bytes([g ^ 0xA5]) * (xs * 4))
        uc.mem_write(A + y * xs + x, bytes([g]))
        uc.mem_write(ARG, struct.pack("<ii", x, y))
        r = m.call(ENTRY, (ARG, ARG + 4), ecx=OBJ) & 0xFF
        print(f"{xs} {x} {y} {g} {d} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
