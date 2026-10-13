#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_target_order_get_target_order.py — `TargetOrder::get_target_order@0047ff40` under unicorn.

    uv run tools/emu/sweep_target_order_get_target_order.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the order's embedded sub-object; returns `ecx - 0x18` (the enclosing `TargetOrder`). Rows: `this -> result`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0047ff40
OBJ = 0x10000000
TYPE = 0x10010000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)

def ints(seed=1740):
    rng = random.Random(seed)
    return list(INT_EDGES) + [rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(200)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x2000)
    uc.mem_map(TYPE, 0x2000)
    for t in ints():
        print(f"{t} -> {signed(m.call(ENTRY, (), ecx=t))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
