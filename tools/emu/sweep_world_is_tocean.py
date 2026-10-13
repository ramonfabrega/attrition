#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_is_tocean.py — `WorldData::is_tocean@0046fb10` under unicorn.

    uv run tools/emu/sweep_world_is_tocean.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the world, two stack arguments (`ret 8`) pointers to the `TCoord` words (x, then y); answers `(tdata[tile_xs * y + x].mask & 0x30) == 0x20`, the tile's surface field is ocean.
The `TData` array (stride 2, `WorldData+0x138`) is laid out in mapped memory.
Rows: `<tile_xs> <x> <y> <mask> <decoy> -> <is_tocean>`; `decoy` is what every OTHER tile holds (0 = a mask that reads ocean, 1 = one that does not), so a read of the wrong tile shows.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x0046fb10
OBJ = 0x10000000
ARG = 0x20000000


def rows():
    rng = random.Random(1708)
    out = []
    masks = [0, 0x20, 0x30, 0x10, 0x0F, 0xFF, 0xFFFF, 0xFFCF, 0xFFDF, 0x2F, 0x120, 0x220, 0x1020, 0x8020, 0x03, 0x40, 0x80, 0x100, 0x200]
    for xs in (4, 8, 12):
        for x, y in ((0, 0), (xs - 1, 0), (0, 3), (xs - 1, 3), (xs // 2, 2)):
            for m in masks:
                for d in (0, 1):
                    out.append((xs, x, y, m, d))
            for b in range(16):
                out.append((xs, x, y, 1 << b, 0))
                out.append((xs, x, y, 0xFFFF ^ (1 << b), 1))
    for _ in range(300):
        xs = 4 * rng.randint(1, 3)
        out.append((xs, rng.randrange(xs), rng.randrange(4), rng.choice((rng.getrandbits(16), rng.choice(masks))), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARG, 0x1000)
    TD = OBJ + 0x10000
    for xs, x, y, mask, decoy in rows():
        uc.mem_write(OBJ + 0x18, struct.pack("<i", xs))
        uc.mem_write(OBJ + 0x138, struct.pack("<I", TD))
        d = 0x20 if decoy == 0 else 0x10
        uc.mem_write(TD, struct.pack("<H", d) * (xs * 4))
        uc.mem_write(TD + (xs * y + x) * 2, struct.pack("<H", mask))
        uc.mem_write(ARG, struct.pack("<ii", x, y))
        r = signed(m.call(ENTRY, (ARG, ARG + 4), ecx=OBJ))
        print(f"{xs} {x} {y} {mask} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
