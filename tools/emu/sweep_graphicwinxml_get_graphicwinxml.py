#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_graphicwinxml_get_graphicwinxml.py — `GraphicWinXML::get_graphicwinxml@0041c000` under unicorn.

    uv run tools/emu/sweep_graphicwinxml_get_graphicwinxml.py <install>/riseofnations.exe

`__thiscall`; returns `ecx` unchanged (`mov eax, ecx`).
One row per input, `<word> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x000041c000
OBJ = 0x10000000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints(seed=1782):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for t in ints():
        print(f"{t} -> {signed(m.call(ENTRY, (), ecx=t))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
