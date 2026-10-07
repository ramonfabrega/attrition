#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_gather_point_is_inside.py — `GatherPoint::is_inside@00730400` under unicorn.

    uv run tools/emu/sweep_gather_point_is_inside.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the GatherPoint, no stack arguments. The function reads
`action` (`+0xc`, a byte), `x` (`+0x4`) and `y` (`+0x8`), both 32-bit. One row
per input, `<x> <y> <action> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00730400
PAGE = 0x1000_0000
INT_MIN, INT_MAX = -(1 << 31), (1 << 31) - 1


def rows():
    rng = random.Random(1575)
    xs = [INT_MIN, INT_MIN + 1, -1_000_000, -4096, -2, -1, 0, 1, 2, 4096, 1_000_000, INT_MAX - 1, INT_MAX]
    actions = [0, 1, 2, 3, 4, 255]
    for x in xs:
        for y in xs:
            for a in actions:
                yield x, y, a
    for _ in range(600):
        lim = rng.choice([3, 3_000, 200_000, INT_MAX])
        yield rng.randint(-lim, lim), rng.randint(-lim, lim), rng.choice(actions + [rng.randint(0, 255)])


def main(argv):
    m = Machine(Image(argv[1]))
    m.uc.mem_map(PAGE, 0x1000)
    for x, y, a in rows():
        m.uc.mem_write(PAGE, b"\0" * 0x10)
        m.uc.mem_write(PAGE, struct.pack("<IiiB", 0, x, y, a))
        print(f"{x} {y} {a} -> {signed(m.call(ENTRY, (), ecx=PAGE))}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
