#!/usr/bin/env python3
"""edge_packet.py — the map's edge border, as a packet holds it (item 4311).

    python3 tools/render/edge_packet.py <frame-snapshot.bin> > <run>-edge.txt

`docs/RENDERER.md` §3.15, "Item 4307: the map's edge border draws it". No
function is called: this reads the buckets `TerrainBorder::create_borders
@008af520` filled through `build_edge@008adda0` at load, one a cell of the
map's edge, which `add_render_bucket@008afdb0` appends to the
`small_border` and `large_border` helpers for the visible ones.

- `terrain` (`0x00C0B0F0`, the static `TerrainOut`); `terrain_border` at
  `+0x4d98` in the packet (`+0x4d58` in the type record, the usual 0x40
  past it).
- `TerrainBorder` (type record): `small_border_buckets` `+0x3b8`,
  `large_border_buckets` `+0x3d4`, each a `PtrArray` whose length is at
  `+4` and whose data is at `+0x10`.
- A bucket (`WCoordBorderData`, `0x70`): `verts` `+0x0` (12 bytes an
  item), the two face lists `+0x1c` and `+0x38` (2 bytes an index), `tex`
  `+0x54` (8 bytes); each an array object, its length at `+4` and its data
  at `+0x10`.

One header line (`frame`), then per bucket a `bucket <small|large> i verts
n faces f coarse g` line, one line a vertex — `v x y z u v` as singles —
and one line a triangle of each list, `f i j k` and `g i j k`. Nothing it
prints enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

TERRAIN = 0x00C0B0F0
BORDER = 0x4D98
BUCKETS = {"small": 0x3B8, "large": 0x3D4}


def main(path):
    s = Snapshot(path)
    out = [f"frame {s.frame}"]

    def array(o, stride):
        n = s.u32(o + 4)
        return n, s.read(s.u32(o + 0x10), stride * n) if n else b""

    for kind, off in BUCKETS.items():
        ptrs = TERRAIN + BORDER + off
        count = s.u32(ptrs + 4)
        data = s.u32(ptrs + 0x10)
        for i in range(count):
            b = s.u32(data + 4 * i)
            n, vb = array(b, 12)
            nf, fb = array(b + 0x1C, 2)
            ng, gb = array(b + 0x38, 2)
            nt, tb = array(b + 0x54, 8)
            if nt != n or nf % 3 or ng % 3:
                sys.exit(f"{path}: {kind} bucket {i}: {n} verts, {nt} uvs, {nf}/{ng} indices")
            out.append(f"bucket {kind} {i} verts {n} faces {nf // 3} coarse {ng // 3}")
            for k in range(n):
                x, y, z = struct.unpack_from("<3f", vb, 12 * k)
                u, v = struct.unpack_from("<2f", tb, 8 * k)
                out.append(f"v {x!r} {y!r} {z!r} {u!r} {v!r}")
            for tag, cnt, raw in (("f", nf, fb), ("g", ng, gb)):
                idx = struct.unpack_from(f"<{cnt}H", raw, 0)
                for t in range(0, cnt, 3):
                    out.append(f"{tag} {idx[t]} {idx[t + 1]} {idx[t + 2]}")
    print("\n".join(out))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
