#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_turn_control_is_cannon_time.py — `TurnControl::is_cannon_time@00956ac0` under unicorn (item 1793, tools/sweep_gen.py).

    uv run tools/emu/sweep_turn_control_is_cannon_time.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the object; reads the int in the global cell 0x00e863f4 and returns whether it is non-negative; one unread stack argument (the rows vary it).
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00956ac0
OBJ, TYPE = 0x10000000, 0x10100000
INT_EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255)


def rows():
    rng = random.Random(1793)
    out = [(v,) for v in INT_EDGES]
    out += [(rng.randint(-0x80000000, 0x7FFFFFFF),) for _ in range(200)]
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(TYPE, 0x1000)
    for row in rows():
        uc.mem_write(0x00e863f4, struct.pack("<i", row[0]))
        r = signed(m.call(ENTRY, (row[0] ^ 0x55,), ecx=OBJ))
        print(" ".join(str(x) for x in row) + f" -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
