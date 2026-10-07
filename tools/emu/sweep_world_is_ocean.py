#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_ocean.py — `WorldData::is_ocean@006b4830` under unicorn.

    uv run tools/emu/sweep_world_is_ocean.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `ecx` = the `WorldData`, then
pointers to two `WCoord` words (x, then y). One row per input,
`<xs> <ys> <x> <y> <flags> <land> <decoy> -> <is_ocean>`:

    xs, ys   the world's extent (`this+0`, `this+4`), cells
    x, y     the queried cell, always inside the map (the words the two
             argument pointers point at)
    flags    that cell's `WData.flags`, an unsigned short (`wdata+0`)
    land     that cell's `WData.land`, a signed char (`wdata+2`)
    decoy    what every OTHER cell holds: 0 = flags 0 / land 1 (reads as
             ocean), 1 = flags 0x100 / land 3 (reads as not), so a read of
             the wrong cell shows

The `WData` array (stride 0x1c, `WorldData+0x134`) is laid out in mapped
memory; nothing is stood in for and nothing is called. Edges: every `land`
from -128 to 127 on the 0x100 bit clear and set, each single flag bit, all
bits, and every bit but 0x100; corner and edge cells of three extents.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006B4830
WORLD, WDATA, ARGS = 0x10000000, 0x10010000, 0x10008000
STRIDE = 0x1C


def rows():
    rng = random.Random(1619)
    out = []
    flag_edges = [0, 0x100, 0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x200, 0x400,
                  0x800, 0x1000, 0x2000, 0x4000, 0x8000, 0xFFFF, 0xFEFF, 0x1FF, 0x00FF]
    shapes = ((6, 5), (1, 1), (9, 3))
    for xs, ys in shapes:
        for x, y in ((0, 0), (xs - 1, 0), (0, ys - 1), (xs - 1, ys - 1), (xs // 2, ys // 2)):
            for land in range(-128, 128):
                for flags in (0, 0x100):
                    out.append((xs, ys, x, y, flags, land, (land + flags) & 1))
            for flags in flag_edges:
                for land in (0, 1, 2, 3, 4, 5, 6, 7, -1):
                    out.append((xs, ys, x, y, flags, land, 0))
                    out.append((xs, ys, x, y, flags, land, 1))
    for _ in range(400):
        xs, ys = rng.randint(1, 12), rng.randint(1, 12)
        out.append((xs, ys, rng.randrange(xs), rng.randrange(ys),
                    rng.choice((rng.getrandbits(16), rng.choice(flag_edges))),
                    rng.choice((rng.randint(0, 7), rng.randint(-128, 127))), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(WORLD, 0x1000)
    uc.mem_map(WDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    for xs, ys, x, y, flags, land, decoy in rows():
        uc.mem_write(WORLD + 0, struct.pack("<ii", xs, ys))
        uc.mem_write(WORLD + 0x134, struct.pack("<I", WDATA))
        for i in range(xs * ys):
            f, l = (0x100, 3) if decoy else (0, 1)
            uc.mem_write(WDATA + i * STRIDE, struct.pack("<Hb", f, l))
        uc.mem_write(WDATA + (y * xs + x) * STRIDE, struct.pack("<Hb", flags, land))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=WORLD))
        print(f"{xs} {ys} {x} {y} {flags} {land} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
