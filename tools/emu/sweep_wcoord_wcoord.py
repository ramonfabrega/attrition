#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wcoord_wcoord.py — `WCoord::WCoord@00461450` under unicorn (item 1819).

    uv run tools/emu/sweep_wcoord_wcoord.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the `WCoord`, one stack argument: stores it at +0 and returns `this`; the printed result is the word stored.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00461450
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    words = list(EDGES) + [random.Random(1819).randint(-0x80000000, 0x7FFFFFFF) for _ in range(150)]
    for w in words:
        uc.mem_write(OBJ, struct.pack("<I", 0xDEADBEEF))
        p = m.call(ENTRY, (w,), ecx=OBJ)
        assert p == OBJ, hex(p)
        (r,) = struct.unpack("<i", bytes(uc.mem_read(OBJ, 4)))
        print(f"{w} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
