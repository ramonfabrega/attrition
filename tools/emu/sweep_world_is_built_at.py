#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_built_at.py — `WorldData::is_built_at@0046f880` under unicorn.

    uv run tools/emu/sweep_world_is_built_at.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `ecx` = the `WorldData`, then
pointers to two `TCoord` words (x, then y). One row per input,
`<txs> <tys> <x> <y> <mask> <decoy> -> <is_built_at>`:

    txs, tys  the tile extent (`this+0x18`, `this+0x1c`)
    x, y      the queried tile, always inside the map
    mask      that tile's `TData.mask`, an unsigned short (`tdata`, `this+0x138`,
              stride 2)
    decoy     what every OTHER tile holds: 0 = mask 0 (reads not built),
              1 = mask 0xFFFF (reads built), so a read of the wrong tile shows

The `TData` array is laid out in mapped memory; nothing is stood in for and
nothing is called. Edges: every low byte 0..255 over high bytes 0x00, 0xFF,
0x55, 0xAA, all 16 single-bit masks and their complements, and random masks;
corner and edge tiles of three extents.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046F880
WORLD, TDATA, ARGS = 0x10000000, 0x10010000, 0x10008000


def rows():
    rng = random.Random(1630)
    out = []
    masks = [lo | (hi << 8) for hi in (0x00, 0xFF, 0x55, 0xAA) for lo in range(256)]
    masks += [1 << b for b in range(16)] + [0xFFFF ^ (1 << b) for b in range(16)]
    shapes = ((6, 5), (1, 1), (9, 3))
    for xs, ys in shapes:
        for x, y in ((0, 0), (xs - 1, 0), (0, ys - 1), (xs - 1, ys - 1), (xs // 2, ys // 2)):
            for m in masks:
                out.append((xs, ys, x, y, m, (m >> 7) & 1))
    for _ in range(400):
        xs, ys = rng.randint(1, 12), rng.randint(1, 12)
        out.append((xs, ys, rng.randrange(xs), rng.randrange(ys),
                    rng.getrandbits(16), rng.randint(0, 1)))
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
