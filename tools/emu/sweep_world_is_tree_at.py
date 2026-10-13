#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_tree_at.py — `WorldData::is_tree_at@0046f930` under unicorn.

    uv run tools/emu/sweep_world_is_tree_at.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `ecx` = the `WorldData`, then
pointers to two `TCoord` words (x, then y; each an int). One row per input,
`<tile_xs> <tile_ys> <x> <y> <mask> <decoy> -> <is_tree_at>`:

    tile_xs, tile_ys  the tile extent (`this+0x18`, `this+0x1c`)
    x, y              the queried tile, always inside the map
    mask              that tile's `TData.mask`, an unsigned short
    decoy             what every OTHER tile holds: 0 = 0x0000 (reads false),
                      1 = 0xFFFF (reads true), so a read of the wrong tile
                      shows

The `TData` array (stride 2, `WorldData+0x138`) is laid out in mapped memory;
nothing is stood in for and nothing is called. Edges: every mask whose bits
0x30 are each of the four values crossed with the other bits, each single
bit, and random 16-bit masks; corner and edge tiles of several extents.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046F930
WORLD, TDATA, ARGS = 0x10000000, 0x10010000, 0x10008000


def rows():
    rng = random.Random(1630)
    out = []
    edges = [0, 0x10, 0x20, 0x30, 0xFFFF, 0xFFCF, 0xFFEF, 0xFFDF, 0x3, 0x2, 0x1,
             0x32, 0x33, 0x31, 0x4000, 0x8000, 0x0F, 0xC0] + [1 << b for b in range(16)]
    shapes = ((24, 20), (4, 4), (36, 12), (1, 1))
    for xs, ys in shapes:
        for x, y in ((0, 0), (xs - 1, 0), (0, ys - 1), (xs - 1, ys - 1), (xs // 2, ys // 2)):
            for mask in edges:
                out.append((xs, ys, x, y, mask, 0))
                out.append((xs, ys, x, y, mask, 1))
            for hi in range(0, 0x10000, 0x1000):
                for sfc in (0, 0x10, 0x20, 0x30):
                    out.append((xs, ys, x, y, hi | sfc | 0x3, (hi >> 12) & 1))
    for _ in range(400):
        xs, ys = rng.randint(1, 40), rng.randint(1, 40)
        out.append((xs, ys, rng.randrange(xs), rng.randrange(ys),
                    rng.choice((rng.getrandbits(16), rng.choice(edges))), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(WORLD, 0x1000)
    uc.mem_map(TDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    for xs, ys, x, y, mask, decoy in rows():
        uc.mem_write(WORLD + 0x18, struct.pack("<ii", xs, ys))
        uc.mem_write(WORLD + 0x138, struct.pack("<I", TDATA))
        fill = 0xFFFF if decoy else 0
        uc.mem_write(TDATA, struct.pack("<H", fill) * (xs * ys))
        uc.mem_write(TDATA + (y * xs + x) * 2, struct.pack("<H", mask))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=WORLD))
        print(f"{xs} {ys} {x} {y} {mask} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
