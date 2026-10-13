#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_cliff_at.py — `WorldData::is_cliff_at@0046f8c0` under unicorn.

    uv run tools/emu/sweep_world_is_cliff_at.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): pointers to two `TCoord` words
(tile x, then tile y). The function ignores `ecx` and reads the singleton
`GameAccessConst::worldc` (the pointer cell at 0x00c061d0): `tile_xs`
(`+0x18`, tiles per row) and `tdata` (`+0x138`, one u16 mask a tile), and
answers `(byte(tdata[y * tile_xs + x]) & 3) == 1`. The pointer cell is
mapped and pointed at a fake world. One row per input,
`<txs> <tys> <x> <y> <mask> <decoy> -> <is_cliff_at>`:

    txs, tys  the tile extent (`tile_xs` and the rows laid out)
    x, y      the queried tile, inside the map
    mask      that tile's u16 mask
    decoy     what every OTHER tile holds: 0 = 0 (not a cliff), 1 = 1 (a
              cliff), so a read of the wrong tile shows

Edges: every mask value 0..=0x3ff and each single bit, all bits, plus
random 16-bit masks (the high byte is never read); corner and edge tiles.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046F8C0
WORLDC = 0x00C061D0
WORLD, TDATA, ARGS = 0x10000000, 0x10010000, 0x10008000


def rows():
    rng = random.Random(1630)
    out = []
    shapes = ((7, 5), (1, 1), (9, 3), (4, 8))
    for txs, tys in shapes:
        for x, y in ((0, 0), (txs - 1, 0), (0, tys - 1), (txs - 1, tys - 1), (txs // 2, tys // 2)):
            for mask in range(0x400):
                out.append((txs, tys, x, y, mask, mask & 1))
            for b in range(16):
                out.append((txs, tys, x, y, 1 << b, 0))
                out.append((txs, tys, x, y, 0xFFFF ^ (1 << b), 1))
            out.append((txs, tys, x, y, 0xFFFF, 0))
            out.append((txs, tys, x, y, 0xFF01, 0))
    for _ in range(400):
        txs, tys = rng.randint(1, 20), rng.randint(1, 20)
        out.append((txs, tys, rng.randrange(txs), rng.randrange(tys),
                    rng.getrandbits(16), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(WORLD, 0x1000)
    uc.mem_map(TDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    try:
        uc.mem_map(WORLDC & ~0xFFF, 0x1000)
    except Exception:
        pass  # already part of the image's data section
    uc.mem_write(WORLDC, struct.pack("<I", WORLD))
    for txs, tys, x, y, mask, decoy in rows():
        uc.mem_write(WORLD + 0x18, struct.pack("<i", txs))
        uc.mem_write(WORLD + 0x138, struct.pack("<I", TDATA))
        uc.mem_write(TDATA, struct.pack("<H", decoy) * (txs * tys))
        uc.mem_write(TDATA + (y * txs + x) * 2, struct.pack("<H", mask))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=0))
        print(f"{txs} {tys} {x} {y} {mask} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
