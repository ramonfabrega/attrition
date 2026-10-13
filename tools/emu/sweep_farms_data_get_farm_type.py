#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_farms_data_get_farm_type.py — `FarmsData::get_farm_type@008d9160` under unicorn (item 1819).

    uv run tools/emu/sweep_farms_data_get_farm_type.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the farms data, one stack argument: `farm_count` at +0x18, the record array behind +0x24 (0xc0 a record); for an index in `0..farm_count` the low bit of the byte at record+0xbd, else 0 (WHEAT_FARM). The record at the index carries the byte, every other record its complement.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x008d9160
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    for count in (0, 1, 2, 3, -1, -0x80000000):
        for idx in (-3, -2, -1, 0, 1, 2, 3, 4, 0x7FFFFFFF, -0x80000000):
            for b in (0, 1, 2, 3, 0xFE, 0xFF):
                uc.mem_write(ARR, bytes(0x1000))
                uc.mem_write(OBJ + 0x18, struct.pack("<i", count))
                uc.mem_write(OBJ + 0x24, struct.pack("<I", ARR))
                for i in range(4):
                    v = b if i == idx else (~b) & 0xFF
                    uc.mem_write(ARR + i * 0xC0 + 0xBD, bytes([v]))
                r = signed(m.call(ENTRY, (idx,), ecx=OBJ))
                print(f"{count} {idx} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
