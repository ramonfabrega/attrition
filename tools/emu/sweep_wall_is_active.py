#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_is_active.py — `WallData::is_active@00472350` under unicorn.

    uv run tools/emu/sweep_wall_is_active.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the record; `+0x8` byte `& 4`, returned as the masked value.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00472350
OBJ = 0x10000000


def rows():
    rng = random.Random(1705)
    return [(v,) for v in range(256)]


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x1000)
    for row in rows():
        uc.mem_write(OBJ + 0x8, struct.pack("<B", row[0]))
        r = signed(m.call(ENTRY, (), ecx=OBJ))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
