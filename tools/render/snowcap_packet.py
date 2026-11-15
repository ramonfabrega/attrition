#!/usr/bin/env python3
"""snowcap_packet.py — the terrain's underlay patches, as a packet holds them (item 4274).

    python3 tools/render/snowcap_packet.py <frame-snapshot.bin> > <run>-underlays.txt

`docs/RENDERER.md` §3.22, "The peak draw". No function is called: this
reads the meshes `TerrainOut::add_weather_underlay@00874d00` laid into
the underlay helpers, the ones `TerrainOut::render_weathering@00852f70`
draws: slots 0 to 3 at `render_weathering(0, 3)`, slot 4 at
`render_weathering(4, 4)` after the mountains.

- `terrain` (`0x00C0B0F0`, the static `TerrainOut`). `+0x4bf0` is a
  pointer to the underlay `RenderHelper`s (`0x13c` each), one per entry
  of the tileset's `<UNDERLAYS>` in file order (`pigfarm`, `wheatfarm`,
  `rough`, `oil`, `snowcap` on the early autumn tilesets).
- A `RenderHelper`'s `PtrArray`s (type record): `faces` `+0x38`, `verts`
  `+0x54`, `norms` `+0x8c`, `colors` `+0xa8`, `tex0` `+0xc4`, `tex1`
  `+0xe0`; each array's data at `+0x10`, item `[cur_pass]` (`+0x124`) an
  array object, its length at `+4` and its data at `+0x10`.

One header line (`frame`), then per filled slot a `slot k verts n faces f`
line, one line a vertex — `v x y` (the corner in tiles), the colour as
hex (`0xAABBGGRR`), then UV0 and UV1 as four singles — and one line a
triangle, `f i j k`. A slot whose arrays lie outside the packet ends the
walk. Nothing it prints enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

TERRAIN = 0x00C0B0F0
UNDERLAYS = 0x4BF0
HELPER = 0x13C
UNITS_PER_TILE = 192.0


def main(path):
    s = Snapshot(path)
    out = [f"frame {s.frame}"]
    base = s.u32(TERRAIN + UNDERLAYS)
    for k in range(16):
        h = base + HELPER * k
        cur = s.u32(h + 0x124)

        def array(off, stride):
            o = s.u32(s.u32(h + off + 0x10) + 4 * cur)
            n = s.u32(o + 4)
            return n, s.read(s.u32(o + 0x10), stride * n) if n else b""

        try:
            n, vb = array(0x54, 12)
        except Exception:
            break
        if n == 0:
            continue
        nc, cb = array(0xA8, 4)
        n0, t0 = array(0xC4, 8)
        n1, t1 = array(0xE0, 8)
        nf, fb = array(0x38, 2)
        if nc != n or n0 != n or n1 != n or nf % 3:
            sys.exit(f"{path}: slot {k}: {n} verts, {nc} colours, {n0}/{n1} uvs, {nf} indices")
        out.append(f"slot {k} verts {n} faces {nf // 3}")
        for i in range(n):
            x, y, _ = struct.unpack_from("<3f", vb, 12 * i)
            cx, cy = round(x / UNITS_PER_TILE), round(y / UNITS_PER_TILE)
            if cx * UNITS_PER_TILE != x or cy * UNITS_PER_TILE != y:
                sys.exit(f"{path}: slot {k} vertex {i} at ({x}, {y}) is not a tile corner")
            c = struct.unpack_from("<I", cb, 4 * i)[0]
            u0 = struct.unpack_from("<2f", t0, 8 * i)
            u1 = struct.unpack_from("<2f", t1, 8 * i)
            out.append(f"v {cx} {cy} {c:08x} {u0[0]!r} {u0[1]!r} {u1[0]!r} {u1[1]!r}")
        idx = struct.unpack_from(f"<{nf}H", fb, 0)
        for t in range(0, nf, 3):
            out.append(f"f {idx[t]} {idx[t + 1]} {idx[t + 2]}")
    print("\n".join(out))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
