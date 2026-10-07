#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_world_has_blocked_neighbors.py — `WorldData::has_blocked_neighbors@006b2990` under unicorn.

    uv run tools/emu/sweep_world_has_blocked_neighbors.py <install>/riseofnations.exe

`__thiscall`: `ecx` = the WorldData, `[esp+4]` = `TCoord*` x, `[esp+8]` = `TCoord*`
y, `ret 8`. One row per input, `<cw> <ch> <tx> <ty> <ring> <mode> -> <0|1>`:

    cw, ch   the map's size in cells; the tile grid is `tile_xs = 4*cw` by
             `tile_ys = 4*ch` (`this+0x18`, `this+0x1c`), the stride the
             function multiplies by
    tx, ty   the two TCoords' values (may be off the map, or negative)
    ring     bit i (0..7) set: the neighbour at (move_x[i+1], move_y[i+1])
             carries mask bit 0x4000
    mode     the rest of the grid: 0 every other tile 0; 1 every other tile
             0xBFFF (all bits but 0x4000); 2 as 1 but the centre tile and
             every tile outside the 3 x 3 are 0xFFFF (0x4000 set - a
             function that read the wrong tile would say 1)

Laid out in mapped pages: the WorldData (only `+0x18`, `+0x1c`, `+0x138` are
read), the `tdata` array of `u16` masks, and the two TCoord words. Nothing is
stood in for: `move_x` / `move_y` are statically initialised in the image
(`.data` 0xADCAF0 / 0xADC400, the PE is mapped whole) and the real tables are
read. A neighbour of a set ring bit that is off the map is simply not written
(the original must not read it).
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x006B2990
THIS, TDATA, COORDS = 0x10000000, 0x10010000, 0x10008000
# The compass ring, move_x[1..=8] / move_y[1..=8] (the function reads the
# image's own; this copy only lays out which neighbour a ring bit means, and
# a disagreement shows as a row the original answers differently).
MOVE = [(-1, -1), (0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0)]


def grid(cw, ch, tx, ty, ring, mode):
    xs, ys = 4 * cw, 4 * ch
    base = (0, 0xBFFF, 0xBFFF)[mode]
    far = (0, 0xBFFF, 0xFFFF)[mode]
    g = [far] * (xs * ys)
    for dy in (-1, 0, 1):
        for dx in (-1, 0, 1):
            x, y = tx + dx, ty + dy
            if 0 <= x < xs and 0 <= y < ys:
                g[y * xs + x] = base
    if mode == 2 and 0 <= tx < xs and 0 <= ty < ys:
        g[ty * xs + tx] = 0xFFFF
    for i, (dx, dy) in enumerate(MOVE):
        x, y = tx + dx, ty + dy
        if ring >> i & 1 and 0 <= x < xs and 0 <= y < ys:
            g[y * xs + x] |= 0x4000
    return g


def rows():
    rng = random.Random(1619)
    out = []
    for cw, ch in ((8, 8), (3, 5), (1, 1), (13, 2)):
        xs, ys = 4 * cw, 4 * ch
        # corners, edges, interior, one off in every direction
        for tx in (-2, -1, 0, 1, xs // 2, xs - 2, xs - 1, xs, xs + 1):
            for ty in (-2, -1, 0, 1, ys // 2, ys - 2, ys - 1, ys, ys + 1):
                # no bit, all bits, every single neighbour, two random rings
                ring = [0, 255] + [1 << i for i in range(8)] + [rng.randint(1, 254) for _ in range(2)]
                for r in ring:
                    out.append((cw, ch, tx, ty, r, rng.randint(0, 2)))
    for _ in range(300):
        cw, ch = rng.randint(1, 16), rng.randint(1, 16)
        tx = rng.choice((rng.randint(-3, 4 * cw + 2), rng.randint(0, 4 * cw - 1)))
        ty = rng.choice((rng.randint(-3, 4 * ch + 2), rng.randint(0, 4 * ch - 1)))
        out.append((cw, ch, tx, ty, rng.randint(0, 255), rng.randint(0, 2)))
    # far off the map, both signs (|v| < 2^29: the port's i32 adds do not wrap)
    for v in (1 << 28, -(1 << 28), (1 << 29) - 1, -((1 << 29) - 1)):
        out.append((8, 8, v, 5, 255, 1))
        out.append((8, 8, 5, v, 255, 1))
        out.append((8, 8, v, v, 255, 2))
    return out


def main(argv):
    m = Machine(Image(argv[1]))
    for a, n in ((THIS, 0x1000), (COORDS, 0x1000), (TDATA, 0x10000)):
        m.uc.mem_map(a, n)
    for r in rows():
        cw, ch, tx, ty, ring, mode = r
        g = grid(cw, ch, tx, ty, ring, mode)
        m.uc.mem_write(TDATA, struct.pack(f"<{len(g)}H", *g))
        m.uc.mem_write(THIS + 0x18, struct.pack("<ii", 4 * cw, 4 * ch))
        m.uc.mem_write(THIS + 0x138, struct.pack("<I", TDATA))
        m.uc.mem_write(COORDS, struct.pack("<ii", tx, ty))
        print(" ".join(map(str, r)), "->", signed(m.call(ENTRY, (COORDS, COORDS + 4), ecx=THIS)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
