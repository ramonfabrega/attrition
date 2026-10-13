#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_group_attack_to_order_get_group_attack_to_order.py — `GroupAttackToOrder::get_group_attack_to_order@004825e0` under unicorn (item 1856).

    uv run tools/emu/sweep_group_attack_to_order_get_group_attack_to_order.py <install>/riseofnations.exe

`__thiscall`, no arguments: `lea eax, [ecx - 0x70]` — the embedded base object's address, `this - 0x70`. Row `<this> -> <result>`.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x004825e0
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
    for this in EDGES + tuple(range(0x70, 0x800, 0x13)) + tuple(range(-0x200, 0, 0x37)) + (0x00400000, 0x7FFFFFF0):
        print(f"{signed(this & 0xFFFFFFFF)} -> {signed(m.call(ENTRY, (), ecx=this))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
