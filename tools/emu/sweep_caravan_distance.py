#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_caravan_distance.py — `Caravan::distance@0073d380` under unicorn (item 1966, tools/sweep_gen.py).

    uv run tools/emu/sweep_caravan_distance.py <install>/riseofnations.exe

`__thiscall` (cdecl by its decompile, `ecx` is the argument `e`); the pointer in the global cell 0x00c06188, the int at +0x0 of its record.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402
from sweeplib import OBJ, TYPE, REC, OUT, cell, setup  # noqa: E402,F401
from unicorn.x86_const import UC_X86_REG_ESP  # noqa: E402,F401

ENTRY = 0x0073d380
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)
CELL = 0x00c06188


def rows():
    rng = random.Random(1966)
    out = [(e, v) for v in INT_EDGES for e in INT_EDGES]
    for _ in range(300):
        v = rng.choice([rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-1000, 1000), rng.randint(0, 5000)])
        # ecx lands near the three cuts (n/4, n/2, 4n/5) as often as anywhere
        cut = rng.choice([v >> 2, v // 2, (v << 2) // 5 if abs(v) < 0x20000000 else 0, 0])
        e = max(-0x80000000, min(0x7FFFFFFF, cut + rng.randint(-2, 2) if rng.random() < 0.7 else rng.randint(-0x80000000, 0x7FFFFFFF)))
        out.append((e, v))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    setup(uc)
    for row in rows():
        cell(uc, CELL, REC)
        uc.mem_write(REC + 0x0, struct.pack("<i", row[1]))
        r = signed(m.call(ENTRY, (), ecx=row[0] & 0xFFFFFFFF))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
