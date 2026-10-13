#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_good_type_data_avail_ctw.py — `GoodTypeData::avail_ctw@0066fd50` under unicorn (item 1819).

    uv run tools/emu/sweep_good_type_data_avail_ctw.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type; reads the `GameAccessConst::gamec` cell at 0xc061e8: with bit 0x2 of the byte at game+0x822 set, true when the dword `exclude_ctw` at +0x2f4 is 0, else true.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0066fd50
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    rows = [(b, e) for b in range(256) for e in (0, 1, -1, 0x100)]
    rows += [(b, e) for e in EDGES for b in (0, 2, 3, 0xFD)]
    for b, e in rows:
        uc.mem_write(0xC061E8, struct.pack("<I", ARR))
        uc.mem_write(ARR + 0x822, bytes([b]))
        uc.mem_write(OBJ + 0x2F4, struct.pack("<i", e))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{b} {e} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
