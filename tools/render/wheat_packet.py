#!/usr/bin/env python3
"""wheat_packet.py — the wheat boxes `FarmsOut::draw` left in a packet (item 4650).

    python3 tools/render/wheat_packet.py <frame-snapshot.bin> > <run>-wheat.txt

`docs/RENDERER.md` §6.1. No function is called: this reads the mesh
`FarmsOut::draw@008d75f0` built into its file-scope arrays (`rise.pdb`'s
S_GDATA32, section 3 at VA 0xC06000) and the state it draws them with.

- `wheat_verts` `0xC8C464` (`Vert3Array`), `wheat_texs` `0xC8C480`
  (`TexArray`), `wheat_colors` `0xC8C49C` (`SimpleArray<ulong>`): each an
  array object, its length at `+4` and its data at `+0x10`. Ten vertices a
  plot, sixteen plots a farm, in the render tree's order.
- `farms` `0xC0A8B0`: the draw's stamp at `+472` (`frame · 200 / 3 +
  graphic_dt` when the mesh was last built), `FarmsData` at `+0x40`
  (`start_color`, `wheat_max_height` `+0x4c`, `wheat_min_height` `+0x50`).
- `MiscAccess::scene` (`0xC06000 + 524`, a pointer): `Scene::wind_vec` at
  `+0x2e8`.

Prints `stamp s`, `heights min max`, `stalk r g b a`, `wind x y` (single
bits in hex), then one line a vertex: `w x y z`, the colour (`0xAABBGGRR`),
`u v`, every field a dword in hex (a single's bits). Nothing it prints
enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

FARMS = 0xC0A8B0
VERTS, TEXS, COLORS = 0xC8C464, 0xC8C480, 0xC8C49C
SCENE = 0xC06000 + 524


def main(path):
    s = Snapshot(path)
    i32 = lambda va: struct.unpack("<i", s.read(va, 4))[0]
    f32 = lambda va: struct.unpack("<f", s.read(va, 4))[0]
    u32 = lambda va: struct.unpack("<I", s.read(va, 4))[0]
    print("stamp", i32(FARMS + 472))
    print("heights", f32(FARMS + 0x50), f32(FARMS + 0x4c))
    print("stalk", *s.read(FARMS + 0x40, 4))
    sc = u32(SCENE)
    print("wind", f"{u32(sc + 0x2e8):08x}", f"{u32(sc + 0x2ec):08x}")
    n, dv = i32(VERTS + 4), u32(VERTS + 16)
    nt, dt = i32(TEXS + 4), u32(TEXS + 16)
    nc, dc = i32(COLORS + 4), u32(COLORS + 16)
    if not (n == nt == nc):
        sys.exit(f"{path}: {n} verts, {nt} texcoords, {nc} colours")
    for k in range(n):
        words = [u32(dv + 12 * k + 4 * j) for j in range(3)] + [u32(dc + 4 * k)]
        words += [u32(dt + 8 * k + 4 * j) for j in range(2)]
        print("w", *(f"{w:08x}" for w in words))


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
