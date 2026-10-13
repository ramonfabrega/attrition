#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_blocked_at.py — `WorldData::is_blocked_at@00461340` under unicorn.

    uv run tools/emu/sweep_world_is_blocked_at.py <install>/riseofnations.exe

`__thiscall`, three stack arguments (`ret 0xc`): `ecx` = the `WorldData`,
pointers to two `TCoord` words (tile x, then tile y), then `mode`. One row
per input, `<tile_xs> <tile_ys> <tx> <ty> <mask> <mode> <decoy> -> <result>`:

    tile_xs, tile_ys  the tile extent (`this+0x18` is `tile_xs`; `tile_ys`
                      only sizes the array)
    tx, ty            the queried tile, always on the map
    mask              that tile's `TData.mask`, an unsigned short
    mode              the third argument; 0 answers `mask & 0x4000` raw,
                      non-zero answers 1 only for a blocked tile whose
                      surface field (`mask & 0x30`) is not 0x30
    decoy             what every OTHER tile holds: 0 = mask 0, 1 = 0x4000,
                      2 = 0x4030, so a read of the wrong tile shows

The `TData` array (stride 2, `WorldData+0x138`) is laid out in mapped
memory; nothing is stood in for and nothing is called. Edges: every single
mask bit, all bits, and each of the 0x4000 / 0x30 combinations on both modes,
corner and edge tiles of several extents, then seeded random masks.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00461340
WORLD, TDATA, ARGS = 0x10000000, 0x10010000, 0x10008000
DECOYS = (0x0000, 0x4000, 0x4030)


def rows():
    rng = random.Random(1630)
    out = []
    edges = [0, 0xFFFF, 0x4000, 0x4010, 0x4020, 0x4030, 0x0030, 0x0010, 0x0020, 0xBFFF,
             0xBFCF, 0x7FFF, 0x403F, 0x400F, 0xC030, 0x8000, 0x2000, 0x6030, 0x5030]
    edges += [1 << b for b in range(16)] + [0xFFFF ^ (1 << b) for b in range(16)]
    shapes = ((24, 20), (4, 4), (36, 8))
    for txs, tys in shapes:
        for x, y in ((0, 0), (txs - 1, 0), (0, tys - 1), (txs - 1, tys - 1), (txs // 2, tys // 3)):
            for mask in edges:
                for mode in (0, 1):
                    for decoy in range(3):
                        out.append((txs, tys, x, y, mask, mode, decoy))
    for _ in range(600):
        txs, tys = rng.randint(1, 16) * 4, rng.randint(1, 16) * 4
        out.append((txs, tys, rng.randrange(txs), rng.randrange(tys),
                    rng.choice((rng.getrandbits(16), rng.choice(edges))),
                    rng.choice((0, 1, 2, -1, 0x100)), rng.randrange(3)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(WORLD, 0x1000)
    uc.mem_map(TDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    for txs, tys, x, y, mask, mode, decoy in rows():
        uc.mem_write(WORLD + 0x18, struct.pack("<i", txs))
        uc.mem_write(WORLD + 0x138, struct.pack("<I", TDATA))
        uc.mem_write(TDATA, struct.pack("<H", DECOYS[decoy]) * (txs * tys))
        uc.mem_write(TDATA + (y * txs + x) * 2, struct.pack("<H", mask))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4, mode), ecx=WORLD))
        print(f"{txs} {tys} {x} {y} {mask} {mode} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
