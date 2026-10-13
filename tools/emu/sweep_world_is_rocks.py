#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_rocks.py — `WorldData::is_rocks@006b4380` under unicorn.

    uv run tools/emu/sweep_world_is_rocks.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `ecx` = the `WorldData`, then
pointers to two `WCoord` words (x, then y). One row per input,
`<xs> <ys> <x> <y> <flags> <decoy> -> <is_rocks>`; the result is
`flags & 8`, so 0 or 8:

    xs, ys   the world's extent (`this+0`, `this+4`), cells
    x, y     the queried cell, always inside the map
    flags    that cell's `WData.flags`, an unsigned short (`wdata+0`)
    decoy    what every OTHER cell holds: 0 = flags 0, 1 = flags 0xFFFF
             (rock bit set), so a read of the wrong cell shows

The `WData` array (stride 0x1c, `WorldData+0x134`) is laid out in mapped
memory; nothing is stood in. Edges: every flag bit singly, all bits, all
but 0x8, and the low byte swept; corner and edge cells of three extents.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006B4380
WORLD, WDATA, ARGS = 0x10000000, 0x10010000, 0x10008000
STRIDE = 0x1C


def rows():
    rng = random.Random(1630)
    out = []
    flag_edges = [0, 0xFFFF, 0xFFF7, 0x8, 0x7, 0xF, 0x10, 0x80, 0x100, 0x800, 0x8000, 0x7FFF]
    flag_edges += [1 << b for b in range(16)]
    for xs, ys in ((6, 5), (1, 1), (9, 3)):
        for x, y in ((0, 0), (xs - 1, 0), (0, ys - 1), (xs - 1, ys - 1), (xs // 2, ys // 2)):
            for flags in flag_edges:
                out.append((xs, ys, x, y, flags, 0))
                out.append((xs, ys, x, y, flags, 1))
            for flags in range(0x100):
                out.append((xs, ys, x, y, flags, flags & 1))
    for _ in range(400):
        xs, ys = rng.randint(1, 12), rng.randint(1, 12)
        out.append((xs, ys, rng.randrange(xs), rng.randrange(ys),
                    rng.choice((rng.getrandbits(16), rng.choice(flag_edges))), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(WORLD, 0x1000)
    uc.mem_map(WDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    for xs, ys, x, y, flags, decoy in rows():
        uc.mem_write(WORLD + 0, struct.pack("<ii", xs, ys))
        uc.mem_write(WORLD + 0x134, struct.pack("<I", WDATA))
        for i in range(xs * ys):
            uc.mem_write(WDATA + i * STRIDE, struct.pack("<H", 0xFFFF if decoy else 0))
        uc.mem_write(WDATA + (y * xs + x) * STRIDE, struct.pack("<H", flags))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=WORLD))
        print(f"{xs} {ys} {x} {y} {flags} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
