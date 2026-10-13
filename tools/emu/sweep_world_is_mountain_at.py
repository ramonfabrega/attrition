#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_mountain_at.py — `WorldData::is_mountain_at@0046f900` under unicorn.

    uv run tools/emu/sweep_world_is_mountain_at.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `ecx` = the `WorldData`, then
pointers to two `TCoord` words (x, then y). One row per input,
`<xs> <ys> <x> <y> <mask> <decoy> -> <is_mountain_at>`:

    xs, ys   the world's extent in cells; the tile grid is four times that
             (`tile_xs` at `this+0x18` is set to 4*xs)
    x, y     the queried tile, always on the map
    mask     that tile's `TData.mask`, an unsigned short
    decoy    what every OTHER tile holds: 0 = mask 0, 1 = mask 2 (reads as a
             mountain), so a read of the wrong tile shows

The `TData` array (stride 2, pointer at `WorldData+0x138`) is laid out in
mapped memory; nothing is stood in for and nothing is called. Edges: every
mask from 0 to 0x3ff, each single bit, all bits, and corner and edge tiles
of three extents.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046F900
WORLD, TDATA, ARGS = 0x10000000, 0x10010000, 0x10008000


def rows():
    rng = random.Random(1630)
    out = []
    edges = [1 << b for b in range(16)] + [0xFFFF, 0xFFFC, 0xFFFE, 0xFFFD, 0x7FFE, 0x8002, 0x4002]
    for xs, ys in ((6, 5), (1, 1), (9, 3)):
        txs, tys = xs * 4, ys * 4
        for x, y in ((0, 0), (txs - 1, 0), (0, tys - 1), (txs - 1, tys - 1), (txs // 2, tys // 2)):
            for mask in range(0x400):
                out.append((xs, ys, x, y, mask, mask & 1))
            for mask in edges:
                out.append((xs, ys, x, y, mask, 0))
                out.append((xs, ys, x, y, mask, 1))
    for _ in range(400):
        xs, ys = rng.randint(1, 12), rng.randint(1, 12)
        out.append((xs, ys, rng.randrange(xs * 4), rng.randrange(ys * 4),
                    rng.choice((rng.getrandbits(16), rng.choice(edges))), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(WORLD, 0x1000)
    uc.mem_map(TDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    for xs, ys, x, y, mask, decoy in rows():
        txs, tys = xs * 4, ys * 4
        uc.mem_write(WORLD + 0x18, struct.pack("<ii", txs, tys))
        uc.mem_write(WORLD + 0x138, struct.pack("<I", TDATA))
        uc.mem_write(TDATA, struct.pack("<H", 2 if decoy else 0) * (txs * tys))
        uc.mem_write(TDATA + (y * txs + x) * 2, struct.pack("<H", mask))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=WORLD))
        print(f"{xs} {ys} {x} {y} {mask} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
