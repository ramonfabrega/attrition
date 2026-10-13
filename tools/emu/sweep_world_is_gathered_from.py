#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_gathered_from.py — `WorldData::is_gathered_from@00472ac0` under unicorn.

    uv run tools/emu/sweep_world_is_gathered_from.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `ecx` = the `WorldData`, then
pointers to two `TCoord` words (x, then y). The answer is
`tdata[tile_xs * y + x].mask & 0x1000` (0 or 0x1000). One row per input,
`<tile_xs> <tile_ys> <x> <y> <mask> <decoy> -> <result>`:

    tile_xs, tile_ys  the tile grid's extent (`this+0x18`, `this+0x1c`)
    x, y              the queried tile, always inside the grid
    mask              that tile's `TData.mask`, an unsigned short
    decoy             what every OTHER tile holds: 0 = mask 0, 1 = 0x1000,
                      2 = 0xEFFF, so a read of the wrong tile shows

The `TData` array (stride 2, pointer at `WorldData+0x138`) is laid out in
mapped memory; nothing is stood in for and nothing is called. Edges: every
single mask bit, all bits, all but 0x1000, corner and edge tiles of
several extents, plus seeded random rows.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00472AC0
WORLD, TDATA, ARGS = 0x10000000, 0x10010000, 0x10008000
DECOY = (0, 0x1000, 0xEFFF)


def rows():
    rng = random.Random(1630)
    out = []
    edges = [0, 0xFFFF, 0xEFFF, 0x1000, 0x0FFF, 0x1FFF, 0x2000, 0x8000, 0x4000, 0x0001]
    edges += [1 << b for b in range(16)]
    for xs, ys in ((24, 20), (4, 4), (36, 12), (8, 4)):
        for x, y in ((0, 0), (xs - 1, 0), (0, ys - 1), (xs - 1, ys - 1), (xs // 2, ys // 2)):
            for mask in edges:
                for decoy in range(3):
                    out.append((xs, ys, x, y, mask, decoy))
    for _ in range(300):
        xs, ys = 4 * rng.randint(1, 10), 4 * rng.randint(1, 10)
        out.append((xs, ys, rng.randrange(xs), rng.randrange(ys),
                    rng.choice((rng.getrandbits(16), rng.choice(edges))), rng.randint(0, 2)))
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
        uc.mem_write(TDATA, struct.pack("<H", DECOY[decoy]) * (xs * ys))
        uc.mem_write(TDATA + (y * xs + x) * 2, struct.pack("<H", mask))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=WORLD))
        print(f"{xs} {ys} {x} {y} {mask} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
