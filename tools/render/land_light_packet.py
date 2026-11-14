#!/usr/bin/env python3
"""land_light_packet.py — the land's normals and vertex colours, as a packet holds them (item 4217).

    python3 tools/render/land_light_packet.py <frame-snapshot.bin> > <run>-land-light.txt

`docs/RENDERER.md` §3.22. No function is called: this reads the arrays
`TerrainOut::calculate_norms@008710b0` and `TerrainOut::light_land_vert@
00866e50` left in the process, the land's vertex buffers' sources.

- `terrain` (`0x00C0B0F0`, the static `TerrainOut` `MiscAccess::terrain`
  points at; `docs/RENDERER.md` §3.12 reads `+0x4d80` off it). `+0x47c`
  is the array index both pointer arrays below are read at.
- `+0x3f4`: an array of pointers; item `[+0x47c]` is an array object whose
  data (`+0x10`) is the normals, one `Vector<float>` (three singles) a
  corner, `(4 · worldc + 1)²` of them row-major (`calculate_norms`).
- `+0x410`: likewise, one `ulong` a corner, `light_vert@00866f90`'s
  colour with the alpha `light_land_vert` keeps or fades.
- `+0x4a4`: the render heights, one single a corner (`calculate_norms`
  reads them; `find_render_z`).
- The corner count is the array object's length (`+4`); the row is its
  square root.

One header line (`frame`, `corners`, `row`), then one line a corner in
row-major order: `x y` and the normal's three singles, the height's
single, as hex bits, and the colour as hex. Nothing it prints enters
the repository.
"""

import math
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

TERRAIN = 0x00C0B0F0


def main(path):
    s = Snapshot(path)
    at = s.u32(TERRAIN + 0x47C)

    def array(off):
        o = s.u32(s.u32(TERRAIN + off) + 4 * at)
        return s.u32(o + 4), s.u32(o + 0x10)

    n, norms = array(0x3F4)
    m, colours = array(0x410)
    row = math.isqrt(n)
    if n != m or row * row != n:
        sys.exit(f"{path}: {n} normals, {m} colours: not one square grid")
    nb = s.read(norms, 12 * n)
    cb = s.read(colours, 4 * n)
    zb = s.read(s.u32(TERRAIN + 0x4A4), 4 * n)
    out = [f"frame {s.frame} corners {n} row {row}"]
    for i in range(n):
        x, y, z = struct.unpack_from("<3I", nb, 12 * i)
        h = struct.unpack_from("<I", zb, 4 * i)[0]
        c = struct.unpack_from("<I", cb, 4 * i)[0]
        out.append(f"{i % row} {i // row} {x:08x} {y:08x} {z:08x} {h:08x} {c:08x}")
    print("\n".join(out))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
