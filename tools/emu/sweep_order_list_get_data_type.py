#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_order_list_get_data_type.py — `OrderList::get_data_type@0046ed70` under unicorn (item 1856).

    uv run tools/emu/sweep_order_list_get_data_type.py <install>/riseofnations.exe

`__thiscall`, one stack argument (`ret 4`): the order pointer; its vtable slot `+0x10` is called with `ecx` = the order and its answer returned. The slot here is `lea eax, [ecx + v]`, so the answer carries `this` and `v`. Row `<order> <v> -> <result>`.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046ed70
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
    n = 0
    for order in (OBJ, OBJ + 0x40, OBJ + 0x1fff0, OBJ + 0x8000):
        for v in EDGES + tuple(range(-30, 30, 4)):
            uc.mem_write(order, struct.pack("<I", VT))
            uc.mem_write(VT + 0x10, struct.pack("<I", CODE + n * 0x10))
            uc.mem_write(CODE + n * 0x10, b"\x8d\x81" + struct.pack("<i", v) + b"\xc3")
            n += 1
            print(f"{order} {v} -> {signed(m.call(ENTRY, (order,), ecx=ARG))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
