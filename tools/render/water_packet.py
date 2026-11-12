#!/usr/bin/env python3
"""water_packet.py — the ocean floor and the water as a packet holds them (item 4174).

    python3 tools/render/water_packet.py <frame-snapshot.bin> > <run>-water.txt

`docs/RENDERER.md` §3.12. No function is called: this reads the meshes the
executable left in its own memory at the packet's frame boundary.

- `MiscAccess::terrain` (`0x00c06218`) points at the terrain. The
  decompile's `TerrainOut` offsets sit 0x40 past `TerrainData`'s type
  record (`+0x5614` is `oceanfade_height`, `+0x564c` `water_color_solid`).
- `OceanVars` (`TerrainData +0x5524`, so terrain `+0x5564`): `oceanred`..
  `oceanalpha` +0x40, `floor*_top` +0x48, `floor*_bot` +0x50, the
  `top*_top`/`top*_bot` bytes +0x58.
- The floor drawn: `scene_ocean_floor` (`TerrainData +0x35a0`, terrain
  `+0x35e0`), a `RenderHelper`: `verts` +0x54, `colors` +0xa8, `tex0`
  +0xc4, `tex1` +0xe0, each a `PtrArray` (+4 count, +0x10 data) of
  per-pass arrays; pass 0 is read.
- The water drawn: `TerrainOut::modify_water`'s function statics, in the
  image's data (`sbl/rise_z.map`): `verts` `0x00ee179c`, `norms`
  `0x00ee17bc`, `colors` `0x00ee183c`, each a `SimpleArray` (+4 count,
  +0x10 data).

One line per record: `ocean …` first, then `floor x y z colour u0 v0 u1 v1`
and `water x y z colour nx ny nz`; floats as decimals, colours as hex.
Nothing it prints enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

TERRAIN = 0x00C06218
WATER_VERTS, WATER_NORMS, WATER_COLORS = 0x00EE179C, 0x00EE17BC, 0x00EE183C


def main(path):
    s = Snapshot(path)

    def array(at):
        h = s.read(at, 0x1C)
        return struct.unpack_from("<I", h, 4)[0], struct.unpack_from("<I", h, 0x10)[0]

    def pass0(at):
        _, d = array(at)
        return array(s.u32(d))

    t = s.u32(TERRAIN)
    ov = s.read(t + 0x5564, 0x7C)
    print(f"ocean frame {s.frame} ocean {','.join(map(str, ov[0x40:0x43]))} "
          f"floor_top {','.join(map(str, ov[0x48:0x4b]))} floor_bot {','.join(map(str, ov[0x50:0x53]))} "
          f"top {','.join(map(str, ov[0x58:0x5e]))}")
    rh = t + 0x35E0
    n, dv = pass0(rh + 0x54)
    counts = [pass0(rh + o)[0] for o in (0xA8, 0xC4, 0xE0)]
    if any(c != n for c in counts):
        sys.exit(f"{path}: the floor's arrays disagree ({n}, {counts})")
    _, dc = pass0(rh + 0xA8)
    _, d0 = pass0(rh + 0xC4)
    _, d1 = pass0(rh + 0xE0)
    for i in range(n):
        x, y, z = struct.unpack("<3f", s.read(dv + 12 * i, 12))
        u0, v0 = struct.unpack("<2f", s.read(d0 + 8 * i, 8))
        u1, v1 = struct.unpack("<2f", s.read(d1 + 8 * i, 8))
        print(f"floor {x!r} {y!r} {z!r} {s.u32(dc + 4 * i):08x} {u0!r} {v0!r} {u1!r} {v1!r}")
    (nv, dv), (nn, dn), (nc, dc) = array(WATER_VERTS), array(WATER_NORMS), array(WATER_COLORS)
    if not nv == nn == nc:
        sys.exit(f"{path}: the water's arrays disagree ({nv}, {nn}, {nc})")
    for i in range(nv):
        x, y, z = struct.unpack("<3f", s.read(dv + 12 * i, 12))
        a, b, c = struct.unpack("<3f", s.read(dn + 12 * i, 12))
        print(f"water {x!r} {y!r} {z!r} {s.u32(dc + 4 * i):08x} {a!r} {b!r} {c!r}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
