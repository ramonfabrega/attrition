#!/usr/bin/env python3
"""road_records.py — every road tile's `RoadElementCandidate`, and the pieces, on a packet (item 4199).

    python3 tools/render/road_records.py <frame-snapshot.bin> [x0 y0 x1 y1]

`docs/RENDERER.md` §3.19. No function is called: this reads what
`Roads::add_roads@0088f4b0` and its texture passes left in the process.
`MiscAccess::roads` (`0x00C06238`, a reference) is the `RoadsOut`; the
`RoadsData` sits at `+0x3c` in it, so `RoadsPieces::modern_elements` is a
`PtrArray<RoadElement>` at `+0x40` (count `+0x44`, list `+0x50`), as
`RoadsOut::build_render_data@00894890` and `find_piece@00890c90` read it.
`MiscAccess::terrain` (`0x00C06218`) `+0x47d8` is `TerrainOut`'s
`width + height` `TerrainList`s; each list's `pLinearCoords` (+8) chains every
`CoordInfo` (0x80) by `pNextLinear` (+8). A `CoordInfo`'s `roads_in_wcoord`
(+0x5c) is sixteen `RoadElementCandidate`s (0x10), tile `(xw·4 + i&3,
yw·4 + i>>2)`. Rows:

    element <n> <name> page <pagenum> w <width> h <height> uv <tu> <tv> <tu_w> <tv_w> flags <8 × hex>
    road <tx> <ty> elem <n> flags <hex> rot <r> age <prev_age> refs <ref_count>/<pending> support <hex> seen <hex> made <is_terrain_creation>

A record whose `element_num` is −1 and whose `flags` are zero is skipped.
Floats print as Python reprs of the f32. Nothing it prints enters the repository.
"""

import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "recomp"))
from snapshot import Snapshot  # noqa: E402

WORLDC, MISC_TERRAIN, MISC_ROADS = 0x00C061D0, 0x00C06218, 0x00C06238


def main(argv):
    if len(argv) not in (2, 6):
        print(__doc__)
        return 2
    box = tuple(int(a) for a in argv[2:6]) if len(argv) == 6 else None
    snap = Snapshot(argv[1])
    u32 = lambda a: struct.unpack("<I", snap.read(a, 4))[0]
    i32 = lambda a: struct.unpack("<i", snap.read(a, 4))[0]
    f32 = lambda a: struct.unpack("<f", snap.read(a, 4))[0]
    print(f"frame {snap.frame}")
    roads = u32(MISC_ROADS)
    n, lst = i32(roads + 0x44), u32(roads + 0x50)
    for e in range(n):
        el = u32(lst + 4 * e)
        name = snap.read(u32(el), 32).decode("utf-16-le", "replace").split("\0")[0]
        page, w, h = snap.read(el + 0x14, 3)
        uv = " ".join(repr(f32(el + o)) for o in (0x58, 0x5C, 0x60, 0x64))
        fl = " ".join(f"{u32(el + 0x68 + 4 * r):08x}" for r in range(8))
        cx = " ".join(repr(f32(el + 0x18 + 4 * r)) for r in range(8))
        cy = " ".join(repr(f32(el + 0x38 + 4 * r)) for r in range(8))
        print(f"element {e} {name} page {page} w {w} h {h} uv {uv} flags {fl} cx {cx} cy {cy}")
    w = u32(WORLDC)
    width, height = u32(w), u32(w + 4)
    terrain = u32(MISC_TERRAIN)
    lists = u32(terrain + 0x47D8)
    rows = {}
    for k in range(width + height):
        c = u32(lists + 0xC * k + 8)
        seen = 0
        while c and seen < width * height:
            xw, yw = i32(c + 0x14), i32(c + 0x18)
            r = u32(c + 0x5C)
            if r:
                for i in range(16):
                    tx, ty = xw * 4 + (i & 3), yw * 4 + (i >> 2)
                    if box and not (box[0] <= tx <= box[2] and box[1] <= ty <= box[3]):
                        continue
                    b = snap.read(r + 0x10 * i, 0x10)
                    elem, flags = struct.unpack_from("<iI", b, 0)
                    if elem == -1 and flags == 0:
                        continue
                    rot, age, refs, pend = b[8], b[9], b[10], b[11]
                    sup = struct.unpack_from("<H", b, 12)[0]
                    rows[(ty, tx)] = (
                        f"road {tx} {ty} elem {elem} flags {flags:08x} rot {rot} age {age} "
                        f"refs {refs}/{pend} support {sup:04x} seen {b[14]:02x} made {b[15]}"
                    )
            c = u32(c + 8)
            seen += 1
    for k in sorted(rows):
        print(rows[k])
    print(f"roads {len(rows)}", file=sys.stderr)
    pieces(snap, roads, u32, i32)
    return 0


