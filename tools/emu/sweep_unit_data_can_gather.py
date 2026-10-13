#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_can_gather.py — `UnitData::can_gather@00608850` under unicorn (item 1856).

    uv run tools/emu/sweep_unit_data_can_gather.py <install>/riseofnations.exe

`__thiscall`, no arguments: `[[this+0x18]+4]` in `0x32..=0x35` gives 1, else 0. Row `<kind> -> <result>`.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00608850
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000
EDGES = (0, 1, 2, -1, 0x70, 0x100, 0x7FFFFFFF, -0x80000000, 0xFF, -7, 12345, 0x12345678, -0x70, 0x1000)


def setup():
    m = Machine(Image(sys.argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x20000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x40000)
    return m, uc

def main(argv):
    m, uc = setup()
    for kind in tuple(range(0x10, 0x70)) + EDGES + (0x31, 0x36, 0x132, 0x10032, -0x32):
        uc.mem_write(OBJ + 0x18, struct.pack("<I", ARG))
        uc.mem_write(ARG + 4, struct.pack("<i", kind))
        print(f"{kind} -> {signed(m.call(ENTRY, (), ecx=OBJ))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
