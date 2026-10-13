#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_mountains_out_mnt_locs.py — `MountainsOut::mnt_locs@0089be90` under unicorn (item 1856).

    uv run tools/emu/sweep_mountains_out_mnt_locs.py <install>/riseofnations.exe

`__thiscall`, no arguments: `[[this]+8] + 0xa0 + this` — the vtable's slot-8 word is a displacement. Row `<this> <disp> -> <result>`.
One row per input, `<args…> -> <result>`.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0089be90
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
    for this in (OBJ, OBJ + 0x8000, OBJ + 0x1f000):
        for off in EDGES + tuple(range(-40, 40, 3)):
            uc.mem_write(this, struct.pack("<I", VT))
            uc.mem_write(VT + 8, struct.pack("<i", off))
            print(f"{this} {off} -> {signed(m.call(ENTRY, (), ecx=this))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
