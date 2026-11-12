#!/usr/bin/env python3
"""sand_packet.py — the land's corner alphas and the sand layer as a packet holds them (item 4192).

    python3 tools/render/sand_packet.py <frame-snapshot.bin> > <run>-sand.txt

`docs/RENDERER.md` §3.12, "The shore's sand (item 4192)". No function is
called: this reads what the executable left in its own memory at the
packet's frame boundary. Offsets are the decompile's `TerrainOut` ones,
which sit 0x40 past `TerrainData`'s type record.

- `MiscAccess::terrain` (`0x00c06218`) points at the terrain.
- `master_land` (`TerrainData +0x318`, terrain `+0x358`), a `RenderHelper`:
  its `colors` (+0xa8) are the land's corner colours,
  `TerrainOut::light_land_vert@00866e50`'s alpha in the top byte, one per
  tile corner of the `(4 · width + 1)²` grid; `master_land_heights`
  (`TerrainData +0x454`, terrain `+0x494`) the corner heights beside them.
- The land pass's per-texture helpers: an array at terrain `+0x4b0`, one
  `RenderHelper` (0x13c bytes) per linear land texture, which
  `CoordInfo::add_geometry@0084bef0` fills four corners a tile; the
  non-empty one is printed (on run1564, index 36: `sand.tga`, the card's
  draw 35).
- `scene_coral` (`TerrainData +0x1830`, terrain `+0x1870`): the vertex
  count only, the card's draw 131 on the same corners.
- `coast_depth` (terrain `+0x5630`) and `land_height` (`0x00cbe54c`,
  `sbl/rise_z.map`).

One line per record: `land …` first, then `corner ix iy h alpha` and
`sand index x y z colour u0 v0`; floats as decimals, colours as hex.
Nothing it prints enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

TERRAIN = 0x00C06218
LAND_HEIGHT = 0x00CBE54C
HELPER = 0x13C


def main(path):
    s = Snapshot(path)

    def array(at):
        h = s.read(at, 0x1C)
        return struct.unpack_from("<I", h, 4)[0], struct.unpack_from("<I", h, 0x10)[0]

    def pass0(at):
        n, d = array(at)
        return array(s.u32(d)) if n else (0, 0)

    def f32(at):
        return struct.unpack("<f", s.read(at, 4))[0]

    t = s.u32(TERRAIN)
    n, dc = pass0(t + 0x358 + 0xA8)
    nh, dh = array(t + 0x494)
    side = int(round(n ** 0.5))
    if nh != n or side * side != n:
        sys.exit(f"{path}: the land's colours and heights disagree ({n}, {nh})")
    helpers = s.u32(t + 0x4B0)
    # The array ends where a slot stops carrying the helper's `faces`
    # `PtrArray` vtable (run1564: 38 slots, 0..37).
    vtable = s.u32(helpers + 0x38)
    full = []
    for k in range(256):
        if s.u32(helpers + k * HELPER + 0x38) != vtable:
            break
        nv, _ = pass0(helpers + k * HELPER + 0x54)
        if nv:
            full.append(k)
    if len(full) != 1:
        sys.exit(f"{path}: expected one filled land helper, found {full}")
    k = full[0]
    rh = helpers + k * HELPER
    nv, dv = pass0(rh + 0x54)
    _, dsc = pass0(rh + 0xA8)
    _, d0 = pass0(rh + 0xC4)
    coral, _ = pass0(t + 0x1870 + 0x54)
    print(f"land frame {s.frame} side {side} coast_depth {f32(t + 0x5630)!r} "
          f"land_height {f32(LAND_HEIGHT)!r} sand_index {k} sand {nv} coral {coral}")
    colours = struct.unpack(f"<{n}I", s.read(dc, 4 * n))
    heights = struct.unpack(f"<{n}f", s.read(dh, 4 * n))
    for i in range(n):
        print(f"corner {i % side} {i // side} {heights[i]!r} {colours[i] >> 24}")
    for i in range(nv):
        x, y, z = struct.unpack("<3f", s.read(dv + 12 * i, 12))
        u0, v0 = struct.unpack("<2f", s.read(d0 + 8 * i, 8))
        print(f"sand {k} {x!r} {y!r} {z!r} {s.u32(dsc + 4 * i):08x} {u0!r} {v0!r}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
