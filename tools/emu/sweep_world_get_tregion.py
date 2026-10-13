#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_get_tregion.py — `WorldData::get_tregion@006b52e0` under unicorn.

    uv run tools/emu/sweep_world_get_tregion.py <install>/riseofnations.exe

`__thiscall`, two stack arguments (`ret 8`): `ecx` = the `WorldData`, then
pointers to two `TCoord` words (x, then y). One row per input,
`<xs> <ys> <tx> <ty> <flags> <region> <region2> <mask> <decoy> -> <result>`:

    xs, ys    the world's extent in cells (`this+0`, `this+4`); `tile_xs`
              (`this+0x18`) is 4*xs
    tx, ty    the queried tile, inside the map
    flags     the covering cell's `WData.flags` (`wdata+0`), unsigned short
    region    `WData.region` (`wdata+4`), signed short (-1 = none; other negatives not driven, the port's ids are u16)
    region2   `WData.region2` (`wdata+6`), signed short
    mask      the tile's `TData.mask` (`tdata+0`), unsigned short
    decoy     0 = every other cell flags 0x100 / region 90 / region2 91 and
              every other tile mask 0x20; 1 = flags 0 / region 92 / region2
              93 and mask 0x00, so a read of the wrong cell or tile shows

The result is the sign-extended `short` the function returns as an int. The
`WData` array (stride 0x1c, `+0x134`) and `TData` array (stride 2, `+0x138`)
are laid out in mapped memory; nothing is called or stood in for.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006B52E0
WORLD, WDATA, TDATA, ARGS = 0x10000000, 0x10010000, 0x10100000, 0x10008000
STRIDE = 0x1C


def rows():
    rng = random.Random(1630)
    out = []
    flag_edges = [0, 0x100, 0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x200, 0x400,
                  0x800, 0x1000, 0x2000, 0x4000, 0x8000, 0xFFFF, 0xFEFF, 0x1FF, 0x00FF]
    masks = [0, 0x10, 0x20, 0x30, 0x0F, 0x2F, 0x20 | 0x100, 0x30 | 0x8000, 0xFFFF, 0xFFCF,
             0xFFDF, 0xFFEF, 0x40, 0x80, 0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000,
             0x4000, 0x8000]
    regs = (-1, 0, 1, 2, 7, 255, 256, 1000, 32767)
    for xs, ys in ((3, 2), (1, 1), (5, 4)):
        tw, th = xs * 4, ys * 4
        for tx, ty in ((0, 0), (tw - 1, 0), (0, th - 1), (tw - 1, th - 1),
                       (tw // 2, th // 2), (1, 2), (2, 1), (3, 3), (4 % tw, 5 % th)):
            for flags in (0, 0x100, 0x80, 0x1FF, 0xFEFF, 0xFFFF):
                for mask in masks:
                    for decoy in (0, 1):
                        r1, r2 = rng.choice(regs), rng.choice(regs)
                        out.append((xs, ys, tx, ty, flags, r1, r2, mask, decoy))
            for flags in flag_edges:
                for r1, r2 in ((5, 6), (-1, 3), (4, -1), (-1, -1), (32767, 0)):
                    for mask in (0x20, 0x30, 0x00):
                        out.append((xs, ys, tx, ty, flags, r1, r2, mask, 1))
    for _ in range(400):
        xs, ys = rng.randint(1, 8), rng.randint(1, 8)
        out.append((xs, ys, rng.randrange(xs * 4), rng.randrange(ys * 4),
                    rng.choice((rng.getrandbits(16), rng.choice(flag_edges))),
                    rng.choice(regs + (rng.randint(-1, 32767),)),
                    rng.choice(regs + (rng.randint(-1, 32767),)),
                    rng.choice((rng.getrandbits(16), rng.choice(masks))), rng.randint(0, 1)))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(WORLD, 0x1000)
    uc.mem_map(WDATA, 0x10000)
    uc.mem_map(TDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    for xs, ys, tx, ty, flags, r1, r2, mask, decoy in rows():
        uc.mem_write(WORLD + 0, struct.pack("<ii", xs, ys))
        uc.mem_write(WORLD + 0x18, struct.pack("<i", xs * 4))
        uc.mem_write(WORLD + 0x134, struct.pack("<II", WDATA, TDATA))
        for i in range(xs * ys):
            rec = (0, 92, 93) if decoy else (0x100, 90, 91)
            uc.mem_write(WDATA + i * STRIDE, struct.pack("<HbBhh", rec[0], 0, 0, rec[1], rec[2]))
        for i in range(xs * ys * 16):
            uc.mem_write(TDATA + i * 2, struct.pack("<H", 0 if decoy else 0x20))
        cell = (ty >> 2) * xs + (tx >> 2)
        uc.mem_write(WDATA + cell * STRIDE, struct.pack("<HbBhh", flags, 0, 0, r1, r2))
        uc.mem_write(TDATA + (ty * xs * 4 + tx) * 2, struct.pack("<H", mask))
        uc.mem_write(ARGS, struct.pack("<ii", tx, ty))
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4), ecx=WORLD))
        print(f"{xs} {ys} {tx} {ty} {flags} {r1} {r2} {mask} {decoy} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
