#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_get_land.py — `WorldData::get_land@006b4c70` under unicorn.

    uv run tools/emu/sweep_world_get_land.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the `WorldData`, three stack arguments (`TCoord *x`,
`TCoord *y`, `int mode`), `ret 0xc`. Each row is

    <cells_wide> <tx> <ty> <mode> <flags> <land> <mask> -> <eax, signed>

`cells_wide` is `WorldData.xs` (the world is `xs` x 3 cells, so `tile_xs` =
`4 * xs`); `tx`, `ty` the tile coordinates (always on the map: the function
does no bounds test, and an off-map tile reads past the arrays); `mode` the
third argument; `flags` the `WData.flags` (u16), `land` the `WData.land`
(signed char) of the cell holding the tile, and `mask` the `TData.mask`
(u16) of the tile itself. Every OTHER cell and tile of the world is a decoy
(`flags`, `land`, `mask` all bit-inverted) so a read of the wrong cell or
tile changes the answer.

Stood in for: only `WorldData.xs` (+0x0), `.tile_xs` (+0x18), `.wdata`
(+0x134, stride 0x1c, `flags` +0, `land` +2) and `.tdata` (+0x138, stride 2)
are laid; the rest of the `WorldData` and of each `WData` is zero.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006B4C70
THIS, WDATA, TDATA, ARGS = 0x1000_0000, 0x1010_0000, 0x1020_0000, 0x1030_0000
WSTRIDE = 0x1C
CELLS_HIGH = 3
MODES = [0, 1, -1, 2, 3, 100, -2, -3, 0x7FFFFFFF, -0x80000000]
# every flag the function tests, and the neighbours of each
BITS = [0x4, 0x20, 0x800, 0x8, 0x10, 0x40, 0x100, 0x1000, 0x8000]
MASKS = [0x00, 0x01, 0x02, 0x03, 0x10, 0x20, 0x30, 0x22, 0x21, 0x32, 0x33,
         0x12, 0x0E, 0x100, 0x120, 0x2, 0xFFFF, 0xFF22, 0xFFFC]
LANDS = [0, 1, 2, 3, 4, 5, 6, 7, -1, 127, -128, 8, 100]


def inv8(v):
    """The bit-inverse of a signed char, as a signed char."""
    return ~v if -128 <= ~v <= 127 else 0


def lay(m, width, flags, land, mask, tx, ty):
    uc = m.uc
    xs = width
    tile_xs = 4 * width
    ncell = xs * CELLS_HIGH
    ntile = tile_xs * 4 * CELLS_HIGH
    w = bytearray(ncell * WSTRIDE)
    for i in range(ncell):
        struct.pack_into("<Hb", w, i * WSTRIDE, (~flags) & 0xFFFF, inv8(land))
    t = bytearray(ntile * 2)
    for i in range(ntile):
        struct.pack_into("<H", t, i * 2, (~mask) & 0xFFFF)
    ci = (ty >> 2) * xs + (tx >> 2)
    struct.pack_into("<Hb", w, ci * WSTRIDE, flags, land)
    struct.pack_into("<H", t, (tile_xs * ty + tx) * 2, mask)
    uc.mem_write(WDATA, bytes(w))
    uc.mem_write(TDATA, bytes(t))
    hdr = bytearray(0x170)
    struct.pack_into("<i", hdr, 0x0, xs)
    struct.pack_into("<i", hdr, 0x18, tile_xs)
    struct.pack_into("<I", hdr, 0x134, WDATA)
    struct.pack_into("<I", hdr, 0x138, TDATA)
    uc.mem_write(THIS, bytes(hdr))
    uc.mem_write(ARGS, struct.pack("<ii", tx, ty))


def inputs(seed=1619, n=420):
    rng = random.Random(seed)
    out = []
    # every flag bit alone and in the pairs the function's order decides,
    # under the masks that decide each arm, for every mode
    pairs = [(a, b) for a in BITS[:5] for b in BITS[:5]]
    for mode in MODES:
        for flags in [0] + BITS + [a | b for a, b in pairs]:
            for mask in (0x00, 0x02, 0x20, 0x30, 0x22, 0x10):
                out.append((rng.choice([1, 3, 5]), mode, flags, rng.choice(LANDS), mask))
    for _ in range(n):
        out.append((rng.choice([1, 2, 3, 5, 7]), rng.choice(MODES + [1, 1, 1, 1]),
                    rng.choice([rng.getrandbits(16), rng.getrandbits(16) & 0x0A3C,
                                rng.choice(BITS) | rng.choice(BITS)]),
                    rng.choice(LANDS + [rng.randint(-128, 127)]),
                    rng.choice(MASKS + [rng.getrandbits(16)])))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(THIS, 0x1000)
    uc.mem_map(WDATA, 0x10000)
    uc.mem_map(TDATA, 0x10000)
    uc.mem_map(ARGS, 0x1000)
    rng = random.Random(16190)
    for width, mode, flags, land, mask in inputs():
        tx = rng.randrange(4 * width)
        ty = rng.randrange(4 * CELLS_HIGH)
        lay(m, width, flags, land, mask, tx, ty)
        r = signed(m.call(ENTRY, (ARGS, ARGS + 4, mode), ecx=THIS))
        print(f"{width} {tx} {ty} {mode} {flags} {land} {mask} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
