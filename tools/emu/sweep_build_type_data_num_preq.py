#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_num_preq.py — `BuildTypeData::num_preq@00472a60` under unicorn.

    uv run tools/emu/sweep_build_type_data_num_preq.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record; returns 3 and reads nothing, so the rows fill the record with seeded words and ask each time.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00472a60
OBJ = 0x10000000
TYPE = 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)

def ints(seed=1736):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_map(TYPE, 0x2000)
    for w in ints():
        uc.mem_write(OBJ, struct.pack("<i", w) * 0x400)
        print(f"{w} -> {signed(m.call(ENTRY, (), ecx=OBJ))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
