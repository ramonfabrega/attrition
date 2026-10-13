#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_type_data_is_civilian.py — `BuildTypeData::is_civilian@00639930` under unicorn (item 1819).

    uv run tools/emu/sweep_build_type_data_is_civilian.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type; `build_flags` at +0x2c0: 0 when bit 0x2000 is set, else bit 4 of the complement.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00639930
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    rng = random.Random(1819)
    words = list(EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(150)]
    words += [lo | hi for lo in range(256) for hi in (0, 0x2000)]
    for w in words:
        uc.mem_write(OBJ + 0x2C0, struct.pack("<i", w))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(f"{w} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