def pieces(snap, roads, u32, i32):
    """`RoadsData::current_roads` (`RoadsOut +0x5c0`, an `ObjectArray<RenderHelper>`,
    count `+0x5c4`, the objects inline at `+0x5d0`, 0x13c each): helper
    `pagenum · 3 + age bucket`, as `build_render_data` picks it. Each pass's
    `verts` (`+0x54`), `tex0` (`+0xc4`) and `faces` (`+0x38`, `u16`
    indices, three a triangle) are `PtrArray`s of arrays (count `+4`, data
    `+0x10`). A piece is a set of vertices its faces join; it prints as its
    box in tiles (192 units a tile) and the texcoord at each corner, NW, NE,
    SW, SE, the vertex standing on it:

        piece <helper> <x0> <y0> <x1> <y1> verts <n> uv <u,v> <u,v> <u,v> <u,v>

    The buffer can hold a piece twice; repeats are dropped."""
    n, lst = i32(roads + 0x5C4), u32(roads + 0x5D0)
    out = set()
    for h in range(n):
        rh = lst + 0x13C * h
        for p in range(i32(rh + 0x124) + 1):

            def arr(off):
                if p >= i32(rh + off + 4):
                    return 0, 0
                a = u32(u32(rh + off + 0x10) + 4 * p)
                return i32(a + 4), u32(a + 0x10)

            nv, va = arr(0x54)
            nt, ta = arr(0xC4)
            nf, fa = arr(0x38)
            if nv == 0 or nt < nv:
                continue
            V = [struct.unpack("<fff", snap.read(va + 12 * k, 12)) for k in range(nv)]
            T = [struct.unpack("<ff", snap.read(ta + 8 * k, 8)) for k in range(nv)]
            idx = struct.unpack(f"<{nf}H", snap.read(fa, 2 * nf))
            up = list(range(nv))

            def root(a):
                while up[a] != a:
                    up[a] = up[up[a]]
                    a = up[a]
                return a

            for t in range(0, nf - 2, 3):
                a = root(idx[t])
                for b in idx[t + 1 : t + 3]:
                    up[root(b)] = a
            groups = {}
            for k in range(nv):
                groups.setdefault(root(k), []).append(k)
            for g in groups.values():
                if len(g) < 4:
                    continue
                xs = [V[k][0] / 192 for k in g]
                ys = [V[k][1] / 192 for k in g]
                x0, y0, x1, y1 = min(xs), min(ys), max(xs), max(ys)

                def at(x, y):
                    for k in g:
                        if V[k][0] / 192 == x and V[k][1] / 192 == y:
                            return f"{T[k][0]!r},{T[k][1]!r}"
                    return "-"

                uv = " ".join(at(x, y) for x, y in ((x0, y0), (x1, y0), (x0, y1), (x1, y1)))
                out.add(f"piece {h} {x0!r} {y0!r} {x1!r} {y1!r} verts {len(g)} uv {uv}")
    for row in sorted(out):
        print(row)
    print(f"pieces {len(out)}", file=sys.stderr)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
