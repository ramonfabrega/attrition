#!/usr/bin/env python3
"""mountain_light_packet.py — a placed range's mesh, as a packet holds it (item 4269).

    python3 tools/render/mountain_light_packet.py <frame-snapshot.bin> > <run>-mountain-light.txt

`docs/RENDERER.md` §3.22. No function is called: this reads the arrays
`TerrainOut::render` left in `scene_mount` after `RenderHelper::
generate_normals@00847f90` and `reduce_scene_data` ran on each template's
`current_mount`.

- `terrain` (`0x00C0B0F0`, the static `TerrainOut`). Its `TerrainData`
  base sits at `+0x40`, so `TerrainData::scene_mount` (`+0x828`, a
  `RenderHelper *`) is `+0x868`: an array of sixteen `RenderHelper`s
  (`0x13c` each), one a template slot, the array `render_mountains@
  008551f0` walks.
- A `RenderHelper`'s `PtrArray`s (type record): `faces` `+0x38`, `verts`
  `+0x54`, `norms` `+0x8c`, `colors` `+0xa8`; each array's data at
  `+0x10`, item `[cur_pass]` (`+0x124`) an array object, its length at
  `+4` and its data at `+0x10`. `faces` is `ushort` indices, three a
  triangle; `verts` and `norms` three singles a vertex; `colors` one
  `ulong`.

One header line (`frame`), then per slot with vertices a `slot k verts n
faces f` line, one line a vertex — `v x y` (the corner, world units over
192) the position's z, the normal's three singles as hex bits, the colour
as hex — and one line a triangle, `f i j k`. Nothing it prints enters the
repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

TERRAIN = 0x00C0B0F0
SCENE_MOUNT = 0x868
HELPER = 0x13C
UNITS_PER_TILE = 192.0


def main(path):
    s = Snapshot(path)
    out = [f"frame {s.frame}"]
    scene = s.u32(TERRAIN + SCENE_MOUNT)
    for k in range(16):
        h = scene + HELPER * k
        cur = s.u32(h + 0x124)

        def array(off, stride):
            o = s.u32(s.u32(h + off + 0x10) + 4 * cur)
            n = s.u32(o + 4)
            return n, s.read(s.u32(o + 0x10), stride * n) if n else b""

        n, vb = array(0x54, 12)
        if n == 0:
            continue
        nn, nb = array(0x8C, 12)
        nc, cb = array(0xA8, 4)
        nf, fb = array(0x38, 2)
        if nn != n or nc != n or nf % 3:
            sys.exit(f"{path}: slot {k}: {n} verts, {nn} normals, {nc} colours, {nf} indices")
        out.append(f"slot {k} verts {n} faces {nf // 3}")
        for i in range(n):
            x, y, z = struct.unpack_from("<3f", vb, 12 * i)
            cx, cy = round(x / UNITS_PER_TILE), round(y / UNITS_PER_TILE)
            if cx * UNITS_PER_TILE != x or cy * UNITS_PER_TILE != y:
                sys.exit(f"{path}: slot {k} vertex {i} at ({x}, {y}) is not a tile corner")
            w = struct.unpack_from("<3I", nb, 12 * i)
            zb = struct.unpack_from("<I", vb, 12 * i + 8)[0]
            c = struct.unpack_from("<I", cb, 4 * i)[0]
            out.append(f"v {cx} {cy} {zb:08x} {w[0]:08x} {w[1]:08x} {w[2]:08x} {c:08x}")
        idx = struct.unpack_from(f"<{nf}H", fb, 0)
        for t in range(0, nf, 3):
            out.append(f"f {idx[t]} {idx[t + 1]} {idx[t + 2]}")
    print("\n".join(out))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
