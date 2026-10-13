#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_special_data_get_radius.py — `SpecialData::get_radius@00740060` under unicorn (item 1819).

    uv run tools/emu/sweep_special_data_get_radius.py <install>/riseofnations.exe

`__thiscall` with no use of `this`; reads the `GameAccessConst::constantsc` cell at 0xc061e4 and returns twice the int `general_radius` at +0xc58, wrapping.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00740060
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    words = list(EDGES) + [random.Random(1819).randint(-0x80000000, 0x7FFFFFFF) for _ in range(150)]
    for w in words:
        uc.mem_write(0xC061E4, struct.pack("<I", ARR))
        uc.mem_write(ARR + 0xC58, struct.pack("<i", w))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{w} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
