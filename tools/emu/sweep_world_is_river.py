#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_river.py — `WorldData::is_river@0046d390` under unicorn.

    uv run tools/emu/sweep_world_is_river.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `ecx` = the WorldData, then
pointers to two `TCoord` words (x, then y). Answers `tdata[tile_xs*y + x].mask
& 0x800`. One row per input, `<cw> <ch> <x> <y> <mask> <decoy> -> <result>`:

    cw, ch   the map's size in cells; `tile_xs = 4*cw` (`this+0x18`)
    x, y     the queried tile, inside the tile grid
    mask     that tile's u16 mask
    decoy    every OTHER tile: 0 = all 0, 1 = all 0xFFFF, 2 = 0xF7FF
             (everything but the river bit), 3 = 0x0800 (only the river bit)

`tdata` (`this+0x138`, u16 per tile) is laid out in mapped memory; nothing is
stood in for. Edges: each single mask bit, all bits, all but 0x800, the
corner and edge tiles of several extents, then seeded random rows.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046D390
THIS, TDATA, ARGS = 0x10000000, 0x10010000, 0x10008000
DECOY = (0, 0xFFFF, 0xF7FF, 0x0800)


def rows():
    rng = random.Random(1630)
    out = []
    edges = [0, 0xFFFF, 0xF7FF, 0x0800] + [1 << i for i in range(16)] + [0xFFFF ^ (1 << i) for i in range(16)]
    for cw, ch in ((3, 2), (1, 1), (5, 4)):
        xs, ys = 4 * cw, 4 * ch
        for x, y in ((0, 0), (xs - 1, 0), (0, ys - 1), (xs - 1, ys - 1), (xs // 2, ys // 2), (1, ys - 2)):
            for m in edges:
                for d in range(4):
                    out.append((cw, ch, x, y, m, d))
    for _ in range(400):
        cw, ch = rng.randint(1, 8), rng.randint(1, 8)
        out.append((cw, ch, rng.randrange(4 * cw), rng.randrange(4 * ch),
                    rng.getrandbits(16), rng.randint(0, 3)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(THIS, 0x1000)
    uc.mem_map(TDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    for cw, ch, x, y, mask, decoy in rows():
        xs, ys = 4 * cw, 4 * ch
        uc.mem_write(THIS + 0x18, struct.pack("<ii", xs, ys))
        uc.mem_write(THIS + 0x138, struct.pack("<I", TDATA))
        uc.mem_write(TDATA, struct.pack("<H", DECOY[decoy]) * (xs * ys))
        uc.mem_write(TDATA + 2 * (y * xs + x), struct.pack("<H", mask))
        uc.mem_write(ARGS, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=THIS))
        print(f"{cw} {ch} {x} {y} {mask} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
