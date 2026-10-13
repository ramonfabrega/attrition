#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_support_cost.py — `UnitData::support_cost@0060fde0` under unicorn.

    uv run tools/emu/sweep_unit_data_support_cost.py <install>/riseofnations.exe

`__thiscall`, `ecx` = this; the constant 0; `this` and the argument are never read.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0060fde0
OBJ = 0x10000000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def ints():
    rng = random.Random(1724)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    for v in ints():
        r = signed(m.call(ENTRY, (v,), ecx=OBJ))
        print(f"{v} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
