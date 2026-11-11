#!/usr/bin/env python3
"""patches.py — the transition patch on every cell, on a packet (item 4132).

    uv run --locked --project tools/emu python tools/render/patches.py <frame-snapshot.bin>

`docs/RENDERER.md` §3.6, "The transition patch". No function is called:
this reads what `TerrainOut::create_transition_renderables` left in the
process, as `tools/render/land_buckets.py` reads the land lists.

- `MiscAccess::terrain` (0x00C06218) is the `TerrainOut`. Its
  `Transitions` sits at `+0x4974` in the shipped build (the call in
  `create_transition_renderables` passes `this + 0x4974`; the type
  record's `+0x4934` is 0x40 short, as its `terrainlist` is against the
  `+0x47d8` `land_buckets.py` reads). `alphas` (`PtrArray<AlphaInfo>`):
  length `+4`, items `+0x10`. An `AlphaInfo` (0x28): `alpha` (`Texture *`,
  its `filename` a `String` at `+0x34`), `bitcode[8]` (`+4`),
  `selected_bitcode` (`+0x24`, the last cell's, since `find_map` writes it
  on the shared record), `bucket` (`+0x25`).
- `+0x6ab0` is the cell-indexed `CoordInfo *` table the function writes
  through; a `CoordInfo`'s `patches` (`+0x2c`) is the `PatchInfo` (0x9c):
  `trans_texcoords0` (`+0`), `trans_texcoords1` (`+0x1c`), each a
  `TexArray` (length `+4`, data `+0x10`, a `(u, v)` float pair a vertex),
  `bucket` (`+0x54`), `landindex` (`+0x55`), `tex` (`+0x58`).
- `TerrainData::texture_fractal` (0x00C8BF20, the map's `S_GDATA32`), a
  `Fractal`: `frac` rows (length `+4`, items `+0x10`, each an
  `Array<uchar>` of 0x1c with its data at `+0x10`), `flags` `+0x20`,
  `parts` `+0x24`, `x_inc` `+0x68`, `y_inc` `+0x70`. `frac` is
  `Fractal::get_height`'s answer at the cell, in its own order of doubles.

Rows:

    terrain <width> <height> fractal <xs> <ys> flags <f> parts0 <p> x_inc <d> y_inc <d>
    alpha <i> bucket <b> sel <s> file <name> codes <bitcode[0..8] hex>
    patch <xw> <yw> land <l> sub <s> alpha <bucket> landindex <li> tex <name> frac <h> nbr <8 subs> tc0 <n> tc1 <n> uv <u,v ...>

`nbr` is the eight neighbours' `land_sub` (WData +3), `-` off the map or on
another `land`, in the order (−1,−1) (0,−1) (1,−1) (−1,0) (1,0) (−1,1)
(0,1) (1,1). `uv` is `trans_texcoords1` whole. Nothing it prints enters
the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

WORLDC, MISC_TERRAIN = 0x00C061D0, 0x00C06218
TEXTURE_FRACTAL = 0x00C8BF20
# `compass.obj`'s `move_x`/`move_y` and `TerrainOut.obj`'s `bit_reversal`
# (the linker map, `rise_z.map`).
MOVE_X, MOVE_Y, BIT_REVERSAL = 0x00ADCAF0, 0x00ADC400, 0x00AF1D58
NBR = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    snap = Snapshot(argv[1])
    u8 = lambda a: snap.read(a, 1)[0]
    u16 = lambda a: struct.unpack("<H", snap.read(a, 2))[0]
    u32 = lambda a: struct.unpack("<I", snap.read(a, 4))[0]
    i32 = lambda a: struct.unpack("<i", snap.read(a, 4))[0]
    f32 = lambda a: struct.unpack("<f", snap.read(a, 4))[0]
    f64 = lambda a: struct.unpack("<d", snap.read(a, 8))[0]

    def string(s):
        p, off, flags = u32(s), u16(s + 6), u8(s + 0xA)
        if not p:
            return ""
        if not flags & 1:
            p = u32(p)
        return snap.read(p + 2 * off, 256).decode("utf-16-le", "replace").split("\0")[0]

    def texname(t):
        return os.path.basename(string(t + 0x34).replace("\\", "/")) if t else "-"

    w = u32(WORLDC)
    width, height, tiles = u32(w), u32(w + 4), u32(w + 0x134)
    terrain = u32(MISC_TERRAIN)

    fr = TEXTURE_FRACTAL
    rows, x_inc, y_inc = u32(fr + 0x10), f64(fr + 0x68), f64(fr + 0x70)
    flags, parts0 = i32(fr + 0x20), i32(fr + 0x24)
    xs, ys = i32(fr + 0x18), i32(fr + 0x1C)
    print(f"terrain {width} {height} fractal {xs} {ys} flags {flags} parts0 {parts0} x_inc {x_inc!r} y_inc {y_inc!r}")

    def frac(x, y):
        # `Fractal::get_height`, in its order of doubles.
        dx = x_inc * (x + 0.5)
        dy = y_inc * (y + 0.5)
        i, j = int(dx), int(dy)
        fx, fy = dx - i, dy - j
        ox, oy = 1.0 - fx, 1.0 - fy
        r0, r1 = u32(rows + 0x10 + i * 0x1C), u32(rows + 0x10 + (i + 1) * 0x1C)
        t = ox * oy * u8(r0 + j)
        t = t + 0.0
        t = t + oy * fx * u8(r1 + j)
        t = t + ox * fy * u8(r0 + j + 1)
        t = t + u8(r1 + j + 1) * (fy * fx)
        h = max(0, min(255, int(t)))
        if parts0 != -1:
            n = 0
            for k in range(16):
                p = i32(fr + 0x24 + 4 * k)
                if p == -1 or h < p:
                    break
                n += 1
            return n
        return (h * 100) >> 8 if flags & 2 else h

    tr = terrain + 0x4974
    n = i32(tr + 4)
    print(f"alphas {n}")
    for k in range(n):
        a = u32(u32(tr + 0x10) + 4 * k)
        codes = " ".join(f"{u32(a + 4 + 4 * c):03x}" for c in range(8))
        print(f"alpha {k} bucket {u8(a + 0x25)} sel {u8(a + 0x24)} file {texname(u32(a))} codes {codes}")

    def wdata(x, y):
        wd = tiles + (width * y + x) * 0x1C
        return struct.unpack("<bB", snap.read(wd + 2, 2))

    table = u32(terrain + 0x6AB0)
    count = 0
    for y in range(height):
        for x in range(width):
            c = u32(table + 4 * (width * y + x))
            if not c:
                continue
            pa = u32(c + 0x2C)
            if not pa:
                continue
            land, sub = wdata(x, y)
            nbr = []
            for dx, dy in NBR:
                nx, ny = x + dx, y + dy
                if 0 <= nx < width and 0 <= ny < height and wdata(nx, ny)[0] == land:
                    nbr.append(str(wdata(nx, ny)[1]))
                else:
                    nbr.append("-")
            n0, n1 = i32(pa + 4), i32(pa + 0x1C + 4)
            d1 = u32(pa + 0x1C + 0x10)
            uv = " ".join(f"{f32(d1 + 8 * v):.4f},{f32(d1 + 8 * v + 4):.4f}" for v in range(n1))
            print(
                f"patch {x} {y} land {land} sub {sub} alpha {u8(pa + 0x54)} landindex {u8(pa + 0x55)}"
                f" tex {texname(u32(pa + 0x58))} frac {frac(x, y)} nbr {' '.join(nbr)}"
                f" tc0 {n0} tc1 {n1} uv {uv}"
            )
            count += 1
    print(f"patches {count}", file=sys.stderr)

    # Item 4134: what `mark_trans_flags` leaves and the consumer reads.
    print("move_x " + " ".join(str(i32(MOVE_X + 4 * k)) for k in range(10)))
    print("move_y " + " ".join(str(i32(MOVE_Y + 4 * k)) for k in range(10)))
    print("bit_reversal " + " ".join(str(u32(BIT_REVERSAL + 4 * k)) for k in range(8)))
    ex, ey = u32(terrain + 0x499C), u32(terrain + 0x49A0)
    for y in range(height + 1):
        for x in range(width + 1):
            print(f"edge {x} {y} x {u32(ex + 4 * ((height + 1) * y + x)):03x} y {u32(ey + 4 * ((width + 1) * x + y)):03x}")
    for y in range(height):
        for x in range(width):
            land, sub = wdata(x, y)
            print(f"cell {x} {y} land {land} sub {sub}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
