#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_farms_data_get_wheat_status.py — `FarmsData::get_wheat_status@008d9190` under unicorn (item 1819).

    uv run tools/emu/sweep_farms_data_get_wheat_status.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the farms data, three stack arguments `(a, b, c)`: the byte at array+`(a*0x30 + c)*4 + b + 0xac`, the array behind +0x24 holding `(o*31+7) & 0xff` at offset `o`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x008d9190
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    for o in range(0x2000):
        uc.mem_write(ARR + o, bytes([(o * 31 + 7) & 0xFF]))
    uc.mem_write(OBJ + 0x24, struct.pack("<I", ARR))
    for a in range(4):
        for b in range(4):
            for c in range(0x30):
                r = signed(m.call(ENTRY, (a, b, c), ecx=OBJ))
                print(f"{a} {b} {c} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
