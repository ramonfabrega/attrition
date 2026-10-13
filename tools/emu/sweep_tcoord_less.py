#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_tcoord_less.py — `TCoord::operator<@00472b30` under unicorn.

    uv run tools/emu/sweep_tcoord_less.py <install>/riseofnations.exe

`__thiscall`, `ecx` = this `TCoord`, one stack argument (`ret 4`) a pointer to the other; answers `this->value < other->value`, signed 32-bit, in `al`.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00472b30
OBJ = 0x10000000
ARG = 0x20000000


def rows():
    rng = random.Random(1708)
    e = [0, 1, -1, 2, 0x7FFFFFFF, -0x80000000, 0x7FFFFFFE, -0x7FFFFFFF, 0x10000, -0x10000, 255, 256, 0xFFFF, -0xFFFF]
    out = [(a, b) for a in e for b in e]
    out += [(rng.randint(-0x80000000, 0x7FFFFFFF), rng.randint(-0x80000000, 0x7FFFFFFF)) for _ in range(200)]
    out += [(v, v) for v in (rng.randint(-0x80000000, 0x7FFFFFFF) for _ in range(30))]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    for a, b in rows():
        uc.mem_write(OBJ, struct.pack("<i", a))
        uc.mem_write(ARG, struct.pack("<i", b))
        r = m.call(ENTRY, (ARG,), ecx=OBJ) & 0xFF
        print(f"{a} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
