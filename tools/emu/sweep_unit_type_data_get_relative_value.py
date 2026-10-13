#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_type_data_get_relative_value.py — `UnitTypeData::get_relative_value@0061d740` under unicorn.

    uv run tools/emu/sweep_unit_type_data_get_relative_value.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record, one stack argument `i`; the signed short at `+0x2b0 + i*2`, sign-extended. Rows: `i v -> result` over decoys in every neighbouring slot.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0061d740
OBJ = 0x10000000
TYPE = 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_map(TYPE, 0x2000)
    rng = random.Random(1735)
    rows = [(i, v) for i in (0, 1, 2, 7, 31, 49, 50, 63) for v in (0, 1, -1, 100, 0x7FFF, -0x8000, 0x1234, -0x1234)]
    rows += [(rng.randint(0, 80), rng.randint(-0x8000, 0x7FFF)) for _ in range(150)]
    for i, v in rows:
        uc.mem_write(OBJ, bytes(0x2000))
        for j in range(0, 100):
            uc.mem_write(OBJ + 0x2b0 + j * 2, struct.pack("<h", 0x1111 + j))
        uc.mem_write(OBJ + 0x2b0 + i * 2, struct.pack("<h", v))
        print(f"{i} {v} -> {signed(m.call(ENTRY, (i,), ecx=OBJ))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
