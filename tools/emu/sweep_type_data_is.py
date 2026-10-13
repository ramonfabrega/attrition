#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_type_data_is.py — `TypeData::is@004771c0` under unicorn (item 1824).

    uv run tools/emu/sweep_type_data_is.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the type record, two stack arguments (`ret 8`): answers `type(+4) == arg1`; `arg2` is ignored.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x004771c0
OBJ = 0x10000000
ARG = 0x20000000
VT = 0x30000000
CODE = 0x31000000


def rows():
    rng = random.Random(1824)
    out = []
    for t in (0, 1, -1, 0x19e, 0x1b7, 0x33f, 0x7FFFFFFF, -0x80000000):
        for a in (t, t + 1, t - 1, 0, -1, 0x1a3):
            for b in (0, 1, t, 77):
                out.append((t, a, b))
    for _ in range(300):
        t = rng.randint(-1000, 1000)
        out.append((t, rng.choice((t, rng.randint(-1000, 1000))), rng.randint(-5, 5)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    uc.mem_map(VT, 0x1000)
    uc.mem_map(CODE, 0x1000)
    for n, row in enumerate(rows()):
        t, a, b = (((x + 0x80000000) & 0xFFFFFFFF) - 0x80000000 for x in row)
        uc.mem_write(OBJ + 4, struct.pack("<i", t))
        r = signed(m.call(ENTRY, (a, b), ecx=OBJ))
        print(f"{t} {a} {b} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
