#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_get_region2.py — `WorldData::get_region2@006b3960` under unicorn.

    uv run tools/emu/sweep_world_get_region2.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world, two stack arguments (`ret 8`) pointers to the `WCoord` words (x, then y); answers the cell's `region2` (`WData+0x6`) sign-extended.
The `WData` array (stride 0x1c, `WorldData+0x134`) is laid out in mapped memory.
Rows: `<xs> <ys> <x> <y> <value> <decoy> -> <result>`; `decoy` is what every OTHER cell holds (-1 or a fixed other value), so a read of the wrong cell shows.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006b3960
WORLD, WDATA, ARGS = 0x10000000, 0x10010000, 0x10008000
STRIDE = 0x1C
OFF = 6
FMT = "<h"
EDGES = [-1, 0, 1, 2, 5, 63, 64, 100, 127, 199]


def rows():
    rng = random.Random(1721)
    out = []
    for xs, ys in ((6, 5), (1, 1), (9, 3)):
        for x, y in ((0, 0), (xs - 1, 0), (0, ys - 1), (xs - 1, ys - 1), (xs // 2, ys // 2)):
            for v in EDGES:
                out.append((xs, ys, x, y, v, 0))
                out.append((xs, ys, x, y, v, 1))
    for _ in range(300):
        xs, ys = rng.randint(1, 12), rng.randint(1, 12)
        out.append((xs, ys, rng.randrange(xs), rng.randrange(ys), rng.choice(EDGES), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(WORLD, 0x1000)
    uc.mem_map(WDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    for xs, ys, x, y, v, decoy in rows():
        uc.mem_write(WORLD + 0, struct.pack("<ii", xs, ys))
        uc.mem_write(WORLD + 0x134, struct.pack("<I", WDATA))
        for i in range(xs * ys):
            uc.mem_write(WDATA + i * STRIDE, b"\0" * STRIDE)
            uc.mem_write(WDATA + i * STRIDE + OFF, struct.pack(FMT, -1 if decoy else 3))
        uc.mem_write(WDATA + (y * xs + x) * STRIDE + OFF, struct.pack(FMT, v))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=WORLD))
        print(f"{xs} {ys} {x} {y} {v} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
